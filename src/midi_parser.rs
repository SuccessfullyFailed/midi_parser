use crate::{ Midi, MidiCommand, MidiTrack, Note, FILE_HEADER_CHUNK, FILE_TRACK_HEADER };
use std::{ error::Error };



pub(crate) struct MidiParser {
	bytes:Vec<u8>,
	bytes_size:usize,
	cursor:usize
}
impl MidiParser {

	/* CONSTRUCTOR METHODS */

	/// Create a new midi parser from midi file content.
	pub fn new(content:Vec<u8>) -> MidiParser {
		let bytes_size:usize = content.len();
		MidiParser {
			bytes: content,
			bytes_size,
			cursor: 0
		}
	}



	/* PARSING METHODS */

	/// Parse the contents into midi.
	pub fn parse(mut self) -> Result<Midi, Box<dyn Error>> {
		const DEFAULT_TEMPO:u32 = 60;

		// Parse header.
		if &self.take(FILE_HEADER_CHUNK.len()) != FILE_HEADER_CHUNK {
			return Err("Midi contents do not start with header chunk.".into());
		}
		let _file_format:u16 = u16::from_be_bytes(self.take_c());
		let track_count:u16 = u16::from_be_bytes(self.take_c());
		let delta_time:u32 = u16::from_be_bytes(self.take_c()) as u32;
		let mut tempo:u32 = DEFAULT_TEMPO;

		// Parse tracks.
		let mut tracks:Vec<MidiTrack> = Vec::new();
		while tracks.len() < track_count as usize && self.take_conditional(FILE_TRACK_HEADER.len(), |bytes| bytes == FILE_TRACK_HEADER).is_some() {
			let track_length:u32 = u32::from_be_bytes(self.take_c());
			let cursor_end:usize = self.cursor + track_length as usize;
			// Parse track content.
			let mut continue_parsing_track:bool = true;
			let mut track_data:Vec<MidiCommand> = Vec::new();
			while continue_parsing_track && self.cursor < cursor_end {
				let delay_ticks:u64 = self.take_vl_int();

				// Parse meta event.
				if self.take_conditional(1, |bytes| bytes[0] == 0xFF).is_some() {
					let command:u8 = self.take_one();
					let data_length:u8 = self.take_one();
					let data:Vec<u8> = self.take(data_length as usize);
					self.parse_meta_event(command, data, delay_ticks, &mut track_data, &mut continue_parsing_track, &mut tempo)?;
				}

				// Parse normal event.
				else {
					let byte:u8 = self.take_one();
					let command:u8 = byte >> 4;
					let midi_channel:u8 = byte & 0b1111;
					self.parse_event(command, midi_channel, delay_ticks, &mut track_data, &mut continue_parsing_track)?;
				}
			}

			// Add track data to tracks.
			tracks.push(MidiTrack(track_data));
		}

		Ok(Midi {
			tempo,
			delta_time,
			tracks
		})
	}

	/// Parse a normal event and handle the action tied to it.
	fn parse_event(&mut self, command:u8, midi_channel:u8, delay_ticks:u64, track_data:&mut Vec<MidiCommand>, _continue_parsing_track:&mut bool) -> Result<(), Box<dyn Error>> {
		match command {
			
			// Release note
			0x08 => {
				track_data.push(MidiCommand::state_change(
					midi_channel,
					delay_ticks,
					false,
					Note::from_midi_id(self.take_one()),
					self.take_one()
				));
			},

			// Press note
			0x09 => {
				track_data.push(MidiCommand::state_change(
					midi_channel,
					delay_ticks,
					true,
					Note::from_midi_id(self.take_one()),
					self.take_one()
				));
			},

			// Todo: Key after-touch
			0x0A => {
				let _note:u8 = self.take_one();
				let _velocity:u8 = self.take_one();
			},

			// Todo: Control Change
			0x0B => {
				let _controller_index:u8 = self.take_one();
				let _value:u8 = self.take_one();
			},

			// Todo: Program (patch) change
			0x0C => {
				let _program_index:u8 = self.take_one();
			},

			// Todo: Channel after-touch
			0x0D => {
				let _channel_index:u8 = self.take_one();
			},

			// Todo: Pitch wheel change
			0x0E => {
				let _bottom:u8 = self.take_one();
				let _top:u8 = self.take_one();
			},

			// Invalid command
			_ => return Err(format!("Nonexistent event command found in midi: {:#0x}", command).into())
		}
		Ok(())
	}

	/// Parse a meta event and handle the action tied to it.
	fn parse_meta_event(&mut self, command:u8, data:Vec<u8>, _delay_ticks:u64, _track_data:&mut Vec<MidiCommand>, continue_parsing_track:&mut bool, tempo:&mut u32) -> Result<(), Box<dyn Error>> {
		match command {

			// TODO: Set sequence number
			0x00 => {},

			// TODO: Text event, sets any text
			0x01 => {},

			// TODO: Copyright info text
			0x02 => {},

			// TODO: Sequence or track name
			0x03 => {},

			// TODO: Track instrument name
			0x04 => {},

			// TODO: Lyric
			0x05 => {},

			// TODO: Marker
			0x06 => {},

			// TODO: Queue point
			0x07 => {},

			// End of track.
			0x2F => {
				*continue_parsing_track = false;
			},

			// Set tempo
			0x51 => {
				*tempo = u32::from_be_bytes([0, data[0], data[1], data[2]]);
			},

			// TODO: SMPTE Offset
			0x54 => {},

			// TODO: Time signature
			0x58 => {},

			// TODO: Key signature
			0x59 => {},

			// TODO: Sequencer specific information
			0x7F => {},

			// TODO: Timing clock used for synchronization
			0xF8 => {},

			// TODO: Start current sequence
			0xFA => {},

			// TODO: Continue a stopped sequence
			0xFB => {},

			// TODO: Stop a sequence
			0xFC => {},

			// Invalid command
			_ => eprintln!("Nonexistent meta event command found in midi: {:#0x}", command) // as Meta events et their arguments dynamically, this will not break the rest of the midi parsing
		}
		Ok(())
	}



	/* DATA TAKER METHODS */

	/// Take one byte. Increments the cursor.
	fn take_one(&mut self) -> u8 {
		self.cursor += 1;
		self.bytes[self.cursor - 1]
	}

	/// Take the set amount of bytes. Increments the cursor.
	fn take(&mut self, bytes:usize) -> Vec<u8> {
		self.cursor += bytes;
		self.bytes[self.cursor - bytes..self.cursor].to_vec()
	}

	/// Take the set amount of bytes if the condition is met. Condition function only gets the amount of bytes requested to take. Returns bytes and increments the cursor only if condition is met.
	fn take_conditional<T>(&mut self, bytes:usize, condition:T) -> Option<Vec<u8>> where T:Fn(&[u8]) -> bool {
		if self.cursor + bytes < self.bytes_size && condition(&self.bytes[self.cursor..self.cursor + bytes]) {
			Some(self.take(bytes))
		} else {
			None
		}
	}

	/// Take the set amount of bytes. Increments the cursor.
	fn take_c<const BYTES:usize>(&mut self) -> [u8; BYTES] {
		self.cursor += BYTES;
		self.bytes[self.cursor - BYTES..self.cursor].try_into().unwrap()
	}

	/// Take bytes while their first bit is 0. Increments the cursor.
	fn take_vl(&mut self) -> Vec<u8> {
		let mut more_bytes_to_follow:bool = true;
		let mut bytes:Vec<u8> = Vec::new();
		while more_bytes_to_follow {
			let byte:u8 = self.take_one();
			bytes.push(byte & 0b01111111);
			more_bytes_to_follow = byte >> 7 == 1;
		}
		bytes
	}

	/// Take bytes while their first bit is 0. Returns a u64 containing the found bytes. Increments the cursor.
	fn take_vl_int(&mut self) -> u64 {
		let mut time_bytes:Vec<u8> = self.take_vl();
		time_bytes = [vec![0; 8 - time_bytes.len()], time_bytes].into_iter().flatten().collect();
		u64::from_be_bytes(time_bytes.try_into().unwrap())
	}
}