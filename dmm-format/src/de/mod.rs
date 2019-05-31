mod parse;

use crate::error::{Error, Result};
use self::parse::*;
use nom::types::CompleteStr;

pub fn from_str(input: &str) -> Result<::dmm::DMM> {
    dbg!(input);
    parse_dmm(CompleteStr(input))
        .map_err(|e| Error::Nom(e.into_error_kind()))
        .and_then(|(remaining, dmm)| {
            if !remaining.0.trim_end().is_empty() {
                dbg!((remaining.0, dmm));
                Err(Error::TrailingCharacters)
            } else {
                Ok(dmm)
            }
        })
        .map(|dmm| dmm.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

}
