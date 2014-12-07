#![feature(macro_rules)]
extern crate getopts;
use getopts::{optopt, optflag, getopts, OptGroup, HasArg, Occur};
use getopts::HasArg::{Yes, No, Maybe};
use getopts::Occur::{Req, Optional, Multi};
use markov::MarkovChain;
use std::io;
use std::os;
use std::error::Error;
use std::hash::hash;
mod markov;
mod midi;

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

fn format_option(opt: &OptGroup) -> String {
    //! Format an OptGoup to print in the help, e.g.
    //!     -o FILE, --output=FILE Sets the output filename
    format!("    -{}{}, --{}{}    {} [{}]",
            opt.short_name,
            match opt.hasarg {
                Yes => format!(" {}", opt.hint),
                No => "".to_string(),
                Maybe => format!(" [{}]", opt.hint),
            },
            opt.long_name,
            match opt.hasarg {
                Yes => format!("={}", opt.hint),
                No => "".to_string(),
                Maybe => format!("[={}]", opt.hint),
            },
            opt.desc,
            match opt.occur {
                Req => "Required",
                Optional => "Optional",
                Multi => "Multi",
            },
            )
}

fn print_usage(options: &[OptGroup]) {
    //! Print the usage help and every possible argument, given by
    //! options.
    println!("Usage: {} ARGS input-file trackno", os::args()[0]);
    for opt in options.iter() {
        println!("{}", format_option(opt));
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
    
macro_rules! opt_argument(
    ($matches:expr, $name:expr, $ty:ty, $default:expr, $opts:expr) => (
        match $matches.opt_str($name) {
            Some(arg) => match from_str::<$ty>(arg.as_slice()) {
                Some(conv) => conv,
                None => {
                    print_usage($opts);
                    return;
                },
            },
            None => $default,
        };
    );
)

fn main() {
    let opts = &[
        optopt("o", "output", "The output file name, defaults to composition.mid", "FILE"),
        optopt("l", "length", "Length of the resulting composition, defaults to 100", "LENGTH"),
        optopt("d", "degree", "Degree of the markov chain, defaults to 1", "DEG"),
        optflag("h", "help", "Show the help"),
    ];
    // Argument parsing and validating
    let matches = match getopts(os::args().tail(), opts) {
        Ok(m) => m,
        Err(err) => {
            write!(&mut io::stderr(), "{}\n", err);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }
    if matches.free.len() != 2 {
        print_usage(opts);
        return;
    }
    let input_filename = matches.free[0].as_slice();
    let output_filename = match matches.opt_str("o") {
        Some(f) => f,
        None => "composition.mid".to_string(),
    };
    let input_trackno = match from_str::<uint>(matches.free[1].as_slice()) {
        Some(n) => n,
        None => {
            print_usage(opts);
            return;
        },
    };
    let degree = opt_argument!(matches, "d", uint, 1, opts);
    let length = opt_argument!(matches, "l", uint, 100, opts);

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

    let composition = compose(&notes, degree, length);

    let mut output = std::io::File::create(&Path::new(output_filename));
    match midi::write_midi_file(&mut output, composition) {
        Err(n) => {
            print_errorstack(&n);
            return;
        }
        _ => (),
    }
}
