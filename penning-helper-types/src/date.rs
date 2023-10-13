use std::{fmt::Display, ops::{Deref, DerefMut}};

use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Date {
    date: NaiveDate,
}

impl Default for Date {
    fn default() -> Self {
        Self::today()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:04}-{:02}-{:02}",
            self.date.year_ce().1,
            self.date.month(),
            self.date.day()
        ))
    }
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        Some(Self {
            date: NaiveDate::from_ymd_opt(year, month, day)?,
        })
    }

    pub fn today() -> Self {
        Self {
            date: chrono::Local::now().date_naive(),
        }
    }

    pub fn in_some_days(days: u64) -> Self {
        let now = chrono::Local::now();
        let res = now.checked_add_days(Days::new(days)).unwrap();
        Self {
            date: res.date_naive(),
        }
    }
}

impl Deref for Date {
    type Target = NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.date
    }
}

impl DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.date
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "{:04}-{:02}-{:02}",
            self.date.year_ce().1,
            self.date.month(),
            self.date.day()
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

        Self::new(year, month, day)
            .ok_or_else(|| serde::de::Error::custom(format!("{} is not a valid date", s)))
    }
}
