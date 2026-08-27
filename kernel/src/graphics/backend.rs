use super::types::{Color, Point, Rect};

pub trait GraphicsBackend {
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn clear(&mut self, color: Color);

    fn draw_pixel(
        &mut self,
        point: Point,
        color: Color,
    );

    fn fill_rect(
        &mut self,
        rect: Rect,
        color: Color,
    );

    fn blend_pixel(
        &mut self,
        point: Point,
        color: Color,
        alpha: u8,
    );
}