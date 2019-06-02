use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// TODO: Rework this
struct GridIterator<'d> {
    dmm: &'d DMM,
    grid_iter: std::collections::hash_map::Iter<'d, (u32, u32, u32), Vec<Key>>,
    cell_iter: Option<std::slice::Iter<'d, Key>>,
    current_coords: Option<(u32, u32, u32)>,
}

impl<'d> GridIterator<'d> {
    fn new(dmm: &'d DMM) -> Self {
        let mut grid_iter = dmm.grid.iter();
        let (current_coords, cell_iter) = grid_iter
            .next()
            .map(|l| (Some(*l.0), Some(l.1.iter())))
            .unwrap_or((None, None));
        GridIterator {
            dmm,
            grid_iter,
            cell_iter,
            current_coords,
        }
    }
}

impl<'d> Iterator for GridIterator<'d> {
    type Item = ((u32, u32, u32), &'d [Datum]);

    fn next(&mut self) -> Option<Self::Item> {
        let coords = self.current_coords.as_mut()?;
        let new_coords = *coords;
        coords.1 += 1;
        let cell_iter = self.cell_iter.as_mut()?;
        if let Some(key) = cell_iter.next() {
            Some((new_coords, self.dmm.dictionary.get(key).unwrap()))
        } else if let Some(next_cell) = self.grid_iter.next() {
            self.current_coords = Some(*next_cell.0);
            self.cell_iter = Some(next_cell.1.iter());
            self.next()
        } else {
            self.current_coords = None;
            None
        }
    }
}

/// DMM structure as it can be found in files
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct DMM {
    dictionary: HashMap<Key, Vec<Datum>>,
    grid: HashMap<(u32, u32, u32), Vec<Key>>,
}

impl DMM {
    pub fn new(
        dictionary: HashMap<Key, Vec<Datum>>,
        grid: HashMap<(u32, u32, u32), Vec<Key>>,
    ) -> Self {
        // TODO Check every key exists
        DMM { dictionary, grid }
    }

    pub fn iter(&self) -> impl Iterator<Item = ((u32, u32, u32), &[Datum])> {
        GridIterator::new(self)
    }
}

/// In a DMM, a Datum is represented by its path (type) and a list of assigns to its var.
/// Example:
/// ```dmm
/// /obj/machinery/firealarm{
///        dir = 8;
///        pixel_x = 24
///        }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Datum {
    /// Path, or the type of the datum
    path: String,
    /// List of assignments to the datum instance
    var_edits: HashMap<String, Literal>,
}

impl Datum {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self::with_var_edits(path, HashMap::new())
    }

    pub fn with_var_edits<S: Into<String>>(path: S, var_edits: HashMap<String, Literal>) -> Self {
        Datum {
            path: path.into(),
            var_edits,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn var_edits(&self) -> &HashMap<String, Literal> {
        &self.var_edits
    }
}

/// DMM Literal
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Literal {
    Path(String),
    Str(String),
    Number(i64),
    Float(f64),
}

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Key(u32);

impl Key {
    const MAX_KEY_CHAR: usize = 3;
    const BASE: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    pub fn new(k: u32) -> Self {
        Key(k)
    }
}

impl From<u32> for Key {
    fn from(k: u32) -> Self {
        Key::new(k)
    }
}

// TODO Refactor with KeyStrConverter, make a real error
impl std::convert::TryFrom<&'_ str> for Key {
    type Error = ();

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        if v.is_empty() || v.len() > Self::MAX_KEY_CHAR {
            return Err(());
        }

        let mut value = 0;

        for c in v.chars() {
            if let Some(index) = Self::BASE.find(c) {
                value = Self::BASE.len() as u32 * value + index as u32;
            } else {
                return Err(());
            }
        }

        Ok(Key(value))
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;

    use std::fmt;

    use serde::{
        de::{Deserializer, Error, Unexpected, Visitor},
        ser::Serializer,
    };

    impl<'de> Deserialize<'de> for Key {
        fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(KeyStrConverter)
        }
    }

    impl Serialize for Key {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&KeyStrConverter::key_to_str(*self))
        }
    }

    struct KeyStrConverter;

    impl KeyStrConverter {
        const MAX_KEY_CHAR: usize = 3;
        const BASE: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

        pub fn key_to_str(key: Key) -> String {
            let mut result = String::new();
            let mut num = key.0;

            while num != 0 {
                let index = num as usize % Self::BASE.len();
                result.insert_str(0, Self::BASE.get(index..=index).unwrap());
                num /= Self::BASE.len() as u32;
            }

            while result.len() < Self::MAX_KEY_CHAR {
                result.insert(0, 'a');
            }

            result
        }
    }

    impl<'de> Visitor<'de> for KeyStrConverter {
        type Value = Key;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "a number between {} and {}, or a string of maximum {} characters within {}",
                std::u32::MIN,
                std::u32::MAX,
                Self::MAX_KEY_CHAR,
                Self::BASE
            )
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v >= 0 {
                self.visit_u64(v as u64)
            } else {
                Err(E::invalid_value(Unexpected::Signed(v), &self))
            }
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Key(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v <= u64::from(std::u32::MAX) {
                self.visit_u32(v as u32)
            } else {
                Err(E::invalid_value(Unexpected::Unsigned(v), &self))
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v.is_empty() || v.len() > Self::MAX_KEY_CHAR {
                return Err(E::invalid_length(v.len(), &self));
            }

            let mut value = 0;

            for c in v.chars() {
                if let Some(index) = Self::BASE.find(c) {
                    value = Self::BASE.len() as u32 * value + index as u32;
                } else {
                    return Err(E::invalid_value(Unexpected::Char(c), &self));
                }
            }

            Ok(Key(value))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use serde_test::{assert_tokens, Token};
        #[test]
        fn test_serialize_key() {
            assert_tokens(&Key(42), &[Token::Str("aaQ")]);

            assert_tokens(
                &Key((KeyStrConverter::BASE
                    .len()
                    .pow(KeyStrConverter::MAX_KEY_CHAR as u32)
                    - 1) as u32),
                &[Token::Str("ZZZ")],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let dmm = DMM::new(
            {
                let mut m = HashMap::new();
                m.insert(
                    0.into(),
                    vec![
                        Datum::new("/turf/open/space/basic"),
                        Datum::new("/area/space"),
                    ],
                );
                m.insert(
                    1.into(),
                    vec![Datum::with_var_edits(
                        "/obj/machinery/firealarm",
                        vec![
                            ("dir".to_string(), Literal::Number(8)),
                            ("name".to_string(), Literal::Str("thing".to_string())),
                        ]
                        .into_iter()
                        .collect(),
                    )],
                );
                m
            },
            {
                let mut m = HashMap::new();
                m.insert((1, 1, 1), vec![0.into(), 1.into()]);
                m.insert((2, 1, 1), vec![0.into()]);
                m
            },
        );
        let mut iterator = dmm.iter();

        assert_eq!(
            iterator.collect::<HashMap<(u32, u32, u32), &[Datum]>>(),
            vec![
                (
                    (1, 1, 1),
                    &[
                        Datum::new("/turf/open/space/basic"),
                        Datum::new("/area/space"),
                    ][..]
                ),
                (
                    (1, 2, 1),
                    &[Datum::with_var_edits(
                        "/obj/machinery/firealarm",
                        vec![
                            ("dir".to_string(), Literal::Number(8)),
                            ("name".to_string(), Literal::Str("thing".to_string()))
                        ]
                        .into_iter()
                        .collect()
                    ),][..]
                ),
                (
                    (2, 1, 1),
                    &[
                        Datum::new("/turf/open/space/basic"),
                        Datum::new("/area/space"),
                    ][..]
                ),
            ]
            .into_iter()
            .collect()
        )
    }
}
