# 🎵 midi_parser

A simple yet flexible Rust crate for parsing and playing back **MIDI files** in a customizable way.

`midi_parser` lets you open a `.mid` file, inspect its contents, and execute user-defined logic as the MIDI events are played or planned.

---

## ✨ Features

- 🧩 Parse standard MIDI files into structured tracks and events  
- ⚙️ Provide your own handler functions for note on/off and timing events  
- 🪶 Lightweight and dependency-free

---

## 🚀 Example

Here’s how you might use `midi_parser` to control a virtual piano inside a game:

```rust
use crate::Midi;

let midi:Midi = Midi::from_file("assets/song.mid").expect("Could not parse midi file");

// Run with own event-handlers.
midi.run(

	// Note-down handler
	|track_index, note, velocity| {
		if track_index == 0 && *velocity > 0 {
			press_hotkey_for_note(note);
		}
	},

	// Note-up handler
	|track_index, note, _velocity| {
		if track_index == 0 {
			release_hotkey_for_note(note);
		}
	},

	// Delay handler
	|delay| {
		std::thread::sleep(delay);
	}
);
```