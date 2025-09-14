
const MIDI_NOTE_NAME_ORDER:&[&str] = &["C", "C#", "D", "D#", "E", "F", "F#", "A", "A#", "B"];
const MIDI_NOTE_NAME_COUNT:u8 = MIDI_NOTE_NAME_ORDER.len() as u8;
const MIDI_OCT_UP_NOTE_OFFSET:u8 = 3;



pub struct Note {
	pub note:u8,
	pub octave:u8,
	pub name:String
}
impl Note {

	/// Create a new note by midi id.
	pub fn from_midi_id(id:u8) -> Note {
		let note:u8 = id % MIDI_NOTE_NAME_COUNT;
		let octave:u8 = ((id + MIDI_OCT_UP_NOTE_OFFSET) / MIDI_NOTE_NAME_COUNT) - 1;
		Note {
			note,
			octave,
			name: format!("{note}{octave}")
		}
	}

	/// Create a new note from note and octave. Combines them into a midi id then recurses into from_midi_id to make sure all values are withing valid ranges.
	pub fn new(note:u8, octave:u8) -> Note {
		Note::from_midi_id(octave * MIDI_NOTE_NAME_COUNT + note)
	}
}