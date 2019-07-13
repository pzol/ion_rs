#[macro_use] extern crate ion;
use std::fs::read_to_string;

fn read_file<T: AsRef<str>>(filename: T) -> String {
    read_to_string(filename.as_ref()).expect(&format!("Failed reading of the file '{}'", filename.as_ref()))
}

fn read_ion<T: AsRef<str>>(filename: T) -> ion::Ion {
    ion!(read_file(filename))
}

fn read_err_ion<T: AsRef<str>>(filename: T) -> ion::IonError {
    read_file(filename).parse::<ion::Ion>().unwrap_err()
}

#[test]
fn test_ion() {
    let ion = read_ion("tests/data/test.ion");
    let expected = read_file("tests/expected/test.ion");

    assert_eq!(expected, ion.to_string());
}

#[test]
fn hotel_ion() {
    let ion = read_ion("tests/data/hotel.ion");
    let expected = read_file("tests/expected/hotel.ion");

    assert_eq!(expected, ion.to_string());
}

#[test]
fn broken_array_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_array_and_eof.ion");
    let expected = "ParserErrors([ParserError { lo: 55, hi: 55, desc: \"Cannot finish an array\" }])";

    assert_eq!(expected, ion_err.to_string());
}

#[test]
fn broken_dictionary_and_eof() {
    let ion_err = read_err_ion("tests/data/broken_dictionary_and_eof.ion");
    let expected = "ParserErrors([ParserError { lo: 67, hi: 67, desc: \"Cannot finish a dictionary\" }])";

    assert_eq!(expected, ion_err.to_string());
}
