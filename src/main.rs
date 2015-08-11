extern crate ion;
extern crate docopt;
extern crate rustc_serialize;
use std::fs::File;
use std::io::{ self, Read };
use docopt::Docopt;

use ion::{ Ion, Validator };

static USAGE : &'static str = "
ion - the ION toolbelt

Usage:
  ion fmt <filename>
  ion validate <filename> <spec>
  ion -h | --help
  ion -v | --version

Options:
  -h --help      Show this screen.
  -v --version   Show version.
";

#[derive(RustcDecodable, Debug)]
struct Args {
  arg_filename: String,
  arg_spec: String,
  flag_version: bool,
  cmd_fmt: bool,
  cmd_validate: bool

}

const VERSION : &'static str = env!("CARGO_PKG_VERSION");

pub fn main() {
    let args : Args = Docopt::new(USAGE)
                              .and_then(|d| d.decode())
                              .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("ion v{}", VERSION);
        return();
    } else if args.cmd_fmt {
        fmt(args.arg_filename)
    } else if args.cmd_validate {

        validate(args.arg_filename, args.arg_spec)
    }
}

fn fmt(filename: String) {
    let s = read_to_string(&filename).unwrap();

    let ion : Ion = s.parse().unwrap();
    println!("{}", ion);
}

fn validate(filename: String, spec: String) {
    let s_ion  = read_to_string(&filename).ok().expect("Expected ion file");
    let s_spec = read_to_string(&spec).ok().expect("Expected ion spec file");
    let ion  : Ion = s_ion.parse().unwrap();
    let spec : Ion = s_spec.parse().unwrap();

    let validator = Validator::new(&ion, &spec);
    if let Some(errors) = validator.validate() {
        for error in errors {
            println!("{}", error);
        }
    }
}

fn read_to_string(filename: &str) -> Result<String, io::Error> {
    let mut f = try!(File::open(filename));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}
