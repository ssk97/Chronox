#![allow(dead_code)]
/*extern crate rand;
use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};*/


use std::env;
use std::path;
use std::io::Read;
use std::io::Write;

mod library;
use library::*;
use library::ggez::*;
use library::ggez::graphics::*;
use library::ggez::event::*;
use library::ggez::nalgebra as na;

extern crate petgraph;
use petgraph::prelude::*;
type NodeInd = NodeIndex<u32>;
type EdgeInd = EdgeIndex<u32>;

extern crate toml;
extern crate bincode;
#[macro_use]
extern crate serde_derive;

fn pt(x: f32, y: f32) -> Point2{
    return Point2::new(x, y);
}


struct Planet {
    loc: Point2,
    count: u64,
    spawn_progress: f32
}
impl Planet{
    fn new(loc: Point2) -> Planet{
        Planet{
            loc,
            count: 0,
            spawn_progress: 0.
        }
    }
}
struct Edge{
    //length: f32,
    transfers: Vec<ArmyGroup>
}
enum DIR{FORWARD, BACKWARD}
struct ArmyGroup{
    direction: DIR,
    progress: f32,
    count: u64
}

struct GlobalResources{
    font: Font,
    num: NumericFont
}
impl GlobalResources{
    fn new(ctx: &mut Context) -> GameResult<GlobalResources>{
        let f =  graphics::Font::new(ctx, "/Tuffy.ttf", 24)?;
        let nf = NumericFont::new(ctx, &f)?;
        let g = GlobalResources { font: f, num: nf};
        Ok(g)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config{

}
struct MainState {
    selected: Option<NodeIndex<u32>>,
    world: Graph<Planet, Edge, Undirected>,
    timestep: u64,
    resources: GlobalResources,
    conf: Config
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut g = Graph::new_undirected();
        let node_a = g.add_node(Planet::new(pt(100., 100.)));
        let node_b = g.add_node(Planet::new(pt(800., 200.)));
        let node_c = g.add_node(Planet::new(pt(400., 600.)));
        g.add_edge(node_a, node_b, Edge{transfers: Vec::new()});
        g.add_edge(node_b, node_c, Edge{transfers: Vec::new()});
        let resources = GlobalResources::new(ctx)?;
        let mut conf_file = ctx.filesystem.open("/conf.toml")?;
        let mut buffer = Vec::new();
        conf_file.read_to_end(&mut buffer)?;
        let conf = toml::from_slice(&buffer).unwrap();
        let s = MainState { world: g, selected: None, timestep: 0, resources, conf };
        Ok(s)
    }

    fn check_planets(&self, pos: Point2) -> Option<NodeIndex<u32>>{
        let mut dist = 1.0/0.0;
        let mut best = None;
        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            let tmpdist = na::distance_squared(&pos,&node.loc);
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
                node.count += 1;
            }
        }
        for edge_ind in self.world.edge_indices(){
            let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
            let mut transfer_set: Vec<(NodeInd, u64)> = Vec::new();
            {
                let edge = &mut self.world[edge_ind];
                for group in &mut edge.transfers {
                    match group.direction {
                        DIR::FORWARD => group.progress += 0.002,
                        DIR::BACKWARD => group.progress -= 0.002
                    }
                    if group.progress > 1. || group.progress < 0. {
                        let ending;
                        match group.direction {
                            DIR::FORWARD => ending = t_ind,
                            DIR::BACKWARD => ending = s_ind
                        }
                        transfer_set.push((ending, group.count));
                    }
                }
                edge.transfers.retain(|ref f| !(f.progress > 1. || f.progress < 0.));
            }
            for removal in transfer_set{
                let node = &mut self.world[removal.0];
                node.count += removal.1;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            graphics::circle(ctx, DrawMode::Fill, node.loc, 32., 0.5)?;
            set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
            self.resources.num.draw_centered(ctx, node.loc, node.count as usize)?;
            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
        }
        for edge_ref in self.world.edge_references(){
            let s = &self.world[edge_ref.source()];
            let t = &self.world[edge_ref.target()];
            graphics::line(ctx, &[s.loc, t.loc], 2.)?;
            let edge = edge_ref.weight();
            for group in &edge.transfers{
                let loc = s.loc + (t.loc - s.loc) * group.progress;
                set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
                self.resources.num.draw_centered(ctx, loc, group.count as usize)?;
                set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
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
                let next_o = self.check_planets(pt(x as f32, y as f32));
                if let Some(next) = next_o {
                    if next != selected {
                        if let Some((edge_ind, dir)) = self.world.find_edge_undirected(selected, next){
                            let (edge, node) = self.world.index_twice_mut(edge_ind, selected);
                            let transfer_amount = node.count; // TODO: make percentage or something?
                            node.count -= transfer_amount;
                            let new_follow = match dir{
                                Direction::Outgoing => ArmyGroup { direction: DIR::FORWARD, progress: 0., count: transfer_amount },
                                Direction::Incoming => ArmyGroup { direction: DIR::BACKWARD, progress: 1., count: transfer_amount }
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
            self.selected = self.check_planets(pt(x as f32, y as f32));
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