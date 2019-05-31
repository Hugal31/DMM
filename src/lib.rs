use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// DMM structure as it can be found in files
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct DMM {
    dictionary: HashMap<Key, Vec<Datum>>,
    grid: HashMap<(u32, u32, u32), Vec<Key>>,
}

impl DMM {
    pub fn new(dictionary: HashMap<Key, Vec<Datum>>, grid: HashMap<(u32, u32, u32), Vec<Key>>) -> Self {
        DMM {
            dictionary,
            grid,
        }
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
}

/// DMM Literal
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Literal {
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
        if v.len() == 0 || v.len() > Self::MAX_KEY_CHAR {
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
                result.insert_str(0, Self::BASE.get(index..index + 1).unwrap());
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

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where
            E: Error, {
            if v >= 0 {
                self.visit_u64(v as u64)
            } else {
                Err(E::invalid_value(Unexpected::Signed(v), &self))
            }
        }

        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where
            E: Error, {
            Ok(Key(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where
            E: Error, {
            if v <= std::u32::MAX as u64 {
                self.visit_u32(v as u32)
            } else {
                Err(E::invalid_value(Unexpected::Unsigned(v), &self))
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v.len() == 0 || v.len() > Self::MAX_KEY_CHAR {
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
