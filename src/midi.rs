use std::{ error::Error, fs::File, io::Read, time::Duration };
use crate::{ MidiParser, Note };



pub type TrackIndex = usize;
pub type Velocity = u8;



pub struct Midi {
	pub(crate) tempo:u32, // microseconds/quarter
	pub(crate) delta_time:u32, // ticks per quarter note
	pub(crate) tracks:Vec<MidiTrack>
}
impl Midi {

	/* CONSTRUCTOR METHODS */

	/// Create a new midi struct.
	pub fn new(tempo:u32, delta_time:u32, tracks:Vec<MidiTrack>) -> Midi {
		Midi {
			tempo,
			delta_time,
			tracks
		}
	}

	/// Create a new midi by reading a file.
	pub fn from_file(file:&str) -> Result<Midi, Box<dyn Error>> {
		let mut file_content:Vec<u8> = Vec::new();
		let mut file:File = File::open(file)?;
		file.read_to_end(&mut file_content)?;
		Midi::from_content(file_content)
	}

	/// Create a new midi from midi content.
	pub fn from_content(content:Vec<u8>) -> Result<Midi, Box<dyn Error>> {
		MidiParser::new(content).parse()
	}



	/* USAGE METHODS */
	
	/// Play the midi using a wave-generator.
	pub fn run<T:Fn(TrackIndex, &Note, &Velocity), U:Fn(TrackIndex, &Note, &Velocity), V:Fn(Duration)>(&self, note_down_handler:T, note_up_handler:U, delay_handler:V) {

		// Prepare timing variables.
		let tick_duration:Duration = self.tick_duration();
		let mut tick_count:u64 = 0;
		let mut valid_track_indexes:Vec<usize> = self.tracks.iter().enumerate().filter(|(_, track)| !track.0.is_empty()).map(|(index, _)| index).collect();
		let mut tick_timers:Vec<u64> = vec![0; valid_track_indexes.iter().last().map(|last_index| last_index + 1).unwrap_or_default()];
		let mut command_indexes:Vec<usize> = vec![0; self.tracks.len()];

		// Find the track with the nearest commands.
		while !valid_track_indexes.is_empty() {

			// Execute next commands in tracks with reached timer.
			if let Some(&track_index) = valid_track_indexes.iter().find(|track_index| tick_timers[**track_index] <= tick_count) {
				let tick_timer:&mut u64 = &mut tick_timers[track_index];
				let command_index:&mut usize = &mut command_indexes[track_index];

				// Execute the next command.
				while *tick_timer <= tick_count && *command_index < self.tracks[track_index].0.len() {
					match &self.tracks[track_index].0[*command_index] {
						MidiCommand::Delay(ticks) => {
							*tick_timer += *ticks;
						},
						MidiCommand::SetKeyState(note, state, velocity) => {
							if *state {
								note_down_handler(track_index, note, velocity);
							} else {
								note_up_handler(track_index, note, velocity)
							}
						}
					}
					*command_index += 1;
					if *command_index >= self.tracks[track_index].0.len() {
						valid_track_indexes.retain(|index| *index != track_index);
					}
				}
			}

			// No reached timer was found, delay until the next timer.
			else {
				let smallest_timer:u64 = valid_track_indexes.iter().map(|track_index| tick_timers[*track_index]).min().unwrap();
				let delay:u64 = smallest_timer - tick_count;
				delay_handler(tick_duration * delay as u32);
				tick_count += delay;
			}
		}
	}

	/// Get the duration of one tick given the current settings.
	fn tick_duration(&self) -> Duration {
		Duration::from_micros((self.tempo / self.delta_time) as u64)
	}
}



pub struct MidiTrack(pub(crate) Vec<MidiCommand>);



#[derive(PartialEq, Eq, Debug)]
pub(crate) enum MidiCommand {
	Delay(u64),
	SetKeyState(Note, bool, Velocity)
}