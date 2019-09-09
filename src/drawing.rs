use crate::window::WindowBackend;

pub mod cairo;

pub trait SurfaceCreator<W: WindowBackend, D: DrawingBackend> {
	fn create_surface(&self, args: &W::Window) -> D::Surface;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextExtents {
	pub x_bearing: f64,
	pub y_bearing: f64,
	pub width: f64,
	pub height: f64,
	pub x_advance: f64,
	pub y_advance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontExtents {
	pub ascent: f64,
	pub descent: f64,
	pub height: f64,
	pub max_x_advance: f64,
	pub max_y_advance: f64,
}

pub trait DrawingBackend: Sized + 'static {
	type Surface;

	fn new(surface: Self::Surface) -> Self;

	fn resize_surface(&mut self, dims: (f64, f64));

	fn move_to(&mut self, x: f64, y: f64);

	fn line_to(&mut self, x: f64, y: f64);

	fn set_line_width(&mut self, width: f64);

	fn set_source_rgba(&mut self, r: f64, g: f64, b: f64, a: f64);

	fn get_font_extents(&self) -> FontExtents;

	fn get_text_extents(&self, text: &str) -> TextExtents;

	fn draw_text(&mut self, text: &str);

	fn new_path(&mut self);

	fn new_sub_path(&mut self);

	fn arc(&mut self, xc: f64, yc: f64, radius: f64, angle1: f64, angle2: f64);

	fn rect(&mut self, x: f64, y: f64, width: f64, height: f64);

	fn stroke(&mut self);

	fn fill(&mut self);

	fn paint(&mut self);

	fn clear(&mut self);

	fn present(&mut self);
}
