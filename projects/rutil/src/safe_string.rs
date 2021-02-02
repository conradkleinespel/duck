#[cfg(feature = "serde")]
use serde::de::{Deserialize, Deserializer, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};
use std::convert::Into;
#[cfg(feature = "serde")]
use std::fmt;
use std::ops::{Deref, DerefMut, Drop};
use std::{ptr, sync::atomic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeString {
    inner: String,
}

#[cfg(feature = "serde")]
struct StringVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for StringVisitor {
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(String::from(v))
    }
    type Value = String;
}

impl SafeString {
    pub fn new() -> SafeString {
        SafeString {
            inner: String::new(),
        }
    }

    pub fn from_string(inner: String) -> SafeString {
        SafeString { inner }
    }

    pub fn into_inner(mut self) -> String {
        std::mem::replace(&mut self.inner, String::new())
    }
}

impl Drop for SafeString {
    fn drop(&mut self) {
        let default = u8::default();

        for c in unsafe { self.inner.as_bytes_mut() } {
            unsafe { ptr::write_volatile(c, default) };
        }

        atomic::fence(atomic::Ordering::SeqCst);
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}

impl Deref for SafeString {
    type Target = String;

    fn deref(&self) -> &String {
        &self.inner
    }
}

impl DerefMut for SafeString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Into<SafeString> for String {
    fn into(self) -> SafeString {
        SafeString::from_string(self)
    }
}

impl<'a> Into<SafeString> for &'a str {
    fn into(self) -> SafeString {
        self.to_string().into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for SafeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.deref())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SafeString {
    fn deserialize<D>(deserializer: D) -> Result<SafeString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_string(StringVisitor)
            .map(|parsed_value| SafeString {
                inner: parsed_value,
            })
    }
}

#[cfg(all(test, all(feature = "serde", feature = "serde_json")))]
mod test {
    use super::SafeString;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[test]
    fn safe_string_serialization() {
        let s = SafeString {
            inner: String::from("blabla"),
        };

        match serde_json::to_string(&s) {
            Ok(json) => assert_eq!("\"blabla\"", json),
            Err(_) => panic!("Serialization failed, somehow"),
        }
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct TestStruct {
        password: SafeString,
    }

    #[test]
    fn safe_string_within_struct_serialization() {
        let ts = TestStruct {
            password: SafeString {
                inner: String::from("blabla"),
            },
        };

        match serde_json::to_string(&ts) {
            Ok(json) => assert_eq!("{\"password\":\"blabla\"}", json),
            Err(_) => panic!("Serialization failed, somehow"),
        }
    }

    #[test]
    fn safe_string_deserialization() {
        let s = "\"blabla\"";

        let res: Result<SafeString, serde_json::Error> = serde_json::from_str(s);

        match res {
            Ok(ss) => assert_eq!(
                ss,
                SafeString {
                    inner: String::from("blabla")
                }
            ),
            Err(_) => panic!("Deserialization failed"),
        }
    }

    #[test]
    fn safe_string_within_struct_deserialization() {
        let json = "{\"password\":\"blabla\"}";
        let res: Result<TestStruct, serde_json::Error> = serde_json::from_str(json);
        match res {
            Ok(ts) => assert_eq!(
                ts,
                TestStruct {
                    password: SafeString {
                        inner: String::from("blabla")
                    }
                }
            ),
            Err(_) => panic!("Deserialization failed"),
        }
    }
}
