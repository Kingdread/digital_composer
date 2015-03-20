extern crate rand;
use std::io;
use std::io::{Read, Write};
use std::error;
use std::fmt;
use rand::distributions::{Range, IndependentSample};
use byteorder;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use self::MidiError::{InvalidFile, InvalidTrackNumber, IoError, UnknownError};
pub type MidiTrack = Vec<u8>;

#[derive(Debug)]
enum MidiError {
    InvalidFile(String),
    InvalidTrackNumber(u16),
    IoError(io::Error),
    UnknownError,
}

impl error::FromError<io::Error> for MidiError {
    fn from_error(err: io::Error) -> MidiError {
        IoError(err)
    }
}

impl error::FromError<byteorder::Error> for MidiError {
    fn from_error(err: byteorder::Error) -> MidiError {
        match err {
            byteorder::Error::UnexpectedEOF =>
                InvalidFile("Reached end of file too early".to_string()),
            byteorder::Error::Io(err) => IoError(err),
        }
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

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            IoError(ref e) => Some(e as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            InvalidFile(ref t) => f.write_str(t),
            InvalidTrackNumber(nr) => f.write_fmt(format_args!("The file has no track {}", nr)),
            _ => Ok(()),
        }
    }
}

trait ByteReader : Read {
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buffer = [0u8];
        try!(self.read(&mut buffer));
        Ok(buffer[0])
    }

    fn read_exact(&mut self, length: usize) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::with_capacity(length);
        try!(self.read(&mut buffer));
        Ok(buffer)
    }
}
impl<T: Read + ?Sized> ByteReader for T {}

fn extract_varlen<R: Read>(input: &mut R) -> io::Result<u32> {
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
    let mut result = 0u32;
    for (i, byte) in nums.iter().rev().enumerate() {
        result |= (*byte as u32) << (i * 7)
    }
    Ok(result)
}

fn get_track_notes<R: Read>(input: &mut R) -> Result<MidiTrack, MidiError> {
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
            try!(input.read_exact(meta_length as usize));
            // End of track event
            if param_1 == 0x2F {
                break;
            }
        }
        // SysEx events
        else if event_type == 0xF {
            let sysex_length = try!(extract_varlen(input));
            // Skip it
            try!(input.read_exact(sysex_length as usize));
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
    Ok(notes)
}

pub fn get_notes<R: Read>(input: &mut R, track_no: u16) -> Result<MidiTrack, MidiError> {
    //! Get the track with number track_no from the MIDI file given by input.
    let midi_header = try!(input.read_exact(4));
    if midi_header != vec![0x4D, 0x54, 0x68, 0x64] {
        return Err(InvalidFile("Invalid MIDI file header".to_string()))
    }
    // Skip the chunk size
    try!(input.read_exact(4));
    // Skip format type
    try!(input.read_exact(2));
    let number_of_tracks = try!(input.read_u16::<BigEndian>());
    if track_no >= number_of_tracks {
        return Err(InvalidTrackNumber(track_no))
    }
    // Skip time division
    try!(input.read_exact(2));

    // Reader is now at the position of the first track
    for tn in (0 .. number_of_tracks) {
        let track_header = try!(input.read_exact(4));
        if track_header != vec![0x4D, 0x54, 0x72, 0x6B] {
            return Err(InvalidFile(format!("Invalid MIDI track header in track {}", tn)));
        }
        let chunk_size = try!(input.read_u32::<BigEndian>());
        if tn != track_no {
            // We just read and discard the track's data
            try!(input.read_exact(chunk_size as usize));
            continue
        }
        return get_track_notes(input);
    }
    Err(UnknownError)
}

fn random_delta_time() -> u8 {
    //! Return a random delta time, used for writing the output file
    let range = Range::<u8>::new(15, 30);
    let mut rng = rand::thread_rng();
    range.ind_sample(&mut rng)
}

fn build_track_data(notes: &MidiTrack) -> Vec<u8> {
    //! Build and return the events for a track (without the track header)
    let mut result = Vec::new();
    for note in notes.iter() {
        // Do everything on channel 1
        result.push_all(&[0x00, 0x91, *note, 127]);
        result.push(random_delta_time());
        result.push_all(&[0x81, *note, 0]);
    }
    // Append End Of Track event
    result.push_all(&[0x00, 0xFF, 0x2F, 0x00]);
    result
}

pub fn write_midi_file<W: Write>(writer: &mut W, tracks: &Vec<MidiTrack>) -> io::Result<()> {
    //! Takes a writer and some notes and writes a valid MIDI file, playing
    //! the notes with random speed.
    // Write file header
    try!(writer.write_all("MThd".as_bytes()));
    // Header size (always 6)
    try!(writer.write_all(&[0x00, 0x00, 0x00, 0x06]));
    // MIDI format type
    try!(writer.write_all(&[0x00, 0x01]));
    // Number of tracks
    try!(writer.write_u16::<BigEndian>(tracks.len() as u16));
    // Time division
    try!(writer.write_all(&[0x00, 0x30]));
    for notes in tracks.iter() {
        // Write track header
        try!(writer.write_all("MTrk".as_bytes()));
        let track_data = build_track_data(notes);
        try!(writer.write_u32::<BigEndian>(track_data.len() as u32));
        try!(writer.write_all(track_data.as_slice()));
    }
    Ok(())
}
