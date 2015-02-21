#![feature(plugin)]
#![feature(io, core, collections, path, rand, hash)]
#![plugin(docopt_macros)]
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
use markov::MarkovChain;
use midi::MidiTrack;
use std::old_io as io;
use std::error::Error;
use std::hash::{hash, SipHasher};
mod markov;
mod midi;

#[allow(unused_must_use)]
fn print_errorstack(err: &Error) {
    //! Prints an error to stderr and the error that caused it, until
    //! error.cause() is None.
    let stderr = &mut io::stderr();
    write!(stderr, "Error: {}\n    {}\n", err.description(), err);
    let mut current = err.cause();
    while current.is_some() {
        let c = current.unwrap();
        write!(stderr, "Caused by: {}\n    {}\n", c.description(), c);
        current = c.cause();
    }
}

fn get_hash(inp: &Vec<u8>) -> u64 {
    return hash::<Vec<u8>, SipHasher>(inp);
}

fn compose(notes: &Vec<u8>, degree: u32, length: u32) -> Vec<u8> {
    //! Takes an original sequence of notes and creates a new composition
    let mut m = MarkovChain::<u64, u8>::new();
    let mut last_note = Vec::new();
    for i in range(0u32, degree) {
        last_note.push(notes[i as usize])
    }
    for note in notes.iter().skip(degree as usize) {
        m.mark(get_hash(&last_note), *note);
        last_note.remove(0);
        last_note.push(*note);
    }
    let mut composition = Vec::<u8>::new();
    while composition.len() != length as usize {
        match m.random_successor(get_hash(&last_note)) {
            Some(note) => {
                composition.push(note);
                last_note.remove(0);
                last_note.push(note);
            }
            None => (),
        }
    };
    composition
}


docopt!(Args derive Debug, "
Usage:
    digital_composer [options] <input> <track>
    digital_composer --help

Options:
    -h, --help                  Show this help message
    -o <file>, --output <file>  Specify the output file name [default: composition.mid]
    -l <len>, --length <len>    Specify the length in notes [default: 100]
    -d <deg>, --degree <deg>    Specify the degree of the markov chain [default: 1]
",
    arg_track: u16,
    flag_length: u32,
    flag_degree: u32,
);

fn main() {
    // Argument parsing and validating
    // Much shorter thanks to docopt
    let args: Args = Args::docopt().decode().unwrap_or_else(|err| err.exit());
    let input_filename = args.arg_input;
    let output_filename = args.flag_output;
    let input_trackno = args.arg_track;
    let degree = args.flag_degree;
    let length = args.flag_length;
    let polyphonic = 1u32;

    // Actual program
    println!("Reading {}...", input_filename);
    let mut file = io::File::open(&Path::new(input_filename));
    let notes = match midi::get_notes(&mut file, input_trackno) {
        Ok(n) => n,
        Err(n) => {
            print_errorstack(&n);
            return;
        },
    };

    let mut composition = Vec::<MidiTrack>::new();
    for _ in range(0, polyphonic) {
        // TODO: Don't create the chain over and over again
        composition.push(compose(&notes, degree, length));
    }

    let mut output = io::File::create(&Path::new(output_filename));
    match midi::write_midi_file(&mut output, &composition) {
        Err(n) => {
            print_errorstack(&n);
            return;
        }
        _ => (),
    }
}
