use std::time::Duration;
use crate::Note;



pub type TrackIndex = usize;
pub type Velocity = u8;



pub struct Midi {
	pub(crate) tempo:u32, // microseconds/quarter
	pub(crate) delta_time:u32, // ticks per quarter note
	pub(crate) tracks:Vec<MidiTrack>
}
impl Midi {

	/// Create a new midi struct.
	pub fn new(tempo:u32, delta_time:u32, tracks:Vec<MidiTrack>) -> Midi {
		Midi {
			tempo,
			delta_time,
			tracks
		}
	}

	/// Play the midi using a wave-generator.
	pub fn run<T:Fn(TrackIndex, &Note, &Velocity), V:Fn(TrackIndex, Duration), U:Fn(TrackIndex, &Note, &Velocity)>(&self, note_down_handler:T, note_up_handler:U, delay_handler:V) {
		let tick_duration:Duration = self.tick_duration();
		for (track_index, track) in self.tracks.iter().enumerate() {
			for command in &track.0 {
				if command.delay_ticks != 0 {
					let sleep_time:Duration = tick_duration * command.delay_ticks as u32;
					delay_handler(track_index, sleep_time);
				}
				if let Some((note_state, note, velocity)) = &command.key_state {
					if *note_state {
						note_down_handler(track_index, note, velocity);
					} else {
						note_up_handler(track_index, note, velocity);
					}
				}
			}
		}
	}

	/// Get the duration of one tick given the current settings.
	fn tick_duration(&self) -> Duration {
		Duration::from_micros((self.tempo / self.delta_time) as u64)
	}
}



pub struct MidiTrack(pub(crate) Vec<MidiCommand>);



pub(crate) struct MidiCommand {
	pub _midi_channel:u8,
	pub delay_ticks:u64,
	pub key_state:Option<(bool, Note, Velocity)>
}
impl MidiCommand {

	/// Create a new state-change command.
	pub fn state_change(midi_channel:u8, delay_ticks:u64, state:bool, note_id:Note, velocity:Velocity) -> MidiCommand {
		MidiCommand {
			_midi_channel: midi_channel,
			delay_ticks,
			key_state: Some((state, note_id, velocity))
		}
	}
}