use std::{io, error, rand};
use std::rand::distributions::{Range, IndependentSample};
type MidiTrack = Vec<u8>;

#[deriving(Show)]
enum MidiError {
    InvalidFile(String),
    InvalidTrackNumber(int),
    IoError(io::IoError),
    UnknownError,
}

impl error::FromError<io::IoError> for MidiError {
    fn from_error(err: io::IoError) -> MidiError {
        IoError(err)
    }
}

impl error::Error for MidiError {
    fn description(&self) -> &str {
        match *self {
            InvalidFile(..) => "invalid MIDI file",
            InvalidTrackNumber(..) => "invalid track number",
            IoError(..) => "underlying IO error",
            UnknownError => "unknown error",
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            InvalidFile(ref t) => Some(t.clone()),
            InvalidTrackNumber(nr) => Some(format!("The file has no track {}", nr)),
            _ => None,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            IoError(ref e) => Some(e as &error::Error),
            _ => None,
        }
    }
}


fn extract_varlen(input: &mut Reader) -> io::IoResult<uint> {
    //! Takes a reader and reads a MIDI varlen field. Advances the Reader
    //! by the amount of bytes read and returns the read uint.
    let mut nums: Vec<u8> = Vec::new();
    // For example, the 3 bytes
    //   0b11000000 0b10100000 0b00000001
    // should end up as the int
    //   0b100000001000000000001
    loop {
        let byte = try!(input.read_byte());
        nums.push(byte & 0b01111111);
        if byte & 0b10000000 == 0 {
            break
        }
    }
    let mut result = 0u;
    for (i, byte) in nums.iter().rev().enumerate() {
        result |= *byte as uint << (i * 7)
    }
    return Ok(result);
}

fn get_track_notes(input: &mut Reader) -> Result<MidiTrack, MidiError> {
    //! Reads the notes of a track. The Reader has to be positioned at the
    //! first byte of the track and will read until it finds the END OF TRACK
    //! event.
    let mut notes: MidiTrack = vec![];
    let mut last_event: u8 = 0;
    loop {
        try!(extract_varlen(input));
        let first_byte = try!(input.read_byte());
        let need_reuse = first_byte & 0x80 == 0;
        let type_and_channel = if need_reuse {
            last_event
        } else {
            first_byte
        };
        let event_type = type_and_channel >> 4;
        let param_1 = if need_reuse {
            first_byte
        } else {
            try!(input.read_byte())
        };
        // Meta event
        if type_and_channel == 0xFF {
            let meta_length = try!(extract_varlen(input));
            // Skip meta data
            try!(input.read_exact(meta_length));
            // End of track event
            if param_1 == 0x2F {
                break;
            }
        }
        // SysEx events
        else if event_type == 0xF {
            let sysex_length = try!(extract_varlen(input));
            // Skip it
            try!(input.read_exact(sysex_length));
        }
        match event_type {
            // Note on
            0x9 => {
                let param_2 = try!(input.read_byte());
                if param_2 != 0 {
                    notes.push(param_1);
                }
            }
            // Note off, aftertouch, controller, pitch bend
            0x8 | 0xA | 0xB | 0xE => {
                // They have a second param (one byte), so we need to
                // read it
                try!(input.read_byte());
            }
            // Program change, aftertouch
            // those events don't have any more params and we don't need
            // to handle them.
            0xC | 0xD => (),
            _ => (),
        }

        last_event = type_and_channel;
    }
    return Ok(notes);
}

pub fn get_notes(input: &mut Reader, track_no: int) -> Result<MidiTrack, MidiError> {
    //! Get the track with number track_no from the MIDI file given by input.
    let midi_header = try!(input.read_exact(4));
    if midi_header != vec![0x4D, 0x54, 0x68, 0x64] {
        return Err(InvalidFile("Invalid MIDI file header".to_string()))
    }
    // Skip the chunk size
    try!(input.read_exact(4));
    // Skip format type
    try!(input.read_exact(2));
    let number_of_tracks = try!(input.read_be_u16()) as int;
    if track_no >= number_of_tracks {
        return Err(InvalidTrackNumber(track_no))
    }
    // Skip time division
    try!(input.read_exact(2));

    // Reader is now at the position of the first track
    for tn in range(0, number_of_tracks) {
        let track_header = try!(input.read_exact(4));
        if track_header != vec![0x4D, 0x54, 0x72, 0x6B] {
            return Err(InvalidFile(format!("Invalid MIDI track header in track {}", tn)));
        }
        let chunk_size = try!(input.read_be_u32());
        if tn != track_no {
            // We just read and discard the track's data
            try!(input.read_exact(chunk_size as uint));
            continue
        }
        return get_track_notes(input);
    }
    Err(UnknownError)
}

fn random_delta_time() -> u8 {
    //! Return a random delta time, used for writing the output file
    let range = Range::<u8>::new(15, 30);
    let mut rng = rand::task_rng();
    range.ind_sample(&mut rng)
}

fn build_track_data(notes: &MidiTrack) -> Vec<u8> {
    //! Build and return the events for a track (without the track header)
    let mut result = Vec::new();
    for note in notes.iter() {
        // Do everything on channel 1
        result.push_all([0x00, 0x91, *note, 127]);
        result.push(random_delta_time());
        result.push_all([0x81, *note, 0]);
    }
    // Append End Of Track event
    result.push_all([0xFF, 0x2F, 0x00]);
    result
}

pub fn write_midi_file(writer: &mut Writer, notes: MidiTrack) -> io::IoResult<()> {
    //! Takes a writer and some notes and writes a valid MIDI file, playing
    //! the notes with random speed.
    // Write file header
    try!(writer.write_str("MThd"));
    try!(writer.write([0x00, 0x00, 0x00, 0x06]));
    try!(writer.write([0x00, 0x01, 0x00, 0x01, 0x00, 0x30]));
    // Write track header
    try!(writer.write_str("MTrk"));
    let track_data = build_track_data(&notes);
    try!(writer.write_be_u32(track_data.len() as u32));
    try!(writer.write(track_data.as_slice()));
    Ok(())
}
