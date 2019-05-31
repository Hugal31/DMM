use std::convert::TryInto;

use nom::{
    alt, char, many0, many1, map, named, none_of, one_of, opt, preceded, tag, tuple,
    types::CompleteStr, value,
};

named!(newline<CompleteStr, ()>,
  map!(
    many1!(
      tuple!(
        spaces_nonl,
        opt!(preceded!(tag!("//"), many0!(none_of!("\n")))),
        char!('\n')
      )
    ),
    |_| ()
  )
);

named!(spaces_nl<CompleteStr, ()>,
  value!((), many0!(alt!(one_of!(" \t\x0c") => { |_|() } | newline)))
);
named!(spaces_nonl<CompleteStr, ()>,
  value!((), many0!(one_of!(" \t\x0c")))
);

/// Like `ws!()`, but ignores comments as well
macro_rules! ws_comm (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::Convert;
      use nom::Err;
      use nom::sep;

      match sep!($i, $crate::de::parse::spaces_nl, $($args)*) {
        Err(e) => Err(e),
        Ok((i1,o))    => {
          match $crate::de::parse::spaces_nl(i1) {
            Err(e) => Err(Err::convert(e)),
            Ok((i2,_))    => Ok((i2, o))
          }
        }
      }
    }
  )
);

mod datum;
mod dictionary;
mod dmm;
mod literal;
mod var_edit;

pub use self::dmm::parse_dmm;

/// Parsed DMM AST
#[derive(Clone, Debug, PartialEq)]
pub struct DMM<'s> {
    pub dictionary: Vec<DictionaryEntry<'s>>,
    pub grid: Vec<GridEntry<'s>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DictionaryEntry<'s> {
    pub key: &'s str,
    pub datums: Vec<Datum<'s>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Datum<'s> {
    pub path: &'s str,
    pub var_edits: Vec<VarEdit<'s>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarEdit<'s> {
    pub identifier: &'s str,
    pub value: Literal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// Not a reference because it could have been escaped
    Str(String),
    Number(i64),
    Float(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridEntry<'s> {
    pub coords: (u32, u32, u32),
    pub keys: Vec<&'s str>,
}

impl Into<::dmm::DMM> for DMM<'_> {
    fn into(self) -> ::dmm::DMM {
        ::dmm::DMM::new(
            self.dictionary
                .into_iter()
                .map(|de| {
                    (
                        de.key.try_into().unwrap(),
                        de.datums
                            .into_iter()
                            .map(std::convert::Into::into)
                            .collect(),
                    )
                })
                .collect(),
            self.grid
                .into_iter()
                .map(|GridEntry { coords, keys }| {
                    (
                        coords,
                        keys.into_iter().map(|k| k.try_into().unwrap()).collect(),
                    )
                })
                .collect(),
        )
    }
}

impl Into<::dmm::Datum> for Datum<'_> {
    fn into(self) -> ::dmm::Datum {
        ::dmm::Datum::with_var_edits(
            self.path,
            self.var_edits
                .into_iter()
                .map(|VarEdit { identifier, value }| (identifier.to_string(), value.into()))
                .collect(),
        )
    }
}

impl Into<::dmm::Literal> for Literal {
    fn into(self) -> ::dmm::Literal {
        match self {
            Literal::Float(f) => ::dmm::Literal::Float(f),
            Literal::Number(n) => ::dmm::Literal::Number(n),
            Literal::Str(s) => ::dmm::Literal::Str(s),
        }
    }
}
