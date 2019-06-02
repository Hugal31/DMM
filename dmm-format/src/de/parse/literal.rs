use std::str::FromStr;

use nom::{
    alt, call, char, delimited, digit, do_parse, escaped_transform, flat_map, map_res, named,
    none_of, not, one_of, opt, parse_to, recognize, tag, take_while, tuple, types::CompleteStr,
    value,
};

use super::Literal;

named!(pub parse_literal<CompleteStr, Literal>,
    alt!(
        tuple!(parse_number, not!(one_of!("Ee."))) => { |(i, _)| Literal::Number(i) }
        | parse_float    => { |f| Literal::Float(f) }
        | parse_string => { |s| Literal::Str(s) }
        | parse_path   => { |p: CompleteStr| Literal::Path(p.0.to_string()) }
        // TODO
        | tag!("null") => { |_| Literal::Str("null".to_string()) }
        | recognize!(parse_list)=> { |p: CompleteStr| Literal::Str(p.0.to_string()) }
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
       alt!(parse_double_quoted_string | parse_single_quoted_string)
);

named!(parse_double_quoted_string<CompleteStr, String>,
    delimited!(
        char!('"'),
        escaped_transform!(none_of!("\\\""), '\\',
            alt!(
                char!('\\') => { |_| "\\" }
                | char!('"') => { |_| "\"" }
                | char!('n') => { |_| "\n" }
                | char!('i') => { |_| "\\i" }
            )
        ),
        char!('"')
    )
);

named!(parse_single_quoted_string<CompleteStr, String>,
    delimited!(
        char!('\''),
        escaped_transform!(none_of!("\\'"), '\\',
            alt!(
                char!('\\') => { |_| "\\" }
                | char!('\'') => { |_| "'" }
                | char!('n') => { |_| "\n" }
                | char!('i') => { |_| "\\i" }
            )
        ),
        char!('\'')
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
        value!((), tuple!(digit, opt!(tuple!(char!('.'), opt!(digit)))))
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

named!(pub parse_path<CompleteStr, CompleteStr>,
    recognize!(
        tuple!(
            char!('/'),
            take_while!(is_path_char)
        )
    )
);

fn is_path_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '_' => true,
        _ => false,
    }
}

named!(pub parse_list<CompleteStr, Vec<Literal>>,
    ws_comm!(delimited!(tag!("list("), separated_list!(char!(','), parse_literal), char!(')')))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(
            parse_literal(CompleteStr("123")),
            Ok((CompleteStr(""), Literal::Number(123)))
        );
        assert_eq!(
            parse_literal(CompleteStr("-1")),
            Ok((CompleteStr(""), Literal::Number(-1)))
        );
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(
            parse_literal(CompleteStr("123.2")),
            Ok((CompleteStr(""), Literal::Float(123.2)))
        );
        assert_eq!(
            parse_literal(CompleteStr("-1.2")),
            Ok((CompleteStr(""), Literal::Float(-1.2)))
        );
        assert_eq!(
            parse_literal(CompleteStr("-1.2E-1")),
            Ok((CompleteStr(""), Literal::Float(-1.2E-1)))
        );
        assert_eq!(
            parse_literal(CompleteStr("42.e+1")),
            Ok((CompleteStr(""), Literal::Float(42e+1)))
        );
        assert_eq!(
            parse_literal(CompleteStr(".2e1")),
            Ok((CompleteStr(""), Literal::Float(0.2e1)))
        );
        assert_eq!(
            parse_literal(CompleteStr("5e+006")),
            Ok((CompleteStr(""), Literal::Float(5.0e6)))
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_literal(CompleteStr("\"This is a string\"")),
            Ok((
                CompleteStr(""),
                Literal::Str("This is a string".to_string())
            ))
        );
        assert_eq!(
            parse_literal(CompleteStr("\"This is an \\\"escaped\\nstring\"")),
            Ok((
                CompleteStr(""),
                Literal::Str("This is an \"escaped\nstring".to_string())
            ))
        );
    }
}
