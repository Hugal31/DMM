use std::str::FromStr;

use nom::{
    digit,
    alt, value, named, map_res, char, delimited, escaped_transform, none_of, do_parse, opt, tag, recognize, flat_map, call, parse_to, tuple,
    types::CompleteStr
};

use super::Literal;

named!(pub parse_literal<CompleteStr, Literal>,
    alt!(
        parse_float    => { |f| Literal::Float(f) }
        | parse_number => { |i| Literal::Number(i) }
        | parse_string => { |s| Literal::Str(s) }
    )
);

named!(parse_number<CompleteStr, i64>,
    map_res!(
        recognize!(
            do_parse!(
                opt!(tag!("-")) >>
                digit >>
                ()
            )
        ), |s: CompleteStr| i64::from_str(*s))
);

named!(parse_string<CompleteStr, String>,
    delimited!(
        char!('"'),
        escaped_transform!(none_of!("\\\""), '\\',
            alt!(
                char!('\\') => { |_| "\\" }
                | char!('"') => { |_| "\"" }
                | char!('n') => { |_| "\n" }
            )
        ),
        char!('"')
    )
);

named!(parse_float<CompleteStr, f64>,
    flat_map!(call!(recognize_float), parse_to!(f64))
);

// Copied from NOM, slightly modified
named!(recognize_float<CompleteStr, CompleteStr>,
    recognize!(tuple!(
      opt!(alt!(char!('+') | char!('-'))),
      alt!(
        value!((), tuple!(digit, char!('.'), opt!(digit)))
      | value!((), tuple!(char!('.'), digit))
      ),
      opt!(tuple!(
        alt!(char!('e') | char!('E')),
        opt!(alt!(char!('+') | char!('-'))),
        digit
        )
      )
    )
  )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_literal(CompleteStr("123")), Ok((CompleteStr(""), Literal::Number(123))));
        assert_eq!(parse_literal(CompleteStr("-1")), Ok((CompleteStr(""), Literal::Number(-1))));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_literal(CompleteStr("123.2")), Ok((CompleteStr(""), Literal::Float(123.2))));
        assert_eq!(parse_literal(CompleteStr("-1.2")), Ok((CompleteStr(""), Literal::Float(-1.2))));
        assert_eq!(parse_literal(CompleteStr("-1.2E-1")), Ok((CompleteStr(""), Literal::Float(-1.2E-1))));
        assert_eq!(parse_literal(CompleteStr("42.e+1")), Ok((CompleteStr(""), Literal::Float(42e+1))));
        assert_eq!(parse_literal(CompleteStr(".2e1")), Ok((CompleteStr(""), Literal::Float(0.2e1))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_literal(CompleteStr("\"This is a string\"")),
            Ok((CompleteStr(""), Literal::Str("This is a string".to_string())))
        );
        assert_eq!(
            parse_literal(CompleteStr("\"This is an \\\"escaped\\nstring\"")),
            Ok((CompleteStr(""), Literal::Str("This is an \"escaped\nstring".to_string())))
        );
    }
}
