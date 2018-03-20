use ggez::*;
use ggez::graphics::*;
use simulation::*;
use timeline::*;
use interface::*;
use library::*;
use std::f32::consts::PI;

struct GlobalResources{
    font: Font,
    num_font: PrerenderedFont,
    small_num_font: PrerenderedFont
}
impl GlobalResources{
    fn new(ctx: &mut Context) -> GameResult<GlobalResources>{
        let font =  graphics::Font::new(ctx, "/Tuffy.ttf", 24)?;
        let num_font = PrerenderedFont::new(ctx, &font, "0123456789")?;
        let small_font =  graphics::Font::new(ctx, "/Tuffy.ttf", 16)?;
        let small_num_font = PrerenderedFont::new(ctx, &small_font, "0123456789:")?;
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
    pub fn render(&self, ctx: &mut Context, conf: &InterfaceConfig, viewing_player: Player, timeline: &Timeline, interface: &GameInterface, dt: f32) -> GameResult<()> {
        let sim = &timeline[viewing_player];
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
                self.resources.small_num_font.draw_centered(ctx, loc, group.count.to_string())?;
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
                self.resources.num_font.draw_centered(ctx, node_loc, node.count[player].to_string())?;
            } else if involved.len() > 1 {
                let count = involved.len() as f32;
                let angle_increment = 2.0*PI/count;
                let mut angle = PI/2.0;
                for player in involved{
                    set_col(ctx, conf, player)?;
                    let loc = node_loc+lendir(16.0, angle);
                    self.resources.num_font.draw_centered(ctx, loc, node.count[player].to_string()) ?;
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
            let energy_bar_height = conf.energy_bar_height as f32;
            let upper_edge_bar = upper_edge-energy_bar_height;
            set_color(ctx, Color::from_rgba(200, 200, 200, 255))?;
            rectangle(ctx, DrawMode::Fill, Rect::new(0.,upper_edge_bar,width,height-upper_edge_bar))?;

            //calculate edge time/position values
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
            //time ticks (every 5 seconds)
            set_color(ctx, Color::from_rgba(0, 0, 0, 128))?;
            const TICK_SIZE: f32 = 50.;
            let mut time_ticker = (left_edge/TICK_SIZE).round()*TICK_SIZE;
            let ticker_height = ui_height*0.05;
            while time_ticker <= right_edge {
                let x_pos = progress(time_ticker, left_edge, right_edge) * width;
                const MULT_ARR: [u8; 12] = [10, 1, 2, 3, 2, 1, 5, 1, 2, 3, 2, 1];
                let mult_index = ((time_ticker / TICK_SIZE).round() as usize) % (MULT_ARR.len());
                let multiplier = MULT_ARR[mult_index] as f32;
                line(ctx, &[pt(x_pos, upper_edge), pt(x_pos, upper_edge + ticker_height * multiplier)], 2.)?;
                if multiplier > 2. {
                    let time = time_ticker/600.;
                    let minutes = time.trunc();
                    let seconds = (time.fract()*60.).trunc();
                    let loc = pt(x_pos, upper_edge+ticker_height*multiplier);
                    self.resources.small_num_font.draw_centered_h(ctx, loc, format!("{}:{:02}",minutes,seconds))?;
                }
                time_ticker += TICK_SIZE;
            }
            //metadata graphs - line graph of living
            let mut line_data = Player::map_from_fn(|_|Vec::new());
            let mut largest_living = 0.0;
            for i in 0..(conf.width){
                let percent = (i as f32)/width;
                let time = lerp(percent, left_edge, right_edge);
                let metadata_l = timeline.get_metadata(time.floor() as ChronalTime);
                let metadata_r = timeline.get_metadata(time.ceil() as ChronalTime);
                for player in Player::values() {
                    let living_l = metadata_l.total_living[player] as f32;
                    let living_r = metadata_r.total_living[player] as f32;
                    let living = lerp(time.fract(),living_l,living_r);
                    if living > largest_living{
                        largest_living = living;
                    }
                    line_data[player].push(living);
                }
            }
            let y_mult = ui_height/(largest_living as f32);
            for player in Player::values(){
                let data = &line_data[player];
                set_col(ctx, conf, player)?;
                let pt_func = |(index, value)|pt(index as f32, height-(value*y_mult));
                let line_graph = data.iter().enumerate().map(pt_func).collect::<Vec<Point2>>();
                line(ctx, line_graph.as_slice(),2.)?;
            }
            //timewaves (normal)
            set_color(ctx, Color::from_rgba(0, 0, 0, 128))?;
            for wave in &timeline.timewaves{
                let time = wave.time as f32+(dt*(wave.speed as f32));
                let x_pos = progress(time, left_edge, right_edge)*width;
                line(ctx, &[pt(x_pos, upper_edge),pt(x_pos, height)],2.)?;
            }
            //timewaves (player)
            for player in Player::values(){
                set_col(ctx, conf, player)?;
                let wave = timeline.player_timewaves[player];
                let time = wave.time as f32+(dt*(wave.speed as f32));
                let x_pos = progress(time, left_edge, right_edge)*width;
                line(ctx, &[pt(x_pos, upper_edge),pt(x_pos, height)],2.)?;
            }
            //chronoenergy display
            let energy = timeline.chrono_energy[viewing_player];
            let limit = timeline.chrono_energy_limit(energy);
            let x_limit = progress(limit as f32, left_edge, right_edge)*width;
            set_color(ctx, Color::from_rgba(255, 255, 0, 64))?;
            rectangle(ctx, DrawMode::Fill, Rect::new(0.,upper_edge, x_limit, ui_height))?;//not enough energy area

            let storage_x = ((energy as f32)/(MAX_CHRONOENERGY as f32))*width;
            set_color(ctx, Color::from_rgba(255, 255, 0, 255))?;
            rectangle(ctx, DrawMode::Fill, Rect::new(0.,upper_edge_bar, storage_x, energy_bar_height))?;

            let action_cost = timeline.chrono_cost(timeline.player_timewaves[viewing_player].time);
            if energy > action_cost {
                let per_action_x = (((energy-action_cost) as f32) / (MAX_CHRONOENERGY as f32)) * width;
                set_color(ctx, Color::from_rgba(255, 128, 0, 255))?;
                rectangle(ctx, DrawMode::Fill, Rect::new(0.,upper_edge_bar, per_action_x, energy_bar_height))?;
            }

        }

        Ok(())
    }
}