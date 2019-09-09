#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigValue {
	X(i32),
	Y(i32),
	Width(u32),
	Height(u32),
	BorderWidth(u32),
}

impl ConfigValue {
	pub fn as_key(self) -> u16 {
		match self {
			ConfigValue::X(_) => xcb::CONFIG_WINDOW_X as u16,
			ConfigValue::Y(_) => xcb::CONFIG_WINDOW_Y as u16,
			ConfigValue::Width(_) => xcb::CONFIG_WINDOW_WIDTH as u16,
			ConfigValue::Height(_) => xcb::CONFIG_WINDOW_HEIGHT as u16,
			ConfigValue::BorderWidth(_) => xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
		}
	}

	pub fn as_value(self) -> u32 {
		match self {
			ConfigValue::X(x) => x as u32,
			ConfigValue::Y(y) => y as u32,
			ConfigValue::Width(width) => width as u32,
			ConfigValue::Height(height) => height as u32,
			ConfigValue::BorderWidth(border_width) => border_width as u32,
		}
	}
}
