mod parse;

use std::io::Read;

use nom::types::CompleteStr;

use self::parse::*;
use crate::error::{Error, Result};

pub fn from_reader<R: Read>(mut input: R) -> Result<::dmm::DMM> {
    let mut s = String::new();
    input.read_to_string(&mut s).map_err(Error::Io)?;
    from_str(&s)
}

pub fn from_str(input: &str) -> Result<::dmm::DMM> {
    parse_dmm(CompleteStr(input))
        .map_err(|e| Error::Nom(e.into_error_kind()))
        .and_then(|(remaining, dmm)| {
            if !remaining.0.trim_end().is_empty() {
                Err(Error::TrailingCharacters)
            } else {
                Ok(dmm)
            }
        })
        .map(std::convert::Into::into)
}
