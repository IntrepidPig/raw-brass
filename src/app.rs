use crate::event::MouseClickEvent;
use crate::window::WindowDims;
use crate::{
	drawing::{
		cairo::{CairoBackend, CairoSurface},
		DrawingBackend, SurfaceCreator,
	},
	window::{
		winit::{WinitBackend, WinitWindow},
		WindowBackend, WindowEvent,
	},
};
use std::collections::VecDeque;

pub struct App<W: WindowBackend, D: DrawingBackend> {
	pub window_backend: W,
	pub window: W::Window,
	pub draw_backend: D,
	evt_buf: VecDeque<WindowEvent>,
	frame_dims: (f64, f64),
	last_hovered: Option<u32>,
}

impl<W: WindowBackend, D: DrawingBackend> App<W, D>
where
	W: SurfaceCreator<W, D>,
{
	pub fn new(title: &str, dims: WindowDims) -> Self {
		let window_backend = W::init().unwrap();
		let window = window_backend.create_window(title, dims).unwrap();
		let surface = window_backend.create_surface(&window);
		let draw_backend = D::new(surface);
		let window_dims = window_backend.get_window_size(&window).unwrap();

		App {
			window_backend,
			window,
			draw_backend,
			evt_buf: VecDeque::new(),
			frame_dims: (window_dims.0 as f64, window_dims.1 as f64),
			last_hovered: None,
		}
	}

	pub fn poll_events<F: FnMut(WindowEvent)>(&mut self, mut f: F) {
		self.window_backend.get_window_events(&mut self.window, &mut self.evt_buf);
		while let Some(evt) = self.evt_buf.pop_front() {
			match evt {
				WindowEvent::ResizeHappened { dims } => {
					self.draw_backend.resize_surface(dims);
					self.frame_dims = dims;
				}
				_ => {}
			}
			f(evt)
		}
	}

	pub fn get_drawer(&mut self) -> &mut D {
		&mut self.draw_backend
	}

	pub fn present(&self) {
		self.window_backend.present();
	}

	pub fn close(self) {
		self.window_backend.close(self.window);
	}
}
