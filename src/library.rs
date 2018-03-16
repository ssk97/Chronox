use ggez::*;
use ggez::graphics::*;
use ggez::nalgebra as na;
use std::cmp::*;
use num::Num;

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
pub fn lendir(len: f32, dir: f32) -> Vector2
{
    let (y, x) = dir.sin_cos();
    Vector2::new(x*len, y*len)
}

pub struct NumericFont{ //Used for drawing rapidly-changing text via a pre-rendered font
    glyphs: Vec<Text>,
    widths: Vec<f32>,
    maxw: f32,
    maxh: f32
}
impl NumericFont{
    pub fn new(ctx: &mut Context, font: &Font) -> GameResult<NumericFont>{
        let mut glyphs = Vec::new();
        let mut widths = Vec::new();
        let mut maxh = 0;
        let mut maxw = 0;
        for i in 0..10{
            let glyph = Text::new(ctx, &i.to_string(), font)?;
            maxw = maxw.max(glyph.width());
            maxh = maxh.max(glyph.height());
            widths.push(glyph.width() as f32);
            glyphs.push(glyph);
        }
        Ok(NumericFont {glyphs, widths, maxw: maxw as f32, maxh: maxh as f32})
    }
    pub fn draw<N: Into<u64>>(&self, ctx: &mut Context, loc: Point2, number: N) -> GameResult<()>{
        let mut stack = Vec::new();
        let mut num = number.into() as usize;
        loop {
            let digit = num%10;
            num /= 10;
            stack.push(digit);
            if num <= 0 {break;}
        }
        let mut pos = loc;
        while let Some(digit) = stack.pop(){
            graphics::draw(ctx, & self.glyphs[digit], pos, 0.0)?;
            let width = self.widths[digit];
            pos += Vector2::new(width, 0.0);
        }
        Ok(())
    }
    pub fn draw_centered<N: Into<u64>>(&self, ctx: &mut Context, loc: Point2, number: N) -> GameResult<()>{
        let mut stack = Vec::new();
        let mut num = number.into() as usize;
        let mut total_width = 0.0;
        loop {
            let digit = num%10;
            num /= 10;
            let glyph = & self.glyphs[digit];
            let width = self.widths[digit];
            stack.push((width, glyph));
            total_width += width;
            if num <= 0 {break;}
        }
        let mut pos = loc - Vector2::new(total_width/2.0, self.maxh/2.0);
        while let Some((width, digit)) = stack.pop(){
            graphics::draw(ctx, digit, pos, 0.0)?;
            pos += Vector2::new(width, 0.0);
        }
        Ok(())
    }
    pub fn get_glyph(&self, digit: usize) -> &Text{
        &self.glyphs[digit]
    }
    pub fn get_maxw(&self) -> f32{
        self.maxw
    }
    pub fn get_maxh(&self) -> f32{
        self.maxh
    }
}