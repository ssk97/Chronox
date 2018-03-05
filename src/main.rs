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
    let verts = [PT::new(20., 0.),
                              PT::new(-20., -5.),
                              PT::new(-10., 0.),
                              PT::new(-20., 5.),];
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

struct MainState {
    mouse: pt,
    boids: Vec<Boid>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let consts1 = BoidConstants{vel: 5., rot_accel: Deg(1.), rot_vel_max: Deg(10.), lookahead: 5.};
        let boid1 = Boid::new(pt{x: 100., y: 100.}, consts1);
        let consts2 = BoidConstants{vel: 4., rot_accel: Deg(0.5), rot_vel_max: Deg(5.), lookahead: 15.};
        let boid2 = Boid::new(pt{x: 100., y: 200.}, consts2);
        let mouse_temp = pt{x: 0., y: 0.};
        let s = MainState { boids: vec![boid1, boid2], mouse: mouse_temp};
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        for boid in &mut self.boids {
            boid.update(self.mouse);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let boid_mesh = boid_mesh(ctx)?;
        for boid in &self.boids {
            graphics::draw_ex(ctx, &boid_mesh,
                              graphics::DrawParam {
                                  dest: pt_gfx(boid.loc),
                                  rotation: rads(boid.dir),
                                  ..Default::default() })?;
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
            let consts = BoidConstants::new(&mut thread_rng());
            println!("new: {:?}", &consts);
            let boid = Boid::new(pt{x: x as f32, y: y as f32}, consts);
            self.boids.push(boid);
        }

    }
}
pub fn main() {

    let mut cb = ContextBuilder::new("chronox", "knipesteven")
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