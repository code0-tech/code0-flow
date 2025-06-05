use std::str::FromStr;

/// STATIC:
/// The service will start with no Sagittarius in mind
///
/// DYNAMIC:
/// The service will start with Sagittarius in mind
#[derive(PartialEq, Eq, Debug)]
pub enum Mode {
    STATIC,
    DYNAMIC,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "static" => Ok(Mode::STATIC),
            "dynamic" => Ok(Mode::DYNAMIC),
            _ => Ok(Mode::STATIC),
        }
    }
}
