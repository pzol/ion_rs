use std::{ error, fmt };

#[derive(Clone, Debug)]
pub enum IonError {
    MissingSection(String),
    MissingValue(String),
}

impl error::Error  for IonError{
    fn description(&self) -> &str { "IonError" }
}

impl fmt::Display for IonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
