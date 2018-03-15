pub extern crate ggez;
use self::ggez::*;
use self::ggez::graphics::*;
use std::cmp::*;

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
    pub fn draw(&self, ctx: &mut Context, loc: Point2, number: usize) -> GameResult<()>{
        let mut stack = Vec::new();
        let mut num = number;
        while num > 0{
            let digit = num%10;
            num /= 10;
            stack.push(digit);
        }
        let mut pos = loc;
        while let Some(digit) = stack.pop(){
            graphics::draw(ctx, & self.glyphs[digit], pos, 0.0)?;
            let width = self.widths[digit];
            pos += Vector2::new(width, 0.0);
        }
        Ok(())
    }
    pub fn draw_centered(&self, ctx: &mut Context, loc: Point2, number: usize) -> GameResult<()>{
        let mut stack = Vec::new();
        let mut num = number;
        let mut total_width = 0.0;
        while num > 0{
            let digit = num%10;
            num /= 10;
            let glyph = & self.glyphs[digit];
            let width = self.widths[digit];
            stack.push((width, glyph));
            total_width += width;
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