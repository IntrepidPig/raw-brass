use crate::window::WindowEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct MouseMoveEvent {
	pub pos: (f64, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseClickEvent {
	pub state: PressState,
	pub button: MouseButton,
	pub pos: (f64, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PressState {
	Pressed,
	Released,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardEvent {
	pub state: PressState,
	pub keycode: winit::VirtualKeyCode,
}
