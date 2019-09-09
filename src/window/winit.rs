use crate::window::{WindowBackend, WindowDims, WindowEvent};

use crate::drawing::cairo::CairoBackend;
use crate::drawing::cairo::CairoSurface;
use crate::drawing::SurfaceCreator;
use crate::event::MouseButton;
use crate::event::MouseClickEvent;
use crate::event::MouseMoveEvent;
use crate::event::PressState;
use std::collections::VecDeque;
use winit::{Event, EventsLoop, Window};

pub struct WinitWindow {
	window: Window,
	events_loop: EventsLoop,
	last_cursor_position: (f64, f64),
}

pub struct WinitBackend;

impl WindowBackend for WinitBackend {
	type Window = WinitWindow;
	type Error = WinitBackendError;

	fn init() -> Result<Self, Self::Error> {
		Ok(Self)
	}

	fn create_window(&self, title: &str, dims: WindowDims) -> Result<Self::Window, Self::Error> {
		let events_loop = EventsLoop::new();

		let window = winit::WindowBuilder::new()
			.with_title(title)
			.build(&events_loop)
			.map_err(WinitBackendError::CreationError)?;

		Ok(WinitWindow {
			window,
			events_loop,
			last_cursor_position: (0.0, 0.0),
		})
	}

	fn get_window_events(&self, window: &mut Self::Window, event_buf: &mut VecDeque<WindowEvent>) {
		let events_loop = &mut window.events_loop;
		let last_cursor_position = &mut window.last_cursor_position;
		events_loop.poll_events(|evt| {
			if let Some(mut evt) = convert_winit_event(evt) {
				// Necessary because winit mouse click events don't contain the position of the click
				match evt {
					WindowEvent::MouseMove(ref mut mouse_move_event) => {
						*last_cursor_position = mouse_move_event.pos;
					}
					WindowEvent::MouseClick(ref mut mouse_click_event) => {
						mouse_click_event.pos = *last_cursor_position;
					}
					_ => {}
				}

				event_buf.push_back(evt);
			}
		});
	}

	fn set_window_size(&self, window: &Self::Window, dims: (u32, u32)) {
		unimplemented!()
	}

	fn set_window_position(&self, window: &Self::Window, position: (i32, i32)) -> Result<(), Self::Error> {
		window.window.set_position(winit::dpi::LogicalPosition::from_physical(
			(position.0 as i32, position.1 as i32),
			1.0,
		));
		Ok(())
	}

	fn get_window_size(&self, window: &Self::Window) -> Result<(u32, u32), Self::Error> {
		let physical = window.window.get_inner_size().unwrap().to_physical(1.0);
		Ok((physical.width.round() as u32, physical.height.round() as u32))
	}

	fn is_window_open(&self, window: &Self::Window) {
		unimplemented!()
	}

	fn present(&self) {}

	fn close(&self, window: Self::Window) {
		drop(window);
	}
}

impl SurfaceCreator<Self, CairoBackend> for WinitBackend {
	//TODO: make cross platform
	fn create_surface(&self, args: &WinitWindow) -> CairoSurface {
		use winit::os::unix::WindowExt;

		let window = &args.window;
		let dims = window.get_inner_size().unwrap().to_physical(1.0);
		let x_window = window.get_xlib_window().unwrap();
		let x_dpy = window.get_xlib_display().unwrap();
		let x_screen = window.get_xlib_screen_id().unwrap();

		let surface = unsafe {
			cairo_sys::cairo_xlib_surface_create(
				x_dpy as *mut _,
				x_window,
				x11::xlib::XDefaultVisual(x_dpy as *mut _, x_screen),
				dims.width as i32,
				dims.height as i32,
			)
		};

		unsafe { CairoSurface::from_surface(cairo::Surface::from_raw_full(surface)) }
	}
}

fn convert_winit_event(evt: winit::Event) -> Option<WindowEvent> {
	Some(match evt {
		Event::WindowEvent { event, .. } => match event {
			winit::WindowEvent::CloseRequested => WindowEvent::CloseRequested,
			winit::WindowEvent::Resized(logical_size) => {
				let physical = logical_size.to_physical(1.0);
				WindowEvent::ResizeHappened {
					dims: (physical.width, physical.height),
				}
			}
			winit::WindowEvent::MouseInput {
				device_id: _,
				state,
				button,
				modifiers,
			} => WindowEvent::MouseClick(MouseClickEvent {
				state: match state {
					winit::ElementState::Pressed => PressState::Pressed,
					winit::ElementState::Released => PressState::Released,
				},
				button: match button {
					winit::MouseButton::Left => MouseButton::Left,
					winit::MouseButton::Right => MouseButton::Right,
					winit::MouseButton::Middle => MouseButton::Middle,
					winit::MouseButton::Other(_) => return None,
				},
				pos: (0.0, 0.0),
			}),
			winit::WindowEvent::CursorMoved {
				device_id: _,
				position,
				modifiers: _,
			} => {
				let physical = position.to_physical(1.0);
				WindowEvent::MouseMove(MouseMoveEvent {
					pos: (physical.x, physical.y),
				})
			}
			evt => {
				//log::debug!("Unhandled event: {:?}", evt);
				return None;
			}
		},
		evt => {
			//log::debug!("Unhandled event: {:?}", evt);
			return None;
		}
	})
}

#[derive(Debug)]
pub enum WinitBackendError {
	CreationError(winit::CreationError),
	Unknown,
}
