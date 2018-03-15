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
type Ipt = na::Point2<i32>;

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
fn gpt(loc: Ipt) -> Point2{
    return Point2::new(loc.x as f32, loc.y as f32);
}
fn ipt(x: i32, y: i32) -> Ipt{
    return Ipt::new(x, y);
}


struct Planet {
    loc: Ipt,
    count: u64,
    spawn_progress: u32,
    spawn_needed: u32
}
impl Planet{
    fn new(loc: Ipt) -> Planet{
        Planet{
            loc,
            count: 0,
            spawn_progress: 0,
            spawn_needed: 64
        }
    }
}
struct Edge{
    length: i32,
    transfers: Vec<ArmyGroup>
}
impl Edge{
    fn new() -> Edge{
        Edge{transfers: Vec::new(), length: 5000}
    }
}
enum DIR{FORWARD, BACKWARD}
struct ArmyGroup{
    direction: DIR,
    progress: i32,
    count: u64
}

struct GlobalResources{
    font: Font,
    num_font: NumericFont,
    small_num_font: NumericFont
}
impl GlobalResources{
    fn new(ctx: &mut Context) -> GameResult<GlobalResources>{
        let font =  graphics::Font::new(ctx, "/Tuffy.ttf", 24)?;
        let num_font = NumericFont::new(ctx, &font)?;
        let small_font =  graphics::Font::new(ctx, "/Tuffy.ttf", 16)?;
        let small_num_font = NumericFont::new(ctx, &small_font)?;
        let g = GlobalResources { font, num_font, small_num_font};
        Ok(g)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config{
    army_speed: i32
}
struct MainState {
    selected: Option<NodeInd>,
    world: Graph<Planet, Edge, Undirected>,
    timestep: u64,
    resources: GlobalResources,
    conf: Config
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut g = Graph::new_undirected();
        let node_a = g.add_node(Planet::new(ipt(100, 100)));
        let node_b = g.add_node(Planet::new(ipt(800, 200)));
        let node_c = g.add_node(Planet::new(ipt(400, 600)));
        g.add_edge(node_a, node_b, Edge::new());
        g.add_edge(node_b, node_c, Edge::new());
        let resources = GlobalResources::new(ctx)?;
        let mut conf_file = ctx.filesystem.open("/conf.toml")?;
        let mut buffer = Vec::new();
        conf_file.read_to_end(&mut buffer)?;
        let conf = toml::from_slice(&buffer).unwrap();
        let s = MainState { world: g, selected: None, timestep: 0, resources, conf };
        Ok(s)
    }

    fn check_planets(&self, pos: Ipt) -> Option<NodeInd>{
        let mut dist = i32::max_value();
        let mut best = None;
        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            let tmpdist = dist2(&pos, &node.loc);
            if tmpdist < dist{
                best = Some(node_ind);
                dist = tmpdist;
            }
        }
        if dist < 32*32{
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
            node.spawn_progress += 1;
            if node.spawn_progress >= node.spawn_needed{
                node.spawn_progress -= node.spawn_needed;
                node.count += 1;
            }
        }
        for edge_ind in self.world.edge_indices(){
            let (s_ind, t_ind) = self.world.edge_endpoints(edge_ind).unwrap();
            let mut transfer_set: Vec<(NodeInd, u64)> = Vec::new();
            {
                let edge = &mut self.world[edge_ind];
                let edge_len = edge.length;
                for group in &mut edge.transfers {
                    group.progress += self.conf.army_speed;
                    if group.progress > edge_len {
                        let ending;
                        match group.direction {
                            DIR::FORWARD => ending = t_ind,
                            DIR::BACKWARD => ending = s_ind
                        }
                        transfer_set.push((ending, group.count));
                    }
                }
                edge.transfers.retain(|ref f| !(f.progress > edge_len));
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
        for edge_ref in self.world.edge_references(){
            let s = &self.world[edge_ref.source()];
            let t = &self.world[edge_ref.target()];
            let s_loc = gpt(s.loc);
            let t_loc = gpt(t.loc);
            graphics::line(ctx, &[s_loc, t_loc], 2.)?;
            let edge = edge_ref.weight();
            for group in &edge.transfers{
                let f_progress = match group.direction {
                    DIR::FORWARD => (group.progress as f32)/(edge.length as f32),
                    DIR::BACKWARD => 1.0-((group.progress as f32)/(edge.length as f32))
                };
                let loc = s_loc + (t_loc - s_loc) * f_progress;
                graphics::circle(ctx, DrawMode::Fill, loc, 16., 0.5)?;
                set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
                self.resources.small_num_font.draw_centered(ctx, loc, group.count as usize)?;
                set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            }
        }

        for node_ind in self.world.node_indices(){
            let node = &self.world[node_ind];
            let node_loc = gpt(node.loc);
            graphics::circle(ctx, DrawMode::Fill, node_loc, 32., 0.5)?;
            set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
            self.resources.num_font.draw_centered(ctx, node_loc, node.count as usize)?;
            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
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
                let next_o = self.check_planets(ipt(x, y));
                if let Some(next) = next_o {
                    if next != selected {
                        if let Some((edge_ind, dir)) = self.world.find_edge_undirected(selected, next){
                            let (edge, node) = self.world.index_twice_mut(edge_ind, selected);
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
            self.selected = self.check_planets(ipt(x, y));
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