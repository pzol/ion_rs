#[macro_use] extern crate ion;
use std::fs::File;
use std::io::Read;

macro_rules! read_ion {
    ($filename:expr) => ({
        let mut f = File::open($filename).unwrap();
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        ion!(s)
    })
}

#[test]
fn test_ion() {
    let ion = read_ion!("tests/test.ion");
    println!("{}", ion);
}

#[test]
fn hotel_ion() {
    let ion = read_ion!("tests/hotel.ion");
    let exp = r#"[HOTEL]
75042 = { dist = { beach_km = 4.1 }, loc = [ "M", "B" ], view = "SV" }
category = 4.5
dict = { a = "b" }
name = "HOTEL"
ptype = "H"

"#;
    assert_eq!(exp, ion.to_string());
}

