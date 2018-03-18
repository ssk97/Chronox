use library::*;
//use ggez::nalgebra as na;

pub extern crate petgraph;
use self::petgraph::prelude::*;
use std::collections::VecDeque;
pub type NodeInd = NodeIndex<u32>;
pub type EdgeInd = EdgeIndex<u32>;

use plain_enum::*;
plain_enum_mod!(player_enum, derive(FromPrimitive, ToPrimitive, Serialize, Deserialize,), map_derive(), Player {
    PASSIVE,
    P1, P2, P3, P4,
});
pub const MAX_SIDES:usize = Player::SIZE;

use std::ops;

//This is a dirty hack to ge around E0210.
impl ops::Index<Player> for [u32;MAX_SIDES]{
    type Output = u32;
    fn index(&self, p: Player) -> &u32{
        let val = p as usize;
        &self[val]
    }
}
impl ops::IndexMut<Player> for [u32;MAX_SIDES]{
    fn index_mut(&mut self, p: Player) -> &mut u32{
        let val = p as usize;
        &mut self[val]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameConfig{
    pub army_speed: i32
}

pub struct Planet {
    pub loc: Ipt,
    pub count: [u32;MAX_SIDES],
    pub fight_progess: [u32;MAX_SIDES],
    pub owner: Player,
    pub owner_strength: u32,
    pub max_strength: u32,
    pub spawn_progress: u32,
    pub spawn_needed: u32,
}
impl Planet{
    pub fn new(loc: Ipt, owner: Player) -> Planet{
        let mut count = [0;MAX_SIDES];
        count[owner] = 10;
        Planet{
            loc,
            count,
            fight_progess: [0;MAX_SIDES],
            owner,
            owner_strength: 64,
            max_strength: 64,
            spawn_progress: 0,
            spawn_needed: 64,
        }
    }
}
pub struct Edge{
    pub length: i32,
    pub transfers: Vec<ArmyGroup>
}
impl Edge{
    pub fn new() -> Edge{
        Edge{transfers: Vec::new(), length: 5000}
    }
}
pub enum DIR{FORWARD, BACKWARD}
pub struct ArmyGroup{
    pub direction: DIR,
    pub progress: i32,
    pub count: u32,
    pub player: Player
}

pub type WorldGraph =  Graph<Planet, Edge, Undirected>;
pub struct Simulation{
    pub world:WorldGraph,
    pub timestep: u64,
}

#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct TransportCommand{
    pub from: NodeInd,
    pub to: NodeInd,
    pub percent: u8
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum CommandEnum{
    Transport(TransportCommand)
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct Order{
    pub player: Player,
    pub command: CommandEnum
}

pub type OrdersType = VecDeque<Vec<Order>>;

pub fn find_sides_node(node: &Planet) -> Vec<Player>{
    let mut sides_found = Vec::new();
    for p in Player::values(){
        if node.count[p] > 0{
            sides_found.push(p);
        }
    }
    sides_found
}
impl Simulation{
    pub fn new(world: WorldGraph) -> Simulation{
        Simulation{world, timestep: 0}
    }

    pub fn handle_orders(&mut self, _conf: &GameConfig, orders: &Vec<Order>){
        for order in orders {
            let player = order.player;
            match order.command {
                CommandEnum::Transport(ref data) => {
                    let percent = data.percent as u32;
                    let edge_data = self.world.find_edge_undirected(data.from, data.to);
                    if let Some((edge_ind, dir)) = edge_data {
                        let (edge, node) = self.world.index_twice_mut(edge_ind, data.from);
                        let transfer_amount = (node.count[player] * percent) / 100;
                        if transfer_amount > 0 {
                            node.count[player] -= transfer_amount;
                            let order_dir = match dir {
                                Direction::Outgoing => DIR::FORWARD,
                                Direction::Incoming => DIR::BACKWARD
                            };
                            let new_follow = ArmyGroup { direction: order_dir, progress: 0, count: transfer_amount, player };
                            edge.transfers.push(new_follow);
                        }
                    } else {
                        panic!("Edge no longer exists");
                    }
                }
            }
        }
    }
    pub fn find_sides(&self, node: NodeInd) -> Vec<Player>{
        find_sides_node(&self.world[node])
    }

    pub fn update(&mut self, conf: &GameConfig) {
        //move armies around, if at end send them to that planet
        self.timestep += 1;
        for edge_ind in self.world.edge_indices() {
            let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
            let mut transfer_set: Vec<(NodeInd, Player, u32)> = Vec::new();
            {
                let edge = &mut self.world[edge_ind];
                let edge_len = edge.length;
                for group in &mut edge.transfers {
                    group.progress += conf.army_speed;
                    if group.progress > edge_len {
                        let ending;
                        match group.direction {
                            DIR::FORWARD => ending = t_ind,
                            DIR::BACKWARD => ending = s_ind
                        }
                        transfer_set.push((ending, group.player, group.count));
                    }
                }
                edge.transfers.retain(|ref f| !(f.progress > edge_len));
            }
            for removal in transfer_set {
                let node = &mut self.world[removal.0];
                node.count[removal.1] += removal.2;
            }
        }
        for node in self.world.node_weights_mut() {
            //spawn more on owned nodes
            if node.owner != Player::PASSIVE {
                node.spawn_progress += 10;
                if node.spawn_progress >= node.spawn_needed {
                    node.spawn_progress -= node.spawn_needed;
                    node.count[node.owner] += 1;
                }
            } else {
                node.spawn_progress = 0;
            }
            //fight!
            let sides_found = find_sides_node(node);
            let sides_count = sides_found.len();
            if sides_count < 2{//zero out all fighting progress if no battle
                node.fight_progess = [0;MAX_SIDES];
                if sides_count == 1{//and advance ownership of the winner
                    if node.count[node.owner] == 0 {//owner has lost
                        node.owner_strength -= 1;
                        if node.owner_strength <= 0{
                            if node.owner == Player::PASSIVE{
                                node.owner = sides_found[0];
                            } else {
                                node.owner = Player::PASSIVE;
                                node.owner_strength = node.max_strength;
                            }
                        }
                    } else {//owner has won
                        if node.owner_strength <= node.max_strength {
                            node.owner_strength += 1
                        }
                    }
                }
            } else {
                //otherwise, do fighting
                let mut total_removal = 0;
                for p_ref in &sides_found{
                    let p = *p_ref;
                    node.fight_progess[p] += node.count[p];//TODO: modify algorithm?
                    let kills = node.fight_progess[p]/100;
                    if kills > 0 {
                        node.fight_progess[p] -= 100*kills;
                        node.count[p] += kills;
                        total_removal += kills;
                    }
                }
                if total_removal > 0 {
                    for p_ref in &sides_found {
                        let p = *p_ref;
                        if node.count[p] > total_removal {
                            node.count[p] -= total_removal;
                        } else {
                            node.count[p] = 0;
                        }
                    }
                }
            }
        }
    }
    pub fn check_planets(&self, pos: Ipt, max_dist: i32) -> Option<NodeInd>{
        let mut dist = i32::max_value();
        let mut best = None;
        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            let tmpdist = dist2(&pos, &node.loc);
            if tmpdist < dist{
                best = Some(node_ind);
                dist = tmpdist;
            }
        }
        if dist < max_dist*max_dist{
            return best;
        } else {
            return None;
        }
    }


}