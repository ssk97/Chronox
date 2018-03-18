use library::*;
//use ggez::nalgebra as na;

pub use orders::*;
pub use petgraph::prelude::*;
pub type NodeInd = NodeIndex<u32>;
pub type EdgeInd = EdgeIndex<u32>;

use plain_enum::*;
plain_enum_mod!(player_enum, derive(FromPrimitive, ToPrimitive, Serialize, Deserialize,), map_derive(Serialize, Deserialize, Copy, ), Player {
    PASSIVE,
    P1, P2, P3, P4,
});
pub const MAX_SIDES:usize = Player::SIZE;

pub type PlayerArr<T> = EnumMap<Player, T>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GameConfig{
    pub army_speed: i32
}

#[derive(Copy, Clone)]
pub struct Planet {
    pub loc: Ipt,
    pub count: PlayerArr<u32>,
    pub fight_progess: PlayerArr<u32>,
    pub send_all: PlayerArr<Option<NodeInd>>,
    pub owner: Player,
    pub owner_strength: u32,
    pub max_strength: u32,
    pub spawn_progress: u32,
    pub spawn_needed: u32,
}
impl Planet{
    pub fn new(loc: Ipt, owner: Player) -> Planet{
        let mut count = PlayerArr::new(0);
        count[owner] = 10;
        Planet{
            loc,
            count,
            fight_progess: PlayerArr::new(0),
            send_all: PlayerArr::new(None),
            owner,
            owner_strength: 64,
            max_strength: 64,
            spawn_progress: 0,
            spawn_needed: 64,
        }
    }
    fn advance(&mut self){
        //if owned, spawn more
        if self.owner != Player::PASSIVE {
            self.spawn_progress += 10;
            if self.spawn_progress >= self.spawn_needed {
                self.spawn_progress -= self.spawn_needed;
                self.count[self.owner] += 1;
            }
        } else {
            self.spawn_progress = 0;
        }
        //fight!
        let sides_found = find_sides_node(&self);
        let sides_count = sides_found.len();
        if sides_count < 2{//zero out all fighting progress if no battle
            self.fight_progess = PlayerArr::new(0);
            if sides_count == 1{//and advance ownership of the winner
                if self.count[self.owner] == 0 {//owner has lost
                    self.owner_strength -= 1;
                    if self.owner_strength <= 0{
                        if self.owner == Player::PASSIVE{
                            self.owner = sides_found[0];
                        } else {
                            self.owner = Player::PASSIVE;
                            self.owner_strength = self.max_strength;
                        }
                    }
                } else {//owner has won
                    if self.owner_strength <= self.max_strength {
                        self.owner_strength += 1
                    }
                }
            }
        } else {
            //otherwise, do fighting
            let mut total_removal = 0;
            for p_ref in &sides_found{
                let p = *p_ref;
                self.fight_progess[p] += self.count[p];//TODO: modify algorithm?
                let kills = self.fight_progess[p]/100;
                if kills > 0 {
                    self.fight_progess[p] -= 100*kills;
                    self.count[p] += kills;
                    total_removal += kills;
                }
            }
            if total_removal > 0 {
                for p_ref in &sides_found {
                    let p = *p_ref;
                    if self.count[p] > total_removal {
                        self.count[p] -= total_removal;
                    } else {
                        self.count[p] = 0;
                    }
                }
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct ArmyGroup{
    pub direction: DIR,
    pub progress: i32,
    pub count: u32,
    pub player: Player
}

pub struct HyperLane{
    pub length: i32,
    pub transfers: Vec<ArmyGroup>
}
#[derive(Copy, Clone)]
pub enum DIR{FORWARD, BACKWARD}

impl HyperLane{
    pub fn new() -> HyperLane{
        HyperLane{transfers: Vec::new(), length: 5000}
    }
}

pub type WorldGraph =  Graph<Planet, HyperLane, Undirected>;
pub struct Simulation{
    pub world:WorldGraph,
    pub timestep: u64,
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

fn send_out(world: &mut WorldGraph, from: NodeInd, to: NodeInd, player:Player, percent: u8){
    let edge_data = world.find_edge_undirected(from, to);
    if let Some((edge_ind, dir)) = edge_data {
        let (edge, node) = world.index_twice_mut(edge_ind, from);
        let transfer_amount = (node.count[player] * (percent as u32)) / 100;
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

impl Simulation{
    pub fn new(world: WorldGraph) -> Simulation{
        Simulation{world, timestep: 0}
    }

    pub fn find_sides(&self, node: NodeInd) -> Vec<Player>{
        find_sides_node(&self.world[node])
    }

    //given self, advance a timestep and return the new Simulation representing it
    pub fn update(&self, conf: &GameConfig, orders: &Vec<Order>) -> Simulation {
        let mut transfer_set: Vec<(NodeInd, Player, u32)> = Vec::new();
        let mut new_world: WorldGraph = self.world.map(
            |_node_ind, node| {
                let mut new_node = node.clone();
                new_node.advance();
                new_node
            },
            |edge_ind, edge| {
                //move armies around, if at end send them to that planet
                let mut new_vec = Vec::new();
                let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
                let edge_len = edge.length;
                for group in &edge.transfers {
                    if group.progress > edge_len {
                        let ending = match group.direction {
                            DIR::FORWARD => t_ind,
                            DIR::BACKWARD => s_ind,
                        };
                        transfer_set.push((ending, group.player, group.count));
                    } else {
                        new_vec.push(ArmyGroup {
                            direction: group.direction,
                            progress: group.progress + conf.army_speed,
                            count: group.count,
                            player: group.player
                        });
                    }
                }
                HyperLane { length: edge_len, transfers: new_vec }
            }
        );
        for removal in transfer_set {
            let node = &mut new_world[removal.0];
            node.count[removal.1] += removal.2;
        }
        //every half-second (5 timesteps) check for "send all" commands
        if self.timestep%5 == 0 {
            for node_ind in new_world.node_indices() {
                for p in Player::values() {
                    if let Some(target) = new_world[node_ind].send_all[p] {
                        send_out(&mut new_world, node_ind, target, p, 100);
                    }
                }
            }
        }

        for order in orders {
            let player = order.player;
            match order.command {
                CommandEnum::Transport(ref data) => {
                    send_out(&mut new_world, data.from, data.to, player, data.percent);
                }
                CommandEnum::SendAll(ref data) => {
                    let node = &mut new_world[data.from];
                    node.send_all[player] = data.to;
                }
            }
        }
        Simulation{world: new_world, timestep: self.timestep+1}
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