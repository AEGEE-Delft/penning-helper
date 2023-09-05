use std::{
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Neg, Sub, SubAssign, Mul},
};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy, Default, Hash, PartialOrd, Ord)]
pub struct Euro(i32, i32);

impl Euro {
    pub fn xml_string(&self) -> String {
        format!("{}.{:0<2}", self.0, self.1)
    }

    pub fn new(euros: i32, cents: i32) -> Self {
        Euro(euros, cents)
    }

    fn fix_negative_cents(mut self) -> Self {
        if self.1 < 0 {
            self.0 -= 1;
            self.1 += 100;
        }
        self
    }
}

impl Neg for Euro {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Euro::default() - self
    }
}

impl Sum for Euro {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

impl From<(i32, i32)> for Euro {
    fn from(value: (i32, i32)) -> Self {
        Euro(value.0, value.1)
    }
}

impl Debug for Euro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0 as f64 + self.1 as f64 / 100.0;
        f.debug_tuple("Euro").field(&value).finish()
    }
}

impl Display for Euro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first = if f.sign_minus() {
            if self.0 < 0 {
                self.0 * -1
            } else {
                self.0
            }
        } else {
            self.0
        };
        write!(f, "€{},{:0<2}", first, self.1)
    }
}

impl<'de> Deserialize<'de> for Euro {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (euros, cents) = s
            .split_once(',')
            .ok_or_else(|| serde::de::Error::custom("Invalid Euro format"))?;
        let euros = euros
            .parse::<i32>()
            .map_err(|_| serde::de::Error::custom("Invalid Euro format"))?;
        let cents = cents
            .parse::<i32>()
            .map_err(|_| serde::de::Error::custom("Invalid Euro format"))?;
        Ok(Euro(euros, cents))
    }
}

impl Serialize for Euro {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{},{}", self.0, self.1).serialize(serializer)
    }
}

impl Add for Euro {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let cents = self.1 + rhs.1;
        let euros = self.0 + rhs.0 + cents / 100;
        let cents = cents % 100;
        Euro(euros, cents).fix_negative_cents()
    }
}

impl AddAssign for Euro {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Euro {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let cents = self.1 - rhs.1;
        let euros = self.0 - rhs.0 + cents / 100;
        let cents = cents % 100;
        // take care of cent underflow
        let (euros, cents) = if cents < 0 {
            (euros - 1, cents + 100)
        } else {
            (euros, cents)
        };
        Euro(euros, cents).fix_negative_cents()
    }
}

impl SubAssign for Euro {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl From<f32> for Euro {
    fn from(value: f32) -> Self {
        let euros = value as i32;
        let cents = ((value - euros as f32) * 100.0).round() as i32;
        Euro(euros, cents).fix_negative_cents()
    }
}

impl From<f64> for Euro {
    fn from(value: f64) -> Self {
        let euros = value as i32;
        let cents = ((value - euros as f64) * 100.0).round() as i32;
        Euro(euros, cents).fix_negative_cents()
    }
}

impl Mul<f64> for Euro {
    type Output = Euro;

    fn mul(self, rhs: f64) -> Self::Output {
        let value = (self.0 as f64 + self.1 as f64 / 100.0) * rhs;
        Euro::from(value)
    }
}