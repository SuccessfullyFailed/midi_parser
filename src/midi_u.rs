#[cfg(test)]
mod tests {
	use crate::{ Midi, FILE_HEADER_CHUNK, FILE_TRACK_HEADER };



	// Create testing midi.
	fn create_test_midi_bytes() -> Vec<u8> {
		/*
			create 3 tracks:
			|v ^|
			|v ^ v ^|
			|v ^ v ^ v ^|
		*/

		// Start with metadata.
		let mut test_midi_bytes:Vec<u8> = [
			FILE_HEADER_CHUNK.to_vec(),
			vec![0, 0], // 2-byte file format
			vec![0, 3], // 2-byte track count
			vec![0, 16] // 2-byte delta time
		].into_iter().flatten().collect();

		// Add tracks.
		for track_index in 0..3 {
			test_midi_bytes.extend_from_slice(FILE_TRACK_HEADER);
			let track_bytes:Vec<u8> = vec![

				// Set tempo.
				vec![
					0, // No delay
					0xFF, // Meta event
					0x51, // Set tempo
					3, // Data length
					0, 0, 8 // 3-byte Tempo
				],

				// Create press/release commands.
				(0..(1 + track_index) * 2).map(|command_index| 
					match command_index % 2 {

						// Press.
						0 => vec![
							(command_index + 1) * 4, // Delay
							0x09 << 4 ^ track_index, // Press on track
							6, // note id
							command_index * 12 // velocity
						],

						// Release.
						_ => vec![
							(command_index + 1) * 4, // Delay
							0x08 << 4 ^ track_index, // Release on track
							6, // note id
							command_index * 12 // velocity
						]
					}
				).flatten().collect::<Vec<u8>>(),

				// End of track.
				vec![
					0, // No delay
					0xFF, // Meta event
					0x2F, // End of track
					0 // Data length
				]
			].into_iter().flatten().collect::<Vec<u8>>();
			test_midi_bytes.extend_from_slice(&(track_bytes.len() as u32).to_be_bytes()); // 4-byte track size.
			test_midi_bytes.extend_from_slice(&track_bytes);
		}
		test_midi_bytes

	}



	#[test]
	fn test_parse_example_midi() {

		// Parse example midi.
		let midi:Midi = Midi::from_content(create_test_midi_bytes()).unwrap();

		// Compare results.
		assert_eq!(midi.delta_time, 16);
		assert_eq!(midi.tempo, 8);
		assert_eq!(midi.tracks.len(), 3);
		for track_index in 0..3 {
			assert_eq!(midi.tracks[track_index].0.len(), (track_index + 1) * 2);
			for command_index in 0..(track_index + 1) * 2 {
				assert_eq!(midi.tracks[track_index].0[command_index]._midi_channel, track_index as u8);
				assert_eq!(midi.tracks[track_index].0[command_index].delay_ticks, (command_index as u64 + 1) * 4);
				assert_eq!(midi.tracks[track_index].0[command_index].key_state.as_ref().unwrap().0, command_index % 2 == 0);
				assert_eq!(midi.tracks[track_index].0[command_index].key_state.as_ref().unwrap().2, command_index as u8 * 12);
			}
		}
	}
}