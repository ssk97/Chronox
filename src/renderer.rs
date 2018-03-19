use ggez::*;
use ggez::graphics::*;
use simulation::*;
use timeline::*;
use interface::*;
use library::*;
use std::f32::consts::PI;

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

fn set_col(ctx: &mut Context, conf: &InterfaceConfig, player: Player) -> GameResult<()>{
    let col = conf.colors[player as usize];
    set_color(ctx, Color::from_rgb_u32(col))?;
    Ok(())
}
pub struct Renderer{
    resources: GlobalResources
}
impl Renderer {
    pub fn new(ctx: &mut Context) -> GameResult<Renderer>{
        let resources = GlobalResources::new(ctx)?;
        Ok(Renderer{resources})
    }
    pub fn render(&self, ctx: &mut Context, conf: &InterfaceConfig, player: Player, timeline: &Timeline, interface: &GameInterface, dt: f32) -> GameResult<()> {
        let sim = &timeline[player];
        //transform from scrolling
        let screen = |loc| {(loc-interface.center_loc)};

        //Draw edges and army groups moving on them
        for edge_ref in sim.world.edge_references() {
            let s = &sim.world[edge_ref.source()];
            let t = &sim.world[edge_ref.target()];
            let s_loc = screen(gpt(s.loc));
            let t_loc = screen(gpt(t.loc));
            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            line(ctx, &[s_loc, t_loc], 2.)?;
            let edge = edge_ref.weight();
            for group in &edge.transfers {
                let future_progress = ((group.progress as f32)+((ARMY_SPEED as f32)*dt))/(edge.length as f32);
                let vis_progress = match group.direction {
                    DIR::FORWARD => future_progress,
                    DIR::BACKWARD => 1.0 - future_progress
                };
                let loc = s_loc + (t_loc - s_loc) * vis_progress;
                let radius = 8.+(group.count  as f32).log2();

                set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
                circle(ctx, DrawMode::Fill, loc, radius, 0.25)?;
                set_col(ctx, conf, group.player)?;
                self.resources.small_num_font.draw_centered(ctx, loc, group.count)?;
            }
        }

        //Draw planets and any armies on them
        for node_ind in sim.world.node_indices() {
            let node = &sim.world[node_ind];
            let node_loc = screen(gpt(node.loc));

            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            circle(ctx, DrawMode::Fill, node_loc, node.max_strength as f32, 0.25)?;
            set_col(ctx, conf, node.owner)?;
            circle(ctx, DrawMode::Line(5.0), node_loc, node.owner_strength as f32, 0.25)?;

            let involved = find_sides_node(node);
            if involved.len() == 1 {
                let player = involved[0];
                set_col(ctx, conf, player)?;
                self.resources.num_font.draw_centered(ctx, node_loc, node.count[player])?;
            } else if involved.len() > 1 {
                let count = involved.len() as f32;
                let angle_increment = 2.0*PI/count;
                let mut angle = PI/2.0;
                for player in involved{
                    set_col(ctx, conf, player)?;
                    let loc = screen(node_loc+lendir(16.0, angle));
                    self.resources.num_font.draw_centered(ctx, loc, node.count[player]) ?;
                    angle += angle_increment;
                }
            }
        }

        //draw selection
        if let Some(node_ind) = interface.selected{
            let node = &sim.world[node_ind];
            let node_loc = screen(gpt(node.loc));
            let radius = (node.max_strength-5) as f32;
            let mouse_pos = mouse::get_position(ctx)?;
            let offset = (mouse_pos-node_loc).normalize()*radius;

            set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
            circle(ctx, DrawMode::Line(2.0), node_loc, radius, 0.25)?;
            line(ctx, &[node_loc+offset, mouse_pos], 2.)?;
        }

        //draw UI
        {
            let width = conf.width as f32;
            let height = conf.height as f32;
            let ui_height = conf.ui_height as f32;
            let upper_edge = height-ui_height;
            set_color(ctx, Color::from_rgba(200, 200, 200, 255))?;
            rectangle(ctx, DrawMode::Fill, Rect::new(0.,upper_edge,width,height-upper_edge))?;
            set_color(ctx, Color::from_rgba(255, 255, 255, 128))?;
            line(ctx, &[pt(0.,upper_edge), pt(width,upper_edge)], 2.)?;

            set_color(ctx, Color::from_rgba(0, 128, 128, 128))?;
            let mut left_edge = timeline.left_edge as f32;
            let mut right_edge = timeline.right_edge as f32;
            let present = (timeline.present as f32)+dt;
            if present < 1000.0 {
                right_edge += 2.0*dt;
            } else {
                right_edge += 1.0*dt;
                left_edge += 1.0*dt;
            }

            for wave in &timeline.timewaves{
                let time = wave.time as f32+(dt*(wave.speed as f32));
                let x_pos = progress(time, left_edge, right_edge)*width;
                line(ctx, &[pt(x_pos, upper_edge),pt(x_pos, height)],2.)?;
            }

            for player in Player::values(){
                set_col(ctx, conf, player)?;
                let wave = timeline.player_timewaves[player];
                let time = wave.time as f32+(dt*(wave.speed as f32));
                let x_pos = progress(time, left_edge, right_edge)*width;
                line(ctx, &[pt(x_pos, upper_edge),pt(x_pos, height)],2.)?;
            }
        }

        Ok(())
    }
}