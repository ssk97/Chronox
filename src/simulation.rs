use library::*;
//use ggez::nalgebra as na;

pub extern crate petgraph;
use self::petgraph::prelude::*;
pub type NodeInd = NodeIndex<u32>;
pub type EdgeInd = EdgeIndex<u32>;

use plain_enum::*;
plain_enum_mod!(player_enum, Player {
    PASSIVE,
    P1, P2, P3, P4,
});
pub const MAX_SIDES:usize = 5;

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
    pub spawn_progress: u32,
    pub spawn_needed: u32
}
impl Planet{
    pub fn new(loc: Ipt) -> Planet{
        let mut count = [0;MAX_SIDES];
        count[0] = 10;
        Planet{
            loc,
            count,
            fight_progess: [0;MAX_SIDES],

            owner: Player::PASSIVE,
            spawn_progress: 0,
            spawn_needed: 64
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

pub struct Simulation{
    pub world: Graph<Planet, Edge, Undirected>,
    timestep: u64,
}

pub struct TransportCommand{
    pub from: NodeInd,
    pub to: NodeInd,
    pub percent: u8
}
pub enum CommandEnum{
    Transport(TransportCommand)
}
pub struct Order{
    pub player: Player,
    pub command: CommandEnum
}

pub fn count_sides_node(node: &Planet) -> u8{
    let mut sides_found:u8 = 0;
    for p in Player::values(){
        if node.count[p] > 0{
            sides_found += 1;
        }
    }
    sides_found
}
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
    pub fn new() -> Simulation{
        let mut g = Graph::new_undirected();
        let mut starting_planet = Planet::new(ipt(100, 100));
        starting_planet.owner = Player::P1;
        starting_planet.count[Player::P1] = 20;
        let node_a = g.add_node(starting_planet);
        let node_b = g.add_node(Planet::new(ipt(800, 200)));
        let node_c = g.add_node(Planet::new(ipt(400, 600)));
        g.add_edge(node_a, node_b, Edge::new());
        g.add_edge(node_b, node_c, Edge::new());
        Simulation{world: g, timestep: 0}
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
                        let transfer_amount = (node.count[player] * percent) / 100; // TODO: make percentage or something?
                        node.count[player] -= transfer_amount;
                        let order_dir = match dir {
                            Direction::Outgoing => DIR::FORWARD,
                            Direction::Incoming => DIR::BACKWARD
                        };
                        let new_follow = ArmyGroup { direction: order_dir, progress: 0, count: transfer_amount, player };
                        edge.transfers.push(new_follow);
                    } else {
                        panic!("Edge no longer exists");
                    }
                }
            }
        }
    }
    pub fn count_sides(&self, node: NodeInd) -> u8{
        count_sides_node(&self.world[node])
    }
    pub fn find_sides(&self, node: NodeInd) -> Vec<Player>{
        find_sides_node(&self.world[node])
    }

    pub fn update(&mut self, conf: &GameConfig) {
        //move armies around, if at end send them to that planet
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
                node.spawn_progress += 1;
                if node.spawn_progress >= node.spawn_needed {
                    node.spawn_progress -= node.spawn_needed;
                    node.count[node.owner] += 1;
                }
            }
            //fight!
            let sides_found = count_sides_node(node);
            if sides_found < 2{//zero out all fighting progress if only one side
                node.fight_progess = [0;MAX_SIDES];
            } else {
                let mut total_removal = 0;
                for p in Player::values(){
                    node.fight_progess[p] += node.count[p];//TODO: modify algorithm?
                    let kills = node.fight_progess[p]/100;
                    if kills > 0 {
                        node.fight_progess[p] -= 100*kills;
                        node.count[p] += kills;
                        total_removal += kills;
                    }
                }
                if total_removal > 0 {
                    for p in Player::values() {
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
}