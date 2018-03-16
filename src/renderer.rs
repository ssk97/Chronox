use ggez::*;
use ggez::graphics::*;
use simulation::*;
use simulation::petgraph::prelude::*;
use library::*;

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

pub struct Renderer{
    resources: GlobalResources
}
impl Renderer {
    pub fn new(ctx: &mut Context) -> GameResult<Renderer>{
        let resources = GlobalResources::new(ctx)?;
        Ok(Renderer{resources})
    }
    pub fn render(&self, ctx: &mut Context, sim: &Simulation) -> GameResult<()> {
        for edge_ref in sim.world.edge_references() {
            let s = &sim.world[edge_ref.source()];
            let t = &sim.world[edge_ref.target()];
            let s_loc = gpt(s.loc);
            let t_loc = gpt(t.loc);
            graphics::line(ctx, &[s_loc, t_loc], 2.)?;
            let edge = edge_ref.weight();
            for group in &edge.transfers {
                let f_progress = match group.direction {
                    DIR::FORWARD => (group.progress as f32) / (edge.length as f32),
                    DIR::BACKWARD => 1.0 - ((group.progress as f32) / (edge.length as f32))
                };
                let loc = s_loc + (t_loc - s_loc) * f_progress;
                graphics::circle(ctx, DrawMode::Fill, loc, 16., 0.5)?;
                set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
                self.resources.small_num_font.draw_centered(ctx, loc, group.count as usize)?;
                set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            }
        }

        for node_ind in sim.world.node_indices() {
            let node = &sim.world[node_ind];
            let node_loc = gpt(node.loc);
            graphics::circle(ctx, DrawMode::Fill, node_loc, 32., 0.5)?;
            set_color(ctx, Color::from_rgba(0, 0, 0, 255))?;
            self.resources.num_font.draw_centered(ctx, node_loc, node.count as usize)?;
            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
        }
        Ok(())
    }
}