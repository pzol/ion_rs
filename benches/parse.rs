#![feature(test)]

extern crate ion;
extern crate test;

use ion::Ion;
use test::{Bencher, black_box};

const DEF_HOTEL_ON_START: &str = include_str!("data/def_hotel_on_start.ion");
const DEF_HOTEL_ON_END: &str = include_str!("data/def_hotel_on_end.ion");

mod parse {
    use super::*;
    use std::str::FromStr;

    #[bench]
    fn section_on_start_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_START);
            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_END);
            black_box(result.unwrap())
        })
    }
}

mod parse_filtered {
    use super::*;

    const FILTERED_SECTIONS: &[&str] = &["CONTRACT", "DEF.HOTEL"];

    #[bench]
    fn section_on_start_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_START, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_END, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    }
}


//test parse::section_on_start_of_ion          ... bench:   4,187,092 ns/iter (+/- 125,166)
//test parse::section_on_end_of_ion            ... bench:   4,223,583 ns/iter (+/- 155,651)
//test parse_filtered::section_on_start_of_ion ... bench:      15,027 ns/iter (+/- 1,612)
//test parse_filtered::section_on_end_of_ion   ... bench:     962,318 ns/iter (+/- 31,853)
