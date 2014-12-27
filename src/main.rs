#![feature(macro_rules)]
#![feature(phase)]
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
use markov::MarkovChain;
use midi::MidiTrack;
use std::io;
use std::error::Error;
use std::hash::hash;
mod markov;
mod midi;

#[allow(unused_must_use)]
fn print_errorstack(err: &Error) {
    //! Prints an error to stderr and the error that caused it, until
    //! error.cause() is None.
    let stderr = &mut io::stderr();
    write!(stderr, "Error: {}\n", err.description());
    match err.detail() {
        Some(detail) => {
            write!(stderr, "     {}\n", detail);
        },
        None => (),
    };
    let mut current = err.cause();
    while current.is_some() {
        let c = current.unwrap();
        write!(stderr, "Caused by: {}\n", c.description());
        match c.detail() {
            Some(detail) => {
                write!(stderr, "     {}\n", detail);
            },
            None => (),
        };
        current = c.cause();
    }
}

fn compose(notes: &Vec<u8>, degree: uint, length: uint) -> Vec<u8> {
    //! Takes an original sequence of notes and creates a new composition
    let mut m = MarkovChain::<u64, u8>::new();
    let mut last_note = Vec::new();
    for i in range(0, degree) {
        last_note.push(notes[i])
    }
    for note in notes.iter().skip(degree) {
        m.mark(hash(&last_note), *note);
        last_note.remove(0);
        last_note.push(*note);
    }
    let mut composition = Vec::<u8>::new();
    while composition.len() != length {
        match m.random_successor(hash(&last_note)) {
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


docopt!(Args deriving Show, "
Usage:
    digital_composer [options] <input> <track>
    digital_composer --help

Options:
    -h, --help                  Show this help message
    -o <file>, --output <file>  Specify the output file name [default: composition.mid]
    -l <len>, --length <len>    Specify the length in notes [default: 100]
    -d <deg>, --degree <deg>    Specify the degree of the markov chain [default: 1]
",
    arg_track: uint,
    flag_length: uint,
    flag_degree: uint,
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
    let polyphonic = 1u;

    // Actual program
    println!("Reading {}...", input_filename);
    let mut file = std::io::File::open(&Path::new(input_filename));
    let notes = match midi::get_notes(&mut file, input_trackno as int) {
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

    let mut output = std::io::File::create(&Path::new(output_filename));
    match midi::write_midi_file(&mut output, &composition) {
        Err(n) => {
            print_errorstack(&n);
            return;
        }
        _ => (),
    }
}
