use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// DMM structure as it can be found in files
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct DMM {
    dictionary: HashMap<Key, Vec<Datum>>,
    grid: HashMap<(u32, u32, u32), Vec<Key>>,
}

/// In a DMM, a Datum is represented by its path (type) and a list of assigns to its var.
/// Example:
/// ```dmm
/// /obj/machinery/firealarm{
///        dir = 8;
///        pixel_x = 24
///        }
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Datum {
    /// Path, or the type of the datum
    path: String,
    /// List of assignments to the datum instance
    assigns: HashMap<String, Literal>,
}

/// DMM Literal
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Literal {
    Str(String),
    Number(i64),
    Float(f64),
}

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Key(u32);

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
                "a string of maximum {} characters within {}",
                Self::MAX_KEY_CHAR,
                Self::BASE
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            debug_assert_eq!(52, Self::BASE.len());

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
