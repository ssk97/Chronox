use library::*;
//use ggez::nalgebra as na;

pub use orders::*;
pub use petgraph::prelude::*;
pub type NodeInd = NodeIndex<u16>;
pub type EdgeInd = EdgeIndex<u16>;

pub use plain_enum::*;
plain_enum_mod!(player_enum, derive(FromPrimitive, ToPrimitive, Serialize, Deserialize,), map_derive(Serialize, Deserialize, Copy, ), Player {
    PASSIVE,
    P1, P2, /*P3, P4,*/
});

pub type PlayerArr<T> = EnumMap<Player, T>;

pub const ARMY_SPEED: i32 = 100;
pub const SPAWN_NEEDED: u32 = 64;

pub struct SimMetadata{
    pub total_planet: PlayerArr<u32>,
    pub total_dead: PlayerArr<u32>,
    pub total_transit: PlayerArr<u32>,
}
impl SimMetadata{
    pub fn new()->SimMetadata{
        SimMetadata{total_planet: PlayerArr::new(0), total_dead:PlayerArr::new(0), total_transit:PlayerArr::new(0)}
    }
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
        }
    }
    fn advance(&mut self, total_planet: &mut PlayerArr<u32>, total_dead: &mut PlayerArr<u32>){
        //if owned, spawn more
        if self.owner != Player::PASSIVE {
            self.spawn_progress += 10;
            if self.spawn_progress >= SPAWN_NEEDED{
                self.spawn_progress -= SPAWN_NEEDED;
                self.count[self.owner] += 1;
            }
        } else {
            self.spawn_progress = 0;
        }
        //fight!
        let sides_found = find_sides_node(&self);
        for side in sides_found.clone(){
            total_planet[side] += self.count[side];
        }
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
            for p_ref in &sides_found{
                let p = *p_ref;
                self.fight_progess[p] += self.count[p];//TODO: modify algorithm?
                let kills = self.fight_progess[p]/100;
                if kills > 0 {
                    self.fight_progess[p] -= 100*kills;
                    for p2_ref in &sides_found {
                        let p2 = *p2_ref;
                        if p2 != p {
                            self.count[p2] += kills;
                            total_dead[p2] += kills;
                        }
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

pub type WorldGraph =  Graph<Planet, HyperLane, Undirected, u16>;
pub struct Simulation{
    pub world:WorldGraph,
    pub timestep: ChronalTime,
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
    pub fn update(&self, orders: &Vec<ChronalCommand>) -> (Simulation, SimMetadata) {
        let mut transfer_set: Vec<(NodeInd, Player, u32)> = Vec::new();
        let mut metadata = SimMetadata::new();
        let mut new_world: WorldGraph;
        {//metadata borrow scope
            let total_dead = &mut metadata.total_dead;
            let total_planet = &mut metadata.total_planet;
            let total_transit = &mut metadata.total_transit;
            new_world = self.world.map(
                |_node_ind, node| {
                    let mut new_node = node.clone();
                    new_node.advance(total_planet, total_dead);
                    new_node
                },
                |edge_ind, edge| {
                    //move armies around, if at end send them to that planet
                    let mut new_vec = Vec::new();
                    let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
                    let edge_len = edge.length;
                    for group in &edge.transfers {
                        total_transit[group.player] += group.count;
                        if group.progress > edge_len {
                            let ending = match group.direction {
                                DIR::FORWARD => t_ind,
                                DIR::BACKWARD => s_ind,
                            };
                            transfer_set.push((ending, group.player, group.count));
                        } else {
                            new_vec.push(ArmyGroup {
                                direction: group.direction,
                                progress: group.progress + ARMY_SPEED,
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
            if self.timestep % 5 == 0 {
                for node_ind in new_world.node_indices() {
                    for p in Player::values() {
                        if let Some(target) = new_world[node_ind].send_all[p] {
                            send_out(&mut new_world, node_ind, target, p, 100);
                        }
                    }
                }
            }

            for order in orders {
                match order {
                    &ChronalCommand::Transport(ref data) => {
                        send_out(&mut new_world, data.from, data.to, data.player, data.percent);
                    }
                    &ChronalCommand::SendAll(ref data) => {
                        let node = &mut new_world[data.from];
                        node.send_all[data.player] = data.to;
                    }
                }
            }
        }
        (Simulation{world: new_world, timestep: self.timestep+1}, metadata)
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