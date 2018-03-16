//#![allow(dead_code)]
/*extern crate rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};*/


use std::env;
use std::path;
use std::io::Read;
//use std::io::Write;
extern crate ggez;
use ggez::*;
use ggez::event::*;

mod library;
use library::*;

mod simulation;
use simulation::*;
use simulation::petgraph::prelude::*;

mod renderer;
use renderer::*;
mod interface;
use interface::*;

extern crate toml;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
struct Config{
    game: GameConfig
}
struct MainState {
    sim: Simulation,
    selected: Option<NodeInd>,
    frame: u64,
    renderer: Renderer,
    conf: Config
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sim = Simulation::new();
        let mut conf_file = ctx.filesystem.open("/conf.toml")?;
        let mut buffer = Vec::new();
        conf_file.read_to_end(&mut buffer)?;
        let conf = toml::from_slice(&buffer).unwrap();
        let renderer = Renderer::new(ctx)?;
        let s = MainState { sim, selected: None, frame: 0, renderer, conf };
        Ok(s)
    }

}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.frame += 1;
        if self.frame % 120 == 0 {
            println!("{} - FPS: {}", self.frame, timer::get_fps(_ctx));
        }
        self.sim.update(&self.conf.game);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.renderer.render(ctx, &self.sim);
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
                let next_o = check_planets(&self.sim, ipt(x, y), 32);
                if let Some(next) = next_o {
                    if next != selected {
                        if let Some((edge_ind, dir)) = self.sim.world.find_edge_undirected(selected, next){
                            let (edge, node) = self.sim.world.index_twice_mut(edge_ind, selected);
                            let transfer_amount = node.count; // TODO: make percentage or something?
                            node.count -= transfer_amount;
                            let new_follow = match dir{
                                Direction::Outgoing => ArmyGroup { direction: DIR::FORWARD, progress: 0, count: transfer_amount },
                                Direction::Incoming => ArmyGroup { direction: DIR::BACKWARD, progress: 0, count: transfer_amount }
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
            self.selected = check_planets(&self.sim, ipt(x, y), 32);
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

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}