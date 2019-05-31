use nom::{
    alpha,
    alphanumeric,
    types::CompleteStr
};

use super::{VarEdit};
use super::literal::parse_literal;

named!(pub parse_var_edit<CompleteStr, VarEdit>,
    ws_comm!(
        do_parse!(
            identifier: parse_identifier >>
            char!('=') >>
            value: parse_literal >>
            (VarEdit { identifier: identifier.0, value })
        )
    )
);

named!(parse_identifier<CompleteStr, CompleteStr>,
    recognize!(
        tuple!(
            alt!(alpha | tag!("_")),
            opt!(many0!(alt!(alphanumeric | tag!("_"))))
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::de::parse::Literal;

    #[test]
    fn test_var_edit_comm() {
        assert_eq!(
            parse_var_edit(CompleteStr("abc // This is a comment\n= -3")),
            Ok((CompleteStr(""), VarEdit{
                identifier: "abc",
                value: Literal::Number(-3)
            }))
        );
    }

    #[test]
    fn test_var_edit() {
        assert_eq!(
            parse_var_edit(CompleteStr("abc = -3")),
            Ok((CompleteStr(""), VarEdit{
                identifier: "abc",
                value: Literal::Number(-3)
            }))
        );
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(
            parse_identifier(CompleteStr("thi_ngs4")),
            Ok((CompleteStr(""), CompleteStr("thi_ngs4")))
        );
        assert!(parse_identifier(CompleteStr("4things")).is_err());
    }
}
