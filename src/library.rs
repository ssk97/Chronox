use ggez::*;
use ggez::graphics::*;
pub use ggez::graphics::{Point2, Vector2};
pub use ggez::nalgebra as na;
use std::cmp::*;
use std::collections::HashMap;
use num::Num;

pub const PI:f32 = 3.14159265358979323846264338327950288419716939937510582097494459230781640628620899;

pub type Ipt = na::Point2<i32>;
pub fn pt(x: f32, y: f32) -> Point2{
    return Point2::new(x, y);
}
pub fn gpt(loc: Ipt) -> Point2{
    return Point2::new(loc.x as f32, loc.y as f32);
}
pub fn ipt(x: i32, y: i32) -> Ipt{
    return Ipt::new(x, y);
}


//returns the distance squared, generally meant for integer format
pub fn dist2<T: na::Scalar+Num>(a: &na::Point2<T>, b: &na::Point2<T>) -> T{
    let x = a.x-b.x;
    let y = a.y-b.y;
    (x*x)+(y*y)
}
pub fn pt_dir(a: &Point2, b: &Point2) -> f32{
    let v = b-a;
    v.y.atan2(v.x)
}
pub fn lendir(len: f32, dir: f32) -> Vector2
{
    let (y, x) = dir.sin_cos();
    Vector2::new(x*len, y*len)
}
pub fn progress(val: f32, lower: f32, upper: f32) -> f32{
    let amount = val-lower;
    let total = upper-lower;
    amount/total
}
pub fn lerp(val: f32, lower: f32, upper: f32) -> f32{
    let spread = upper-lower;
    (val*spread)+lower
}

struct Glyph{
    text: Text,
    width: f32,
}
pub struct PrerenderedFont{ //Used for drawing rapidly-changing text via a pre-rendered font
    glyphs: HashMap<char, Glyph>,
    maxw: f32,
    maxh: f32
}
impl PrerenderedFont{
    pub fn new(ctx: &mut Context, font: &Font, chars: &str) -> GameResult<PrerenderedFont>{
        let mut glyphs = HashMap::new();
        let mut maxh = 0;
        let mut maxw = 0;
        for c in chars.chars(){
            let text = Text::new(ctx, &(c.to_string()), font)?;
            maxw = maxw.max(text.width());
            maxh = maxh.max(text.height());
            let width = text.width() as f32;
            let glyph = Glyph{text, width};
            glyphs.insert(c, glyph);
        }
        Ok(PrerenderedFont {glyphs, maxw: maxw as f32, maxh: maxh as f32})
    }
    pub fn draw<S: Into<String>>(&self, ctx: &mut Context, loc: Point2, str_base: S) -> GameResult<()>{
        let str_data = str_base.into();
        let mut pos = loc;
        for c in str_data.chars() {
            let glyph = & self.glyphs[&c];
            graphics::draw(ctx, &glyph.text, pos, 0.0)?;
            pos += Vector2::new(glyph.width, 0.0);
        }
        Ok(())
    }
    pub fn draw_centered<S: Into<String>>(&self, ctx: &mut Context, loc: Point2, str_base: S) -> GameResult<()>{
        let mut stack = Vec::new();
        let str_data = str_base.into();
        let mut total_width = 0.0;
        for c in str_data.chars().rev() {
            let glyph = & self.glyphs[&c];
            stack.push(glyph);
            total_width += glyph.width;
        }
        let mut pos = loc - Vector2::new(total_width/2.0, self.maxh/2.0);
        while let Some(glyph) = stack.pop(){
            graphics::draw(ctx, &glyph.text, pos, 0.0)?;
            pos += Vector2::new(glyph.width, 0.0);
        }
        Ok(())
    }
    pub fn draw_centered_h<S: Into<String>>(&self, ctx: &mut Context, loc: Point2, str_base: S) -> GameResult<()>{
        self.draw_centered(ctx,loc+Vector2::new(0.0, self.maxh/2.0),str_base)
    }
    pub fn get_maxw(&self) -> f32{
        self.maxw
    }
    pub fn get_maxh(&self) -> f32{
        self.maxh
    }
}