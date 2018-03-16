use simulation::*;
use library::*;

pub fn check_planets(sim: &Simulation, pos: Ipt, max_dist: i32) -> Option<NodeInd>{
    let mut dist = i32::max_value();
    let mut best = None;
    for node_ind in sim.world.node_indices(){
        let node = &sim.world[node_ind];
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