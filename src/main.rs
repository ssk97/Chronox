//! The simplest possible example that does something.

extern crate ggez;
use ggez::*;
use ggez::graphics::{DrawMode, Point2};
mod library;
use library::*;
use library::cgmath::*;

struct globals{
    mouse_loc: Point2
}
//static globs: globals;

struct boidConstants{
    vel: f32,
    rotAccel: Deg<f32>,
    rotVelMax: Deg<f32>,
    lookahead: f32
}
struct boid{
    loc: library::cgmath::Point2<f32>,
    rot: Deg<f32>,
    rotVel: Deg<f32>,
    consts: boidConstants
}
impl boid{
    fn update(&mut self, target: Point2){
        let angle: Deg<f32> = pt_dir(self.loc, self.loc);
        self.loc += lendir(self.consts.vel, angle);
    }
}

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { pos_x: 0.0 };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::circle(ctx,
                         DrawMode::Fill,
                         Point2::new(self.pos_x, 380.0),
                         100.0,
                         2.0)?;
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}