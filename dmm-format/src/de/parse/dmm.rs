use nom::{
    alpha,
    digit,
    types::CompleteStr
};

use super::{DMM, DictionaryEntry, GridEntry};
use super::dictionary::parse_dictionary_entry;

named!(pub parse_dmm<CompleteStr, DMM>,
   do_parse!(
       dictionary: parse_dictionary >>
       grid: parse_grid >>
       (DMM { dictionary, grid })
   )
);

named!(parse_grid<CompleteStr, Vec<GridEntry>>,
    many0!(parse_grid_entry)
);

named!(parse_grid_entry<CompleteStr, GridEntry>,
    ws_comm!(
        do_parse!(
            coords: parse_grid_coords >>
            char!('=') >>
            keys: parse_grid_line >>
            (GridEntry { coords, keys })
        )
    )
);

named!(parse_grid_coords<CompleteStr, (u32, u32, u32)>,
    ws_comm!(
        delimited!(
            char!('('),
            do_parse!(
                u0: map_res!(digit, |u: CompleteStr| u.0.parse()) >>
                char!(',') >>
                u1: map_res!(digit, |u: CompleteStr| u.0.parse()) >>
                char!(',') >>
                u2: map_res!(digit, |u: CompleteStr| u.0.parse()) >>
                (u0, u1, u2)
            ),
            char!(')')
        )
    )
);

named!(parse_grid_line<CompleteStr, Vec<&str>>,
    ws_comm!(delimited!(tag!("{\""), parse_grid_keys, tag!("\"}")))
);

named!(parse_grid_keys<CompleteStr, Vec<&str>>,
    ws_comm!(
        many0!(map!(alpha, |c| c.0))
    )
);

named!(parse_dictionary<CompleteStr, Vec<DictionaryEntry>>,
    ws_comm!(
        separated_list!(char!(','), parse_dictionary_entry)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dmm() {
        assert_eq!(
            parse_dmm(CompleteStr("// COMMENT\n\"aaB\" = ( //OTHER COMMENT\n ), \"aaC\" = ()\n(1,2,3) = {\"aaa\nbbb\"}\n(2,3,4) = {\"aac\nbbc\"}")),
            Ok((CompleteStr(""), DMM {
                dictionary: vec![
                    DictionaryEntry {
                        key: "aaB",
                        datums: Vec::new(),
                    },
                    DictionaryEntry {
                        key: "aaC",
                        datums: Vec::new(),
                    }
                ],
                grid: vec![
                    GridEntry {
                        coords: (1,2,3),
                        keys: vec!["aaa", "bbb"],
                    },
                    GridEntry {
                        coords: (2,3,4),
                        keys: vec!["aac", "bbc"],
                    },
                ],
            }))
        );
    }

    #[test]
    fn test_parse_grid_entry() {
        assert_eq!(
            parse_grid_entry(CompleteStr("(1,2,3) = {\"aaa bbb\"}")),
            Ok((CompleteStr(""), GridEntry {
                coords: (1,2,3),
                keys: vec!["aaa", "bbb"],
            }))
        );
    }

    #[test]
    fn test_parse_grid_coord() {
        assert_eq!(
            parse_grid_coords(CompleteStr(" (1,2, 4 )")),
            Ok((CompleteStr(""), (1, 2, 4)))
        );
    }

    #[test]
    fn test_parse_dictionary() {
        assert_eq!(
            parse_dictionary(CompleteStr("\"aaB\" = (  ), \"aaC\" = () (1,2,3)")),
            Ok((CompleteStr("(1,2,3)"), vec![
                DictionaryEntry {
                    key: "aaB",
                    datums: Vec::new(),
                },
                DictionaryEntry {
                    key: "aaC",
                    datums: Vec::new(),
                }
            ]))
        );

    }
}
