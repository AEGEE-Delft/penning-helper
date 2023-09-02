use std::fmt::Display;

use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        ))
    }
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn today() -> Self {
        let now = chrono::Local::now();
        Self {
            year: now.year(),
            month: now.month(),
            day: now.day(),
        }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        ))
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.split('-');
        let year = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("Missing Year in Date"))?;
        let month = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("Missing Month in Date"))?;
        let day = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("Missing Day in Date"))?;
        let year = year
            .parse()
            .map_err(|_| serde::de::Error::custom("Year is not a number"))?;
        let month = month
            .parse()
            .map_err(|_| serde::de::Error::custom("Month is not a number"))?;
        let day = day
            .parse()
            .map_err(|_| serde::de::Error::custom("Day is not a number"))?;
        Ok(Date { year, month, day })
    }
}
