use crate::event::KeyboardEvent;
use crate::event::MouseClickEvent;
use crate::event::MouseMoveEvent;
use std::collections::VecDeque;
use std::fmt::Debug;

pub mod winit;
pub mod xcb;

pub trait WindowBackend: Sized {
	type Window;
	type Error: Debug;

	fn init() -> Result<Self, Self::Error>;

	fn create_window(&self, title: &str, dims: WindowDims) -> Result<Self::Window, Self::Error>;

	fn get_window_events(&self, window: &mut Self::Window, event_buf: &mut VecDeque<WindowEvent>);

	fn set_window_size(&self, window: &Self::Window, dims: (u32, u32));

	fn set_window_position(&self, window: &Self::Window, position: (i32, i32)) -> Result<(), Self::Error>;

	fn get_window_size(&self, window: &Self::Window) -> Result<(u32, u32), Self::Error>;

	fn is_window_open(&self, window: &Self::Window);

	fn present(&self);

	fn close(&self, window: Self::Window);
}

#[derive(Debug, Clone, Copy)]
pub struct WindowDims {
	pub x: i32,
	pub y: i32,
	pub width: u32,
	pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
	CloseRequested,
	CloseHappened,
	ResizeHappened { dims: (f64, f64) },
	MouseMove(MouseMoveEvent),
	MouseClick(MouseClickEvent),
	MouseEnter,
	MouseExit,
	Keyboard(KeyboardEvent),
	Expose,
}
