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
  map!(many0!(alt!(one_of!(" \t\x0c") => { |_|() } | newline)), |_| ())
);
named!(spaces_nonl<CompleteStr, ()>,
  map!(many0!(one_of!(" \t\x0c")), |_| ())
);


/// Like `ws!()`, but ignores comments as well
macro_rules! ws_comm (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::Convert;
      use nom::Err;

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
mod dmm;
mod dictionary;
mod literal;
mod var_edit;

/// Parsed DMM AST
#[derive(Clone, Debug, PartialEq)]
pub struct DMM<'s> {
    pub dictionary: Vec<DictionaryEntry<'s>>,
    pub grid: Vec<GridEntry<'s>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DictionaryEntry<'s> {
    pub key: &'s str,
    pub datums: Vec<Datum<'s>>
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

use nom::types::CompleteStr;
