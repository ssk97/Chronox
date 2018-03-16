#![allow(dead_code)]
/*extern crate rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};*/

#[macro_use]
extern crate plain_enum;

use std::env;
use std::path;
use std::io::Read;
use std::collections::VecDeque;
use std::io::Write;
extern crate ggez;
use ggez::*;
use ggez::event::*;

mod library;
use library::*;

mod simulation;
use simulation::*;
//use simulation::petgraph::prelude::*;

mod renderer;
use renderer::*;
mod interface;
use interface::*;

extern crate toml;
#[macro_use]
extern crate serde_derive;


#[derive(Serialize, Deserialize, Debug)]
struct SystemConfig{
    tick_rate: u32,
    command_delay: usize
}
#[derive(Serialize, Deserialize, Debug)]
struct Config{
    system: SystemConfig,
    render: RenderConfig,
    game: GameConfig
}
use std::default::Default;
impl Default for Config{
    fn default() -> Config{
        let system = SystemConfig{tick_rate: 10, command_delay: 4};
        let render = RenderConfig{colors: vec![0x808080, 0xFF0000, 0x00FF00, 0x0000FF, 0xC0C000] };
        let game = GameConfig{army_speed: 100};
        Config{
            system, render, game
        }
    }
}
struct MainState {
    sim: Simulation,
    selected: Option<NodeInd>,
    frame: u64,
    renderer: Renderer,
    conf: Config,
    orders: VecDeque<Vec<Order>>
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut conf_file = ctx.filesystem.open("/conf.toml")?;
        let mut buffer = Vec::new();
        conf_file.read_to_end(&mut buffer)?;
        let conf: Config = toml::from_slice(&buffer).unwrap_or_default();
        let sim = Simulation::new();
        let renderer = Renderer::new(ctx)?;
        let mut orders = VecDeque::new();
        for _ in 0..conf.system.command_delay{
            orders.push_front(Vec::new());
        }
        let s = MainState { sim, selected: None, frame: 0, renderer, conf, orders };
        Ok(s)
    }

}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.frame += 1;
        if self.frame % 120 == 0 {
            println!("{} - FPS: {}", self.frame, timer::get_fps(ctx));
        }
        while timer::check_update_time(ctx, self.conf.system.tick_rate) {
            self.orders.push_back(Vec::new());
            self.sim.handle_orders(&self.conf.game, &(self.orders.pop_front().unwrap()));
            self.sim.update(&self.conf.game);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.renderer.render(ctx, &self.conf.render, &self.sim)?;
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
                let next_o = check_planets(&self.sim, ipt(x, y), 96);
                if let Some(next) = next_o {
                    if next != selected {
                        if self.sim.world.contains_edge(selected, next){
                            let command = TransportCommand{from: selected, to: next, percent: 50};
                            let order = Order{player: Player::P1, command: CommandEnum::Transport(command)};
                            self.orders.back_mut().unwrap().push(order);
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
            self.selected = check_planets(&self.sim, ipt(x, y), 96);
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