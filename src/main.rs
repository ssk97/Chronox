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
            vel: Range::new(3., 7.).ind_sample(rng),
            rot_accel: Deg(Range::new(0.5, 2.).ind_sample(rng)),
            rot_vel_max: Deg(Range::new(4., 8.).ind_sample(rng)),
            lookahead: Range::new(5., 15.).ind_sample(rng),
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
    fn new_random(loc: pt) -> Boid{
        let consts = BoidConstants::new(&mut thread_rng());
        Boid{loc, dir:Deg(0.), rot_vel:Deg(0.), stats:consts}
    }
    fn new(loc: pt, stats: BoidConstants) -> Boid{
        Boid{loc, dir:Deg(0.), rot_vel:Deg(0.), stats}
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
    length: f32,
    transfers: Vec<FollowPoint>
}
enum DIR{FORWARD, BACKWARD}
struct FollowPoint{
    direction: DIR,
    progress: f32,
    boids: Vec<Boid>
}
struct MainState {
    mouse: pt,
    mouse_drag: pt,
    world: Graph<Planet, Edge>
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mouse_temp = pt{x: 0., y: 0.};
        let mut g = Graph::new();
        g.add_node(Planet::new(pt{x: 100., y: 100.}));
        let s = MainState { world: g, mouse: mouse_temp, mouse_drag: mouse_temp};
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        println!("FPS: {}", 1_000_000_000./((timer::get_average_delta(_ctx).subsec_nanos()) as f64));
        for node_ind in self.world.node_indices(){
            let node = &mut self.world[node_ind];
            node.spawn_progress += 0.01;
            if (node.spawn_progress >= 1.){
                node.spawn_progress -= 1.;
                node.boids.push(Boid::new_random(node.loc));
            }
            for boid in &mut node.boids{
                boid.update(node.loc);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let boid_mesh = boid_mesh(ctx)?;

        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            graphics::circle(ctx, DrawMode::Fill, pt_gfx(node.loc), 64., 0.5)?;
            for boid in &node.boids{
                graphics::draw_ex(ctx, &boid_mesh,
                                  graphics::DrawParam {
                                      dest: pt_gfx(boid.loc),
                                      rotation: rads(boid.dir),
                                      ..Default::default() })?;
            }
        }
        graphics::present(ctx);
        Ok(())
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut Context,
                          _state: MouseState,
                          x: i32,
                          y: i32,
                          _xrel: i32,
                          _yrel: i32) {
        self.mouse = pt { x: x as f32, y: y as f32 };
    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut Context,
                               button: MouseButton,
                               x: i32,
                               y: i32) {
        if button == MouseButton::Left{
            self.mouse_drag = pt { x: x as f32, y: y as f32 };
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