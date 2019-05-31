use nom::{
    alpha,
    types::CompleteStr
};

use super::{Datum, DictionaryEntry};

use super::datum::parse_datum;

named!(pub parse_dictionary_entry<CompleteStr, DictionaryEntry>,
    ws_comm!(
        do_parse!(
            key: parse_key >>
            char!('=') >>
            datums: parse_datums_block >>
            (DictionaryEntry { key: key.0, datums })
        )
    )
);

named!(parse_datums_block<CompleteStr, Vec<Datum>>,
    ws_comm!(
        delimited!(
            char!('('),
            many0!(parse_datum),
            char!(')')
        )
    )
);

named!(parse_key<CompleteStr, CompleteStr>,
    delimited!(
        char!('"'),
        alpha,
        char!('"')
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dictionary_entry() {
        assert_eq!(
            parse_dictionary_entry(CompleteStr("\"aaB\" = (  )")),
            Ok((CompleteStr(""), DictionaryEntry {
                key: "aaB",
                datums: Vec::new(),
            }))
        );
    }

    #[test]
    fn test_parse_datums_block() {
        assert_eq!(parse_datums_block(CompleteStr("(  )")), Ok((CompleteStr(""), Vec::new())));
        assert_eq!(
            parse_datums_block(CompleteStr("(   /foo/bar    )")),
            Ok((CompleteStr(""), vec![Datum{path: "/foo/bar", var_edits: Vec::new()}]))
        );
    }

    #[test]
    fn test_parse_key() {
        assert_eq!(parse_key(CompleteStr("\"abC\"")), Ok((CompleteStr(""), CompleteStr("abC"))));
    }
}
