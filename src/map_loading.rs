use simulation::*;
use simulation::petgraph::prelude::*;
use library::*;

use std::collections::HashMap;

use num::FromPrimitive;
#[derive(Serialize, Deserialize, Debug)]
struct Globals{
    map_size: Vec<i64>
}
#[derive(Serialize, Deserialize, Debug)]
struct MapPlanet{
    id: String,
    loc: Vec<i64>,
    owner: Option<i64>,
    count: Option<i64>,
    max_strength: Option<i64>,
    spawn_needed: Option<i64>,
    edges: Option<Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct LoadingMap{
    global: Globals,
    planet: Vec<MapPlanet>
}

fn to_vec(p: &Ipt) -> Vec<i64>{
    vec![p.x as i64, p.y as i64]
}
#[allow(non_snake_case)]
fn to_Ipt(p: Vec<i64>) -> Ipt{
    debug_assert!(p.len() == 2);
    ipt(p[0] as i32, p[1] as i32)
}

//returns the graph
pub fn load_map(map: LoadingMap) -> WorldGraph{

    let mut g = Graph::new_undirected();
    let mut data: HashMap<String, NodeInd> = HashMap::new();
    for p in map.planet{
        let loc = to_Ipt(p.loc);
        let owner = Player::from_i64(p.owner.unwrap_or(Player::PASSIVE as i64)).unwrap();
        let max_strength = p.max_strength.unwrap_or(64) as u32;
        let spawn_needed = p.spawn_needed.unwrap_or(64) as u32;

        let mut count = [0;MAX_SIDES];
        count[owner] = p.count.unwrap_or(10) as u32;
        let node = Planet{
            loc,
            count,
            fight_progess: [0;MAX_SIDES],
            owner,
            owner_strength: max_strength,
            max_strength,
            spawn_progress: 0,
            spawn_needed,
        };
        let node_ind = g.add_node(node);
        data.insert(p.id, node_ind);
        if let Some(edges) = p.edges {
            for e in edges{
                let other = data.get(&e).unwrap();
                g.add_edge(node_ind, *other, Edge::new());
            }
        }
    }
    g
}
/*pub fn save_map(g: &Graph<Planet, Edge, Undirected>, size: &Ipt) -> LoadingMap{
    let glob = Globals{map_size: to_vec(size)};
    let mut planets = Vec::new();
    let mut i = 0;
    for node_ind in g.node_indices() {
        let node = &g[node_ind];
        let mut planet = MapPlanet {
            id: i.to_string(),
            loc: to_vec(node.loc),
            edges: Vec::new(),
            owner: Some(node.owner),
            max_strength: Some(node.max_strength),
            count: Some(node.count)
        }
    }
}*/