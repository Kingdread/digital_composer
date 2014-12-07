Digital Composer
================
A program that takes a MIDI file as input and outputs a MIDI file that sounds
way worse. It does so by using modern technology, advanced mathematics, quantum
analytics, state of the art algorithms and high-end markov chains, combined
with a pinch black magic.

![Meme](http://i2.kym-cdn.com/photos/images/original/000/234/739/fa5.jpg)

Motivation
----------
* <del>Become the world's most powerful man</del>
* fun
* learn [Rust](http://www.rust-lang.org)

This is my first experiment with Rust, so the code is very messy.

Installation
------------
Tested with `rustc 0.13.0-nightly (7e43f419c 2014-11-15 13:22:24 +0000)`.
To build digital\_composer, run `cargo build`. The executable will be
`target/digital_composer`.

Usage
-----
    Usage: digital_composer ARGS input-file trackno
        -o FILE, --output=FILE    The output file name, defaults to composition.mid [Optional]
        -l LENGTH, --length=LENGTH    Length of the resulting composition, defaults to 100 [Optional]
        -d DEG, --degree=DEG    Degree of the markov chain, defaults to 1 [Optional]
        -h, --help    Show the help [Optional]

License
-------
               DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
                   Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
               DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
      TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
     0. You just DO WHAT THE FUCK YOU WANT TO.
