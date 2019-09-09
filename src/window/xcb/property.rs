use crate::window::xcb::{XcbBackend, XcbBackendError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XPropertyType {
	Atom,
	Latin1String,
	Utf8String,
	Cardinal,
}

impl XPropertyType {
	pub fn atom(self, backend: &XcbBackend) -> xcb::Atom {
		match self {
			XPropertyType::Atom => xcb::ATOM_ATOM,
			XPropertyType::Latin1String => xcb::ATOM_STRING,
			XPropertyType::Utf8String => backend.intern_atom("UTF8_STRING").unwrap(),
			XPropertyType::Cardinal => xcb::ATOM_CARDINAL,
		}
	}
}

pub trait XPropertyFormat: Copy {
	fn size() -> usize;
	fn format() -> u32 {
		Self::size() as u32 * 8
	}
	fn as_u8(self) -> Option<u8>;
	fn as_u16(self) -> Option<u16>;
	fn as_u32(self) -> Option<u32>;
}

impl XPropertyFormat for u8 {
	fn size() -> usize {
		std::mem::size_of::<Self>()
	}

	fn as_u8(self) -> Option<u8> {
		Some(self)
	}

	fn as_u16(self) -> Option<u16> {
		None
	}

	fn as_u32(self) -> Option<u32> {
		None
	}
}

impl XPropertyFormat for u16 {
	fn size() -> usize {
		std::mem::size_of::<Self>()
	}

	fn as_u8(self) -> Option<u8> {
		None
	}

	fn as_u16(self) -> Option<u16> {
		Some(self)
	}

	fn as_u32(self) -> Option<u32> {
		None
	}
}

impl XPropertyFormat for u32 {
	fn size() -> usize {
		std::mem::size_of::<Self>()
	}

	fn as_u8(self) -> Option<u8> {
		None
	}

	fn as_u16(self) -> Option<u16> {
		None
	}

	fn as_u32(self) -> Option<u32> {
		Some(self)
	}
}

pub trait XProperty<F: XPropertyFormat>: Sized {
	fn property_type() -> XPropertyType;

	fn from_property_reply(
		backend: &XcbBackend,
		reply: xcb::GetPropertyReply,
		target_offset: u32,
		target_length: u32,
	) -> Result<Vec<Self>, XcbBackendError>;

	fn to_property_value(backend: &XcbBackend, values: Vec<Self>) -> Result<Vec<F>, XcbBackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardinalProperty(pub u32);

impl XProperty<u32> for CardinalProperty {
	fn property_type() -> XPropertyType {
		XPropertyType::Cardinal
	}

	fn from_property_reply(
		backend: &XcbBackend,
		reply: xcb::GetPropertyReply,
		_target_offset: u32,
		_target_length: u32,
	) -> Result<Vec<Self>, XcbBackendError> {
		let value = reply.value::<u32>();
		if reply.type_() != Self::property_type().atom(backend) {
			return Err(XcbBackendError::PropertyTypeMismatch {
				expected: Self::property_type().atom(backend),
				found: reply.type_(),
			});
		}
		Ok(value.iter().map(|atom| CardinalProperty(*atom)).collect())
	}

	fn to_property_value(_backend: &XcbBackend, values: Vec<Self>) -> Result<Vec<u32>, XcbBackendError> {
		Ok(values.into_iter().map(|cardinal| cardinal.0).collect())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomProperty(pub xcb::Atom);

impl XProperty<u32> for AtomProperty {
	fn property_type() -> XPropertyType {
		XPropertyType::Atom
	}

	fn from_property_reply(
		backend: &XcbBackend,
		reply: xcb::GetPropertyReply,
		_target_offset: u32,
		_target_length: u32,
	) -> Result<Vec<Self>, XcbBackendError> {
		let value = reply.value::<u32>();
		if reply.type_() != Self::property_type().atom(backend) {
			return Err(XcbBackendError::PropertyTypeMismatch {
				expected: Self::property_type().atom(backend),
				found: reply.type_(),
			});
		}
		Ok(value.iter().map(|atom| AtomProperty(*atom)).collect())
	}

	fn to_property_value(_backend: &XcbBackend, values: Vec<Self>) -> Result<Vec<u32>, XcbBackendError> {
		Ok(values.into_iter().map(|atom| atom.0).collect())
	}
}

impl XProperty<u8> for String {
	fn property_type() -> XPropertyType {
		XPropertyType::Utf8String
	}

	fn from_property_reply(
		_backend: &XcbBackend,
		reply: xcb::GetPropertyReply,
		_target_offset: u32,
		_target_length: u32,
	) -> Result<Vec<Self>, XcbBackendError> {
		let value = reply.value::<u8>();
		log::debug!("Getting Latin1 strings with length {}", value.len());
		value
			.split(|b| *b == 0u8)
			.filter(|s| !s.is_empty())
			.map(|s| {
				String::from_utf8(s.to_owned()).map_err(|e| {
					log::error!("Error while decoding UTF8 String property: {}", e);
					XcbBackendError::PropertyEncodingError
				})
			})
			.collect()
	}

	fn to_property_value(_backend: &XcbBackend, values: Vec<Self>) -> Result<Vec<u8>, XcbBackendError> {
		let mut buf = Vec::new();
		for value in values {
			buf.extend_from_slice(value.as_bytes());
			buf.push(0u8);
		}
		Ok(buf)
	}
}

pub struct Latin1String {
	pub data: Vec<u8>,
}

impl From<Latin1String> for String {
	fn from(t: Latin1String) -> String {
		t.data.into_iter().map(|v| v as char).collect()
	}
}

impl XProperty<u8> for Latin1String {
	fn property_type() -> XPropertyType {
		XPropertyType::Latin1String
	}

	fn from_property_reply(
		_backend: &XcbBackend,
		reply: xcb::GetPropertyReply,
		_target_offset: u32,
		_target_length: u32,
	) -> Result<Vec<Self>, XcbBackendError> {
		let value = reply.value::<u8>();
		log::debug!("Getting UTF-8 strings property with length {}", value.len());
		Ok(value
			.split(|b| *b == 0u8)
			.filter(|s| !s.is_empty())
			.map(|s| Latin1String { data: s.to_owned() })
			.collect())
	}

	fn to_property_value(_backend: &XcbBackend, values: Vec<Self>) -> Result<Vec<u8>, XcbBackendError> {
		let mut buf = Vec::new();
		for mut value in values {
			buf.append(&mut value.data);
			buf.push(0u8);
		}
		Ok(buf)
	}
}
