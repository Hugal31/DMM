use nom::{char, named, opt, sep, types::CompleteStr};

use super::literal::parse_path;
use super::var_edit::parse_var_edit;
use super::{Datum, VarEdit};

named!(pub parse_datum<CompleteStr, Datum>,
    ws_comm!(
        do_parse!(
            path: parse_path >>
            var_edits: opt!(parse_data_block) >>
            (Datum { path: path.0, var_edits: var_edits.unwrap_or_default() })
        )
    )
);

named!(parse_data_block<CompleteStr, Vec<VarEdit>>,
    ws_comm!(
        delimited!(char!('{'), parse_var_edits, char!('}'))
    )
);

named!(parse_var_edits<CompleteStr, Vec<VarEdit>>,
    ws_comm!(
        do_parse!(
            list: separated_list!(char!(';'), parse_var_edit) >>
            opt!(char!(';')) >>
            (list)
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::de::parse::Literal;

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path(CompleteStr("/foo/bar123")),
            Ok((CompleteStr(""), CompleteStr("/foo/bar123")))
        );
        assert!(parse_path(CompleteStr("foo/bar")).is_err());
    }

    #[test]
    fn test_parse_data_block() {
        assert_eq!(
            parse_data_block(CompleteStr("{ abc = -3;}")),
            Ok((
                CompleteStr(""),
                vec![VarEdit {
                    identifier: "abc",
                    value: Literal::Number(-3),
                }]
            ))
        );
    }
    #[test]
    fn test_parse_var_edits() {
        assert_eq!(
            parse_var_edits(CompleteStr("abc = -3; bcd = \"42\"")),
            Ok((
                CompleteStr(""),
                vec![
                    VarEdit {
                        identifier: "abc",
                        value: Literal::Number(-3),
                    },
                    VarEdit {
                        identifier: "bcd",
                        value: Literal::Str("42".to_string()),
                    }
                ]
            ))
        );
        assert_eq!(
            parse_var_edits(CompleteStr("abc = -3; bcd = \"42\";")),
            Ok((
                CompleteStr(""),
                vec![
                    VarEdit {
                        identifier: "abc",
                        value: Literal::Number(-3),
                    },
                    VarEdit {
                        identifier: "bcd",
                        value: Literal::Str("42".to_string()),
                    }
                ]
            ))
        );
    }
}
