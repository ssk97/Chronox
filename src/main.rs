#![allow(dead_code)]
/*extern crate rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};*/

//This has been replaced with a local copy to add
#[macro_use]
mod plain_enum;
//extern crate plain_enum;
extern crate num;
#[macro_use]
extern crate num_derive;
use num::FromPrimitive;

use std::env;
use std::path;
use std::io::Read;
use std::time;
use std::collections::VecDeque;
//use std::io::Write;
extern crate ggez;
use ggez::*;
use ggez::event::*;

extern crate petgraph;

mod map_loading;
mod orders;
mod networking;
use networking::*;
mod library;
use library::*;
mod simulation;
use simulation::*;
mod renderer;
use renderer::*;
mod interface;
use interface::*;
mod timeline;
use timeline::*;

extern crate toml;
extern crate bincode;
#[macro_use]
extern crate serde_derive;



#[derive(Serialize, Deserialize, Debug)]
struct Config{
    system: SystemConfig,
    interface: InterfaceConfig,
}
use std::default::Default;
impl Default for Config{
    fn default() -> Config{
        let system = SystemConfig{
            tick_time: 100,
            command_delay: 4,
            port_from: Some(40004),
            port_to: Some(40004)
        };
        let interface = InterfaceConfig{
            scroll_speed: 2.0,
            colors: vec![0x808080, 0xFF0000, 0x00FF00, 0x0000FF, 0xC0C000],
            ui_height: 100, energy_bar_height: 20,
            width: 1200, height: 700
        };
        Config{
            system, interface,
        }
    }
}
enum MenuState{
    WaitingForConnection,
    Playing,
}
struct MainState {
    timeline: Timeline,
    renderer: Renderer,
    interface: GameInterface,
    networking: Option<NetworkManager>,
    conf: Config,

    orders: CommandBuffer,
    player: Player,

    frame: u64,
    turn: u64,
    residual_update_dt: time::Duration,
    last_instant: time::Instant,
    last_turn: time::Instant,
    state: MenuState,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut conf_file = ctx.filesystem.open("/conf.toml")?;
        let mut buffer = Vec::new();
        conf_file.read_to_end(&mut buffer)?;
        let conf_data = toml::from_slice(&buffer);
        if conf_data.is_err(){
            println!("Config file failed to load, loading default.");
        }
        let conf: Config = conf_data.unwrap_or_default();

        let mut level_file = ctx.filesystem.open("/level1.toml")?;
        let mut buffer_l = Vec::new();
        level_file.read_to_end(&mut buffer_l)?;
        let level: map_loading::LoadingMap = toml::from_slice(&buffer_l).unwrap();
        let graph = map_loading::load_map(level);
        let sim = Simulation::new(graph);
        let timeline = Timeline::new(sim);
        let renderer = Renderer::new(ctx)?;
        let mut orders = VecDeque::new();
        for _ in 0..conf.system.command_delay{
            orders.push_front(Vec::new());
        }


        let args: Vec<String> = env::args().collect();
        println!("args: {:?}", args);
        let player;
        if let Some(player_str) = args.get(1) {
            let player_num = player_str.parse::<i64>().expect("Player ID (1st arg) not a number");
            player = Player::from_i64(player_num).expect("Player ID (1st arg) not 1-4");
            //Have fun with player 0!
        } else {
            player = Player::P1;
        }
        let ipaddr = args.get(2).cloned();
        let state = match ipaddr.is_some(){
            true => MenuState::WaitingForConnection,
            false => MenuState::Playing,
        };
        let networking = match ipaddr{
            Some(ip) => Some(NetworkManager::new(&ip, &conf.system)),
            None => None,
        };
        let interface = GameInterface::new();
        let s = MainState {
            timeline, renderer, interface, networking, conf, orders, player,
            frame: 0, turn: 0, residual_update_dt: time::Duration::from_secs(0),
            last_instant: time::Instant::now(), last_turn: time::Instant::now(), state
        };
        Ok(s)
    }

    fn tick(&mut self) {
        let now = time::Instant::now();
        let time_since_last = now - self.last_instant;
        self.residual_update_dt += time_since_last;
        self.last_instant = now;
        self.frame += 1;
    }
    fn turn_tick(&mut self) {
        let now = time::Instant::now();;
        self.turn += 1;
        self.last_turn = now;
    }
    fn reset_time(&mut self){
        self.last_instant = time::Instant::now();
        self.residual_update_dt = time::Duration::from_secs(0);

    }
    fn check_networking(&mut self) {
        if let Some(n) = self.networking.as_mut() {
            n.receive_commands(&mut self.orders, self.turn, &self.conf.system);
        }
    }
    fn send_commands(&mut self){
        if let Some(n) = self.networking.as_mut() {
            n.send_commands(&mut self.orders, self.turn);
        }
    }
    fn check_update(&mut self) -> bool {
        let dt = time::Duration::from_millis(self.conf.system.tick_time as u64);
        if self.residual_update_dt > dt {
            self.residual_update_dt -= dt;
            if let Some(n) = self.networking.as_mut() {
                if n.can_advance(){
                    n.advance();
                    true
                } else {
                    n.send_commands(&mut self.orders, self.turn);
                    self.last_turn = time::Instant::now();//stop dt-prediction
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }
    fn dt(&self) -> f32{
        let now = time::Instant::now();
        let dt_expected = time::Duration::from_millis(self.conf.system.tick_time as u64);
        let dt_now = now-self.last_turn;
        let dt = ((dt_now.subsec_nanos() as f64)/(dt_expected.subsec_nanos() as f64)) as f32;
        dt
    }
}


impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.state{
            MenuState::Playing => {
                self.tick();
                self.interface.update(&self.conf.interface);
                if self.frame % 120 == 0 {
                    println!("{} - FPS: {}", self.frame, timer::get_fps(ctx));
                }
                self.check_networking();
                while self.check_update() {
                    self.turn_tick();
                    self.orders.push_back(Vec::new());
                    let orders_this_turn = self.orders.pop_front().unwrap();
                    if orders_this_turn.len() > 0 {
                        println!("process command on turn {}", self.turn)
                    }
                    self.timeline.evaluate_timestep(orders_this_turn);
                    self.send_commands();
                }
            }
            MenuState::WaitingForConnection => {
                let connected = {
                    let net = &mut self.networking.as_mut().unwrap();
                    net.attempt_connect(self.player)
                };
                if connected {
                    self.state = MenuState::Playing;
                    self.reset_time();
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        self.renderer.render(ctx, &self.conf.interface, self.player, &self.timeline, &self.interface, self.dt())?;
        graphics::present(ctx);
        Ok(())
    }


    fn mouse_button_up_event(&mut self,
                             _ctx: &mut Context,
                             button: MouseButton,
                             x: i32,
                             y: i32) {
        self.interface.mouse_up(button, ipt(x, y), self.player, &self.timeline, &mut self.orders, &self.conf.interface);
    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut Context,
                               button: MouseButton,
                               x: i32,
                               y: i32) {
        self.interface.mouse_down(button, ipt(x, y), self.player, &self.timeline, &mut self.orders, &self.conf.interface);
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        state: MouseState,
        _x: i32,
        _y: i32,
        xrel: i32,
        yrel: i32, ) {
        self.interface.mouse_move(state, Vector2::new(xrel as f32, yrel as f32));
    }
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: i32, _y: i32) {
        self.interface.mouse_wheel(_y);
    }
    //event::Mod to fix unresolved reference failure in IDE
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: event::Mod, _repeat: bool) {
        if !_repeat{
            self.interface.key_down(keycode);
        }
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: event::Mod, _repeat: bool) {
        if !_repeat {
            self.interface.key_up(keycode);
        }
    }
}
pub fn main() {

    let cb = ContextBuilder::new("chronox", "knipesteven")
        .window_setup(conf::WindowSetup::default()
            .title("Chronox!")
        )
        .window_mode(conf::WindowMode::default()
            .dimensions(1200, 700)
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