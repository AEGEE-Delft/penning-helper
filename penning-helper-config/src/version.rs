use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Version<const V: u8>;

impl<const V: u8> Serialize for Version<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(V)
    }
}

impl<'de, const V: u8> Deserialize<'de> for Version<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        if value == V {
            Ok(Version::<V>)
        } else {
            Err(serde::de::Error::custom("Invalid Success value"))
        }
    }
}

impl<const C1: u8, const C2: u8> PartialEq<Version<C2>> for Version<C1> {
    fn eq(&self, _: &Version<C2>) -> bool {
        C1 == C2
    }
}

impl<const C1: u8, const C2: u8> PartialOrd<Version<C2>> for Version<C1> {
    fn partial_cmp(&self, _: &Version<C2>) -> Option<std::cmp::Ordering> {
        // self < other == Less
        Some(C1.cmp(&C2))
    }
}
