//! The simplest possible example that does something.
extern crate rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};

extern crate ggez;
use ggez::*;
use ggez::graphics::{DrawMode};
use ggez::event::*;

mod library;
use library::*;
use library::cgmath::*;

extern crate petgraph;
use petgraph::prelude::*;
type NodeInd = NodeIndex<u32>;
type EdgeInd = EdgeIndex<u32>;

use ggez::graphics::Point2 as PT;
#[allow(non_camel_case_types)]
type pt = Point2<f32>;
#[allow(non_camel_case_types)]
type deg = Deg<f32>;

fn rads(angle: deg) -> f32{
    return Rad::from(angle).0;
}
fn pt_gfx(p: pt) -> PT{
    return PT::new(p.x, p.y);
}



fn boid_mesh(ctx: &mut Context) -> GameResult<graphics::Mesh>{
    let mb = &mut graphics::MeshBuilder::new();
    let verts = [PT::new(10., 0.),
                              PT::new(-10., -5.),
                              PT::new(-5., 0.),
                              PT::new(-10., 5.),];
    mb.polygon(DrawMode::Fill, &verts);
    return mb.build(ctx);
}

#[derive(Debug)]
struct BoidConstants{
    vel: f32,
    rot_accel: deg,
    rot_vel_max: deg,
    lookahead: f32
}
impl BoidConstants{
    fn new<T: Rng>(rng: &mut T) -> BoidConstants{
        BoidConstants{
            vel: Range::new(4., 6.).ind_sample(rng),
            rot_accel: Deg(Range::new(0.5, 4.).ind_sample(rng)),
            rot_vel_max: Deg(Range::new(6., 10.).ind_sample(rng)),
            lookahead: Range::new(0., 10.).ind_sample(rng),
        }
    }
}
struct Boid{
    loc: pt,
    dir: deg,
    rot_vel: deg,
    stats: BoidConstants
}
impl Boid{
    fn new(loc: pt, stats: BoidConstants) -> Boid{
        let dir = Deg(Range::new(0., 360.).ind_sample(&mut thread_rng()));
        Boid{loc, dir, rot_vel:Deg(0.), stats}
    }
    fn new_random(loc: pt) -> Boid{
        let consts = BoidConstants::new(&mut thread_rng());
        Boid::new(loc, consts)
    }
    fn update(&mut self, target: pt){
        let ahead = self.loc + (lendir(self.stats.vel, self.dir)*self.stats.lookahead);
        let dir = pt_dir(ahead, target);
        self.rot_vel += self.stats.rot_accel*check_dir(self.dir, dir);
        self.rot_vel = bound(self.rot_vel, self.stats.rot_vel_max);
        self.dir += self.rot_vel;
        self.loc += lendir(self.stats.vel, self.dir);
    }
}


struct Planet {
    loc: pt,
    boids: Vec<Boid>,
    spawn_progress: f32
}
impl Planet{
    fn new(loc: pt) -> Planet{
        Planet{
            loc,
            boids: Vec::new(),
            spawn_progress: 0.
        }
    }
}
struct Edge{
    //length: f32,
    transfers: Vec<FollowPoint>
}
enum DIR{FORWARD, BACKWARD}
struct FollowPoint{
    direction: DIR,
    progress: f32,
    boids: Vec<Boid>
}

struct MainState {
    selected: Option<NodeIndex<u32>>,
    world: Graph<Planet, Edge, Undirected>,
    timestep: u64
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut g = Graph::new_undirected();
        let node_a = g.add_node(Planet::new(pt{x: 100., y: 100.}));
        let node_b = g.add_node(Planet::new(pt{x: 800., y: 200.}));
        let node_c = g.add_node(Planet::new(pt{x: 400., y: 600.}));
        g.add_edge(node_a, node_b, Edge{transfers: Vec::new()});
        g.add_edge(node_b, node_c, Edge{transfers: Vec::new()});
        let s = MainState { world: g, selected: None, timestep: 0};
        Ok(s)
    }

    fn check_planets(&self, pos: pt) -> Option<NodeIndex<u32>>{
        let mut dist = 1.0/0.0;
        let mut best = None;
        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            let tmpdist = pos.distance2(node.loc);
            if tmpdist < dist{
                best = Some(node_ind);
                dist = tmpdist;
            }
        }
        if dist < 32.*32.{
            return best;
        } else {
            return None;
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.timestep += 1;
        if self.timestep % 120 == 0 {
            println!("{} - FPS: {}", self.timestep, timer::get_fps(_ctx));
        }
        for node in self.world.node_weights_mut(){
            node.spawn_progress += 0.1;
            if node.spawn_progress >= 1.{
                node.spawn_progress -= 1.;
                node.boids.push(Boid::new_random(node.loc));
            }
            for boid in &mut node.boids{
                boid.update(node.loc);
            }
        }
        for edge_ind in self.world.edge_indices(){
            let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
            let (s_loc, t_loc);{
                let (s, t) = (&self.world[s_ind], &self.world[t_ind]);
                s_loc = s.loc;
                t_loc = t.loc;
            }
            let mut transfer_set: Vec<(NodeInd, usize)> = Vec::new();
            {
                let edge = &mut self.world[edge_ind];
                for (transfer_ind, follows) in edge.transfers.iter_mut().enumerate() {
                    match follows.direction {
                        DIR::FORWARD => follows.progress += 0.002,
                        DIR::BACKWARD => follows.progress -= 0.002
                    }
                    let loc = s_loc + (t_loc - s_loc) * follows.progress;
                    for boid in &mut follows.boids {
                        boid.update(loc);
                    }
                    if follows.progress > 1. || follows.progress < 0. {
                        let ending;
                        match follows.direction {
                            DIR::FORWARD => ending = t_ind,
                            DIR::BACKWARD => ending = s_ind
                        }
                        transfer_set.push((ending, transfer_ind));
                    }
                }
            }
            for removal in transfer_set{
                let (node, edge) = self.world.index_twice_mut(removal.0, edge_ind);
                node.boids.append(&mut edge.transfers[removal.1].boids);
            }
            {
                let edge = &mut self.world[edge_ind];
                edge.transfers.retain(|ref f| !(f.progress > 1. || f.progress < 0.));
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let boid_mesh = boid_mesh(ctx)?;

        for node in self.world.node_weights_mut(){
            graphics::circle(ctx, DrawMode::Fill, pt_gfx(node.loc), 32., 0.5)?;
            for boid in &node.boids{
                graphics::draw_ex(ctx, &boid_mesh,
                                  graphics::DrawParam {
                                      dest: pt_gfx(boid.loc),
                                      rotation: rads(boid.dir),
                                      ..Default::default() })?;
            }
        }
        for edge in self.world.edge_references(){
            let s = &self.world[edge.source()];
            let t = &self.world[edge.target()];
            graphics::line(ctx, &[pt_gfx(s.loc), pt_gfx(t.loc)], 2.)?;
            let edgeWeight = edge.weight();
            for follows in &edgeWeight.transfers{
                for boid in &follows.boids{
                    graphics::draw_ex(ctx, &boid_mesh,
                                      graphics::DrawParam {
                                          dest: pt_gfx(boid.loc),
                                          rotation: rads(boid.dir),
                                          ..Default::default() })?;
                }
            }
        }
        graphics::present(ctx);
        Ok(())
    }


    fn mouse_button_up_event(&mut self,
                             _ctx: &mut Context,
                             button: MouseButton,
                             x: i32,
                             y: i32) {
        if let Some(selected) = self.selected {
            if button == MouseButton::Left {
                let next_o = self.check_planets(pt { x: x as f32, y: y as f32 });
                if let Some(next) = next_o {
                    if next != selected {
                        if let Some((edge_ind, dir)) = self.world.find_edge_undirected(selected, next){
                            let (edge, node) = self.world.index_twice_mut(edge_ind, selected);
                            let boid_vec = node.boids.drain(..).collect();//TODO: change from drain all to specified amount
                            let new_follow = match dir{
                                Direction::Outgoing => FollowPoint { direction: DIR::FORWARD, progress: 0., boids: boid_vec },
                                Direction::Incoming => FollowPoint { direction: DIR::BACKWARD, progress: 1., boids: boid_vec }
                            };
                            edge.transfers.push(new_follow);
                        }
                    }
                }
            }
        }
        self.selected = None;
    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut Context,
                               button: MouseButton,
                               x: i32,
                               y: i32) {
        if button == MouseButton::Left{
            self.selected = self.check_planets(pt{x: x as f32, y: y as f32});
        }
    }
}
pub fn main() {

    let cb = ContextBuilder::new("chronox", "knipesteven")
        .window_setup(conf::WindowSetup::default()
            .title("Chronox!")
        )
        .window_mode(conf::WindowMode::default()
            .dimensions(1920, 1080)
        );
    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}