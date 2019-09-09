use crate::drawing::cairo::CairoBackend;
use crate::drawing::cairo::CairoSurface;
use crate::drawing::{DrawingBackend, SurfaceCreator};
use crate::event::MouseButton;
use crate::event::MouseClickEvent;
use crate::event::MouseMoveEvent;
use crate::event::PressState;
use crate::window::xcb::config::*;
use crate::window::xcb::property::*;
use crate::window::{WindowBackend, WindowDims, WindowEvent};

use std::collections::VecDeque;
use std::sync::Arc;

pub mod config;
pub mod property;

pub struct XcbBackend {
	conn: Arc<xcb::Connection>,
	screen: xcb::Screen<'static>,
	wm_delete_window_atom: xcb::Atom,
	visual_type: xcb::Visualtype,
}

impl XcbBackend {
	pub fn get_screen(&self) -> &xcb::Screen {
		unsafe { std::mem::transmute(&self.screen) }
	}

	pub fn intern_atom(&self, name: &str) -> Result<xcb::Atom, XcbBackendError> {
		let atom: xcb::Atom = xcb::intern_atom(self.conn.as_ref(), false, name)
			.get_reply()
			.map_err(|_| XcbBackendError::InternAtomFailed)?
			.atom();
		Ok(atom)
	}

	pub fn get_property<F: XPropertyFormat, T: XProperty<F>>(
		&self,
		window: xcb::Window,
		property: xcb::Atom,
		property_type: xcb::Atom,
		offset: u32,
		length: u32,
	) -> Result<Vec<T>, XcbBackendError> {
		let property_reply = xcb::get_property(self.conn.as_ref(), false, window, property, property_type, offset, length)
			.get_reply()
			.map_err(|_| XcbBackendError::Unknown)?;

		log::trace!("Target type: {}, got type: {}", property_type, property_reply.type_());

		let prop = T::from_property_reply(self, property_reply, offset, length)?;
		Ok(prop)
	}

	pub fn set_property<F: XPropertyFormat, T: XProperty<F>>(
		&self,
		window: xcb::Window,
		property: xcb::Atom,
		values: Vec<T>,
	) -> Result<(), XcbBackendError> {
		let value = T::to_property_value(self, values)?;
		xcb::change_property(
			self.conn.as_ref(),
			xcb::PROP_MODE_REPLACE as u8,
			window,
			property,
			T::property_type().atom(self),
			F::format() as u8,
			&value,
		);
		Ok(())
	}

	pub fn create_window(&self, dims: WindowDims) -> Result<xcb::Window, XcbBackendError> {
		let conn = self.conn.as_ref();
		let wid = conn.generate_id();
		let screen = self.get_screen();

		let colormap = if screen.root_depth() == 32 {
			screen.default_colormap()
		} else {
			let id = self.conn.generate_id();
			let cookie = xcb::create_colormap_checked(
				self.conn.as_ref(),
				xcb::COLORMAP_ALLOC_NONE as u8,
				id,
				self.get_screen().root(),
				self.visual_type.visual_id(),
			);
			cookie.request_check().map_err(|e| {
				log::error!("Failed to create custom colormap: {}", e);
				XcbBackendError::Unknown
			})?;
			id
		};

		let values: &[_] = &[
			(xcb::CW_BACK_PIXEL, screen.black_pixel()),
			(xcb::CW_BORDER_PIXEL, screen.black_pixel()),
			(xcb::CW_COLORMAP, colormap),
			(
				xcb::CW_EVENT_MASK,
				xcb::EVENT_MASK_EXPOSURE
					| xcb::EVENT_MASK_BUTTON_PRESS
					| xcb::EVENT_MASK_BUTTON_RELEASE
					| xcb::EVENT_MASK_STRUCTURE_NOTIFY,
			),
			//(xcb::CW_OVERRIDE_REDIRECT, 1),
		];
		xcb::create_window_checked(
			conn,
			32,
			wid,
			screen.root(),
			dims.x as i16,
			dims.y as i16,
			dims.width as u16,
			dims.height as u16,
			0,
			xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
			self.visual_type.visual_id(),
			&values,
		)
		.request_check()
		.map_err(|e| {
			log::error!("Failed to create window: {}", e);
			XcbBackendError::Unknown
		})?;

		// Set the WM_PROTOCOLS property of the window to type ATOM with value of the "WM_DELETE_WINDOW" atom, allowing the window
		// to receive client messages indicating closing.
		let wm_protocols_atom: xcb::Atom = self.intern_atom("WM_PROTOCOLS")?;
		self.set_property(wid, wm_protocols_atom, vec![AtomProperty(self.wm_delete_window_atom)])?;

		Ok(wid)
	}

	pub fn configure_window(&self, window: xcb::Window, args: &[ConfigValue]) -> Result<(), XcbBackendError> {
		let xcb_config_values = args.iter().map(|c| (c.as_key(), c.as_value())).collect::<Vec<_>>();
		let cookie = xcb::configure_window(self.conn.as_ref(), window, &xcb_config_values);
		cookie.request_check().map_err(|e| {
			log::error!("Failed to configure XCB window: {}", e);
			XcbBackendError::Unknown
		})?;
		Ok(())
	}

	pub fn map_window(&self, window: xcb::Window) -> Result<(), XcbBackendError> {
		xcb::map_window(self.conn.as_ref(), window).request_check().map_err(|e| {
			log::error!("Failed to map XCB window: {}", e);
			XcbBackendError::Unknown
		})
	}
}

#[test]
fn window_prop_test() {
	fn inner(backend: &XcbBackend, window: xcb::Window) {
		let tree_cookie = xcb::query_tree(backend.conn.as_ref(), window);
		let tree_reply = tree_cookie.get_reply().unwrap();
		let children = tree_reply.children();
		let window_type_atom = backend.intern_atom("_NET_WM_WINDOW_TYPE").unwrap();
		let window_class_atom = backend.intern_atom("WM_CLASS").unwrap();

		for child in children {
			println!("window id: 0x{:x}, parent id: 0x{:x}", *child, window);
			let window_class = backend.get_property::<_, String>(*child, window_class_atom, xcb::ATOM_STRING, 0, 5000);
			println!("Window class: {:?}", window_class);
			let window_type = backend
				.get_property::<_, AtomProperty>(*child, window_type_atom, xcb::ATOM_ATOM, 0, 1)
				.map(|v| v.into_iter().map(|a| a.0).collect::<Vec<_>>());
			println!("Window type: {:?}", window_type);

			let window_class = window_class.unwrap();
			if window_class.get(0).map(|s| s.as_str()) == Some("kitty") {
				let custom_atom_atom = backend.intern_atom("CUSTOM_KITTY_ATOM_BRASS").unwrap();
				let custom_utf8_atom = backend.intern_atom("CUSTOM_UTF8_ATOM_BRASS").unwrap();
				backend
					.set_property::<_, AtomProperty>(
						*child,
						custom_atom_atom,
						vec![AtomProperty(xcb::ATOM_ATOM), AtomProperty(xcb::ATOM_STRING)],
					)
					.unwrap();
				backend
					.set_property::<_, String>(*child, custom_utf8_atom, vec![String::from("nice"), String::from("meme")])
					.unwrap();
			}

			inner(backend, *child);
		}
	}
	let backend = XcbBackend::init().unwrap();
	let root = backend.screen.root();

	let window_type_atom = backend.intern_atom("_NET_WM_WINDOW_TYPE").unwrap();
	let window_class_atom = backend.intern_atom("WM_CLASS").unwrap();

	println!("root id: 0x{:x}", root);
	let window_class = backend.get_property::<_, String>(root, window_class_atom, xcb::ATOM_STRING, 0, 5000);
	println!("Window class: {:?}", window_class);
	let window_type = backend.get_property::<_, AtomProperty>(root, window_type_atom, xcb::ATOM_ATOM, 0, 1);
	println!("Window type: {:?}", window_type);

	inner(&backend, root);
}

pub struct XcbWindow {
	pub window: xcb::Window,
}

impl WindowBackend for XcbBackend {
	type Window = XcbWindow;
	type Error = XcbBackendError;

	fn init() -> Result<Self, Self::Error> {
		let (conn, screen_idx) = xcb::Connection::connect(None).map_err(|_| XcbBackendError::ConnectionFailed)?;
		let screen: xcb::Screen<'static> =
			unsafe { std::mem::transmute(conn.get_setup().roots().nth(screen_idx as usize).unwrap()) };
		// Atom referring to string "WM_DELETE_WINDOW"
		let wm_delete_window_atom: xcb::Atom = xcb::intern_atom(&conn, false, "WM_DELETE_WINDOW").get_reply().unwrap().atom();

		let mut visual_type = None;
		'outer: for depth in screen.allowed_depths() {
			if depth.depth() != 32 {
				continue;
			}

			for test_visual_type in depth.visuals() {
				visual_type = Some(test_visual_type);
				break 'outer;
			}
		}
		let visual_type = visual_type.unwrap();

		Ok(Self {
			conn: Arc::new(conn),
			screen,
			wm_delete_window_atom,
			visual_type,
		})
	}

	fn create_window(&self, title: &str, dims: WindowDims) -> Result<Self::Window, Self::Error> {
		let window = XcbBackend::create_window(self, dims)?;

		self.map_window(window)?;

		log::info!("Created and mapped window successfully");

		Ok(XcbWindow { window })
	}

	fn get_window_events(&self, window: &mut Self::Window, event_buf: &mut VecDeque<WindowEvent>) {
		self.conn.flush();
		while let Some(event) = self.conn.poll_for_event() {
			let translated_e = match event.response_type() & !0x80 {
				xcb::BUTTON_PRESS => {
					let button_event = unsafe { xcb::cast_event::<xcb::ButtonPressEvent>(&event) };
					Some(WindowEvent::MouseClick(MouseClickEvent {
						state: PressState::Pressed,
						button: {
							log::debug!("Got button {}", button_event.detail());
							match button_event.detail() {
								_ => {}
							};
							MouseButton::Left
						},
						pos: (button_event.event_x() as f64, button_event.event_y() as f64),
					}))
				}
				xcb::BUTTON_RELEASE => {
					let button_event = unsafe { xcb::cast_event::<xcb::ButtonPressEvent>(&event) };
					Some(WindowEvent::MouseClick(MouseClickEvent {
						state: PressState::Released,
						button: {
							log::debug!("Got button {}", button_event.detail());
							match button_event.detail() {
								_ => {}
							};
							MouseButton::Left
						},
						pos: (button_event.event_x() as f64, button_event.event_y() as f64),
					}))
				}
				xcb::EXPOSE => Some(WindowEvent::Expose),
				xcb::DESTROY_NOTIFY => Some(WindowEvent::CloseHappened),
				xcb::CLIENT_MESSAGE => {
					log::debug!("Got client message");
					let client_message_event = unsafe { xcb::cast_event::<xcb::ClientMessageEvent>(&event) };
					if client_message_event.data().data32()[0] == self.wm_delete_window_atom {
						Some(WindowEvent::CloseRequested)
					} else {
						log::warn!("Got unknown client message");
						None
					}
				}
				event => {
					log::debug!("Got unhandled event of type {}", event);
					None
				}
			};
			if let Some(e) = translated_e {
				event_buf.push_back(e);
			}
		}
		self.conn.flush();
	}

	fn set_window_size(&self, window: &Self::Window, dims: (u32, u32)) {
		log::error!("Attempted to set window size but the operation is unsupported");
	}

	fn set_window_position(&self, window: &Self::Window, position: (i32, i32)) -> Result<(), Self::Error> {
		let cookie = xcb::configure_window(
			self.conn.as_ref(),
			window.window,
			&[
				(xcb::CONFIG_WINDOW_X as u16, position.0 as u32),
				(xcb::CONFIG_WINDOW_Y as u16, position.1 as u32),
			],
		);
		let reply = cookie.request_check();
		reply.map_err(|_| {
			log::error!("Failed to set window position");
			XcbBackendError::Unknown
		})?;
		Ok(())
	}

	fn get_window_size(&self, window: &Self::Window) -> Result<(u32, u32), Self::Error> {
		let geometry = xcb::get_geometry(self.conn.as_ref(), window.window).get_reply().unwrap();
		Ok((geometry.width() as u32, geometry.height() as u32))
	}

	fn is_window_open(&self, window: &Self::Window) {
		unimplemented!()
	}

	fn present(&self) {
		self.conn.flush();
	}

	fn close(&self, window: Self::Window) {
		xcb::destroy_window_checked(self.conn.as_ref(), window.window);
	}
}

impl SurfaceCreator<Self, CairoBackend> for XcbBackend {
	fn create_surface(&self, args: &<XcbBackend as WindowBackend>::Window) -> <CairoBackend as DrawingBackend>::Surface {
		let dims = self.get_window_size(args).unwrap();
		log::trace!("Creating surface with dims {}x{}", dims.0, dims.1);
		unsafe {
			/* let screen = self.get_screen();
			let mut visual_type = None;

			// Gets the visual_type of the root window
			for depth in screen.allowed_depths() {
				for visual in depth.visuals() {
					if visual.visual_id() == screen.root_visual() {
						visual_type = Some(visual)
					}
				}
			}

			let mut visual_type = Box::leak(Box::new(visual_type.unwrap())); */

			// TODO: don't leak...????
			let visual_type = Box::leak(Box::new(self.visual_type));

			let cairo_xcb_connection = cairo::XCBConnection::from_raw_none(self.conn.get_raw_conn() as *mut _);
			let cairo_drawable = cairo::XCBDrawable(args.window);
			let cairo_xcb_visualtype = cairo::XCBVisualType::from_raw_none(&mut visual_type.base as *mut _ as *mut _);

			let cairo_xcb_surface = cairo::XCBSurface::create(
				&cairo_xcb_connection,
				&cairo_drawable,
				&cairo_xcb_visualtype,
				dims.0 as i32,
				dims.1 as i32,
			);

			let surface = CairoSurface::from_surface(cairo::Surface::from_raw_none(cairo_xcb_surface.to_raw_none()));

			std::mem::forget(cairo_xcb_surface);
			std::mem::forget(cairo_xcb_visualtype);
			std::mem::forget(cairo_xcb_connection);
			std::mem::forget(cairo_drawable);

			surface
		}
	}
}

#[derive(Debug, Clone)]
pub enum XcbBackendError {
	ConnectionFailed,
	InternAtomFailed,
	PropertyTypeMismatch { expected: xcb::Atom, found: xcb::Atom },
	PropertyEncodingError,
	Other(String),
	Unknown,
}
