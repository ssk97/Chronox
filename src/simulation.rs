use library::*;
//use ggez::nalgebra as na;

pub extern crate petgraph;
use self::petgraph::prelude::*;
pub type NodeInd = NodeIndex<u32>;
pub type EdgeInd = EdgeIndex<u32>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GameConfig{
    army_speed: i32
}

pub struct Planet {
    pub loc: Ipt,
    pub count: u64,
    pub spawn_progress: u32,
    pub spawn_needed: u32
}
impl Planet{
    pub fn new(loc: Ipt) -> Planet{
        Planet{
            loc,
            count: 0,
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
    pub count: u64
}

pub struct Simulation{
    pub world: Graph<Planet, Edge, Undirected>,
    timestep: u64,
}

impl Simulation{
    pub fn new() -> Simulation{
        let mut g = Graph::new_undirected();
        let node_a = g.add_node(Planet::new(ipt(100, 100)));
        let node_b = g.add_node(Planet::new(ipt(800, 200)));
        let node_c = g.add_node(Planet::new(ipt(400, 600)));
        g.add_edge(node_a, node_b, Edge::new());
        g.add_edge(node_b, node_c, Edge::new());
        Simulation{world: g, timestep: 0}
    }

    pub fn update(&mut self, conf: &GameConfig) {
        for node in self.world.node_weights_mut() {
            node.spawn_progress += 1;
            if node.spawn_progress >= node.spawn_needed {
                node.spawn_progress -= node.spawn_needed;
                node.count += 1;
            }
        }
        for edge_ind in self.world.edge_indices() {
            let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
            let mut transfer_set: Vec<(NodeInd, u64)> = Vec::new();
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
                        transfer_set.push((ending, group.count));
                    }
                }
                edge.transfers.retain(|ref f| !(f.progress > edge_len));
            }
            for removal in transfer_set {
                let node = &mut self.world[removal.0];
                node.count += removal.1;
            }
        }
    }

}