extern crate ion;
use std::fs::File;
use std::io::Read;
use ion::Ion;

#[test]
fn name() {
    let mut f = File::open("tests/test.ion").unwrap();
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);

    let ion : Ion = s.parse().unwrap();
    println!("{}", ion);
}
