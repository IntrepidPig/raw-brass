use crate::drawing::{DrawingBackend, SurfaceCreator};

use crate::drawing::FontExtents;
use crate::drawing::TextExtents;
use cairo::Context;
use cairo::FontSlant;
use cairo::FontWeight;
use cairo::Surface;

pub struct CairoBackend {
	pub ctx: Context,
	pub surface: <Self as DrawingBackend>::Surface,
}

impl From<cairo::TextExtents> for TextExtents {
	fn from(t: cairo::TextExtents) -> Self {
		TextExtents {
			x_bearing: t.x_bearing,
			y_bearing: t.y_bearing,
			width: t.width,
			height: t.height,
			x_advance: t.x_advance,
			y_advance: t.y_advance,
		}
	}
}

impl From<cairo::FontExtents> for FontExtents {
	fn from(t: cairo::FontExtents) -> Self {
		FontExtents {
			ascent: t.ascent,
			descent: t.descent,
			height: t.height,
			max_x_advance: t.max_x_advance,
			max_y_advance: t.max_y_advance,
		}
	}
}

pub struct CairoSurface(Surface);

impl CairoSurface {
	pub fn from_surface(surface: Surface) -> Self {
		CairoSurface(surface)
	}
}

impl DrawingBackend for CairoBackend {
	type Surface = CairoSurface;

	fn new(surface: Self::Surface) -> Self {
		let mut surface = surface;
		let mut cairo = CairoBackend {
			ctx: Context::new(&surface.0),
			surface,
		};
		cairo
			.ctx
			.select_font_face(".SF Compact Display", FontSlant::Normal, FontWeight::Normal);
		cairo.ctx.set_font_size(13.5);
		cairo.ctx.push_group();
		cairo
	}

	fn resize_surface(&mut self, dims: (f64, f64)) {
		// TODO: make cross platform
		log::warn!("Resized surface which only works on Xlib as of now");
		unsafe {
			cairo_sys::cairo_xlib_surface_set_size(self.surface.0.to_raw_none(), dims.0 as i32, dims.1 as i32);
		}
	}

	fn move_to(&mut self, x: f64, y: f64) {
		self.ctx.move_to(x, y);
	}

	fn line_to(&mut self, x: f64, y: f64) {
		self.ctx.line_to(x, y);
	}

	fn set_line_width(&mut self, width: f64) {
		self.ctx.set_line_width(width);
	}

	fn set_source_rgba(&mut self, r: f64, g: f64, b: f64, a: f64) {
		self.ctx.set_source_rgba(r, g, b, a);
	}

	fn get_font_extents(&self) -> FontExtents {
		self.ctx.font_extents().into()
	}

	fn get_text_extents(&self, text: &str) -> TextExtents {
		let extents = self.ctx.text_extents(text);
		extents.into()
	}

	fn draw_text(&mut self, text: &str) {
		self.ctx.show_text(text);
	}

	fn new_path(&mut self) {
		self.ctx.new_path();
	}

	fn new_sub_path(&mut self) {
		self.ctx.new_sub_path();
	}

	fn arc(&mut self, xc: f64, yc: f64, radius: f64, angle1: f64, angle2: f64) {
		self.ctx.arc(xc, yc, radius, angle1, angle2);
	}

	fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
		self.ctx.rectangle(x, y, width, height);
	}

	fn stroke(&mut self) {
		self.ctx.stroke();
	}

	fn fill(&mut self) {
		self.ctx.fill();
	}

	fn paint(&mut self) {
		self.ctx.paint();
	}

	fn clear(&mut self) {
		let old_operator = self.ctx.get_operator();
		self.ctx.set_operator(cairo::Operator::Source);
		self.ctx.paint();
		self.ctx.set_operator(old_operator);
	}

	fn present(&mut self) {
		self.ctx.pop_group_to_source();
		self.clear();
		self.surface.0.flush();
		self.ctx.push_group();
	}
}
