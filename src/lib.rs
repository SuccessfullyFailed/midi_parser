pub(crate) const FILE_HEADER_CHUNK:&[u8] = &[0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06];
pub(crate) const FILE_TRACK_HEADER:&[u8] = &[0x4D, 0x54, 0x72, 0x6B];
		
mod note;
mod midi;
mod midi_u;
mod midi_parser;

pub use note::*;
pub use midi::*;
pub(crate) use midi_parser::*;