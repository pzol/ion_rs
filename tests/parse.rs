#[macro_use] extern crate ion;
use std::fs::File;
use std::io::Read;

macro_rules! read_file {
    ($filename:expr) => ({
        let mut f = File::open($filename).expect(&format!("Cannot open the file '{}'", $filename));
        let mut s = String::new();
        f.read_to_string(&mut s).expect(&format!("Failed reading of the file '{}'", $filename));
        s
    })
}

macro_rules! read_ion {
    ($filename:expr) => ({
        ion!(read_file!($filename))
    })
}

#[test]
fn test_ion() {
    let ion = read_ion!("tests/data/test.ion");
    let expected = read_file!("tests/expected/test.ion");

    assert_eq!(expected, ion.to_string());
}

#[test]
fn hotel_ion() {
    let ion = read_ion!("tests/data/hotel.ion");
    let expected = read_file!("tests/expected/hotel.ion");

    assert_eq!(expected, ion.to_string());
}

#[test]
fn broken_array_and_eof() {
    let ion = read_ion!("tests/data/broken_array_and_eof.ion");
    let expected = read_file!("tests/expected/broken_array_and_eof.ion");

    assert_eq!(expected, ion.to_string());
}

#[test]
fn broken_dictionary_and_eof() {
    let ion = read_ion!("tests/data/broken_dictionary_and_eof.ion");
    let expected = read_file!("tests/expected/broken_dictionary_and_eof.ion");

    assert_eq!(expected, ion.to_string());
}
