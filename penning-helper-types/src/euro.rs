use std::{
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
    str::FromStr, hash::Hash,
};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Default, PartialOrd)]
pub struct Euro(f64);

impl Eq for Euro {}

impl Ord for Euro {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Euro {
    pub fn xml_string(&self) -> String {
        format!("{:.2}", self.0)
    }

    pub fn new(euros: i32, cents: i32) -> Self {
        let cents = cents as f64 / 100.0;
        let euros = euros as f64;
        Euro(euros + cents)
    }

    /// rounds to the nearest cent
    fn round(mut self) -> Self {
        self.0 = (self.0 * 100.0).round() / 100.0;
        self
    }
}

impl Hash for Euro {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state)
    }
}

impl FromStr for Euro {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = s.parse::<f64>().map_err(|_| "Invalid Euro format")?;
        Ok(Euro::from(f))
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
    fn from((euros, cents): (i32, i32)) -> Self {
        Euro::new(euros, cents).round()
    }
}

impl From<i32> for Euro {
    fn from(value: i32) -> Self {
        Euro::new(value, 0).round()
    }
}

macro_rules! from_integer_type {
    ($($t:ty),* $(,)?) => {
        $(impl From<$t> for Euro {
            fn from(value: $t) -> Self {
                Euro::new(value as i32, 0).round()
            }
        })*
    };
}

from_integer_type!(i8, i16, i64, i128, isize, u8, u16, u32, u64, u128, usize);


impl Debug for Euro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0 as f64;
        f.debug_tuple("Euro").field(&value).finish()
    }
}

impl Display for Euro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = if f.sign_minus() { self.0.abs() } else { self.0 };
        write!(f, "€{:.2}", value)
    }
}

impl<'de> Deserialize<'de> for Euro {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.replace('.', "").replace(',', ".");
        let r = s.parse::<f64>().map_err(serde::de::Error::custom)?;
        Ok(Euro(r))
    }
}

impl Serialize for Euro {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{:0.02}", self.0)
            .replace('.', ",")
            .serialize(serializer)
    }
}

impl Add for Euro {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Euro(self.0 + rhs.0).round()
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
        Euro(self.0 - rhs.0).round()
    }
}

impl Sub<f64> for Euro {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        Euro(self.0 - rhs).round()
    }
}

impl SubAssign for Euro {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl From<f32> for Euro {
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

impl From<f64> for Euro {
    fn from(value: f64) -> Self {
        Euro(value).round()
    }
}

impl Mul<f64> for Euro {
    type Output = Euro;

    fn mul(self, rhs: f64) -> Self::Output {
        Euro(self.0 * rhs).round()
    }
}

impl Mul<usize> for Euro {
    type Output = Euro;

    fn mul(self, rhs: usize) -> Self::Output {
        Euro(self.0 * rhs as f64).round()
    }
}

#[cfg(test)]
mod tests {
    use crate::Euro;

    #[test]
    fn add_test() {
        let a = Euro::from(1);
        let b = Euro::from(2);
        let c = Euro::from(3);
        assert_eq!(a + b, c);
    }

    macro_rules! foo {
        ($($a:expr),+) => {
            $(Euro::from($a) + )+ Euro::from(0)
        };
    }
    #[test]
    fn add_test_2() {
        let a = foo!(
            29.30, 89.78, 82.16, 8.80, 49.21, 52.83, 36.21, 22.80, 14.80, 5.50, 5.41, 2.50, 53.98,
            40.70, 3.80, 83.45, 85.34, 57.00, 68.80, 37.58, 83.81, 28.80, 7.00, 7.50, 25.60, 84.44,
            93.30, 28.50, 74.30, 95.80, 50.00, 24.30, 71.41, 50.00, 14.50, 10.30, 83.80, 65.50,
            66.80, 7.00, 34.14, 47.30, 55.00, 53.17, 10.80, 33.20, 94.44, 5.00, 16.50, 60.61,
            11.00, 6.00, 50.00, 50.00, 1.50, 25.00, 4.00, 64.73, 4.00, 28.80, 55.30, 25.00, 4.00,
            73.32, 55.92, 4.00, 4.00, 4.00, 49.32, 5.00, 80.00, 5.00, 5.00, 65.00, 97.12, 98.00,
            46.62, 70.50, 80.50, 5.00, 10.50, 5.00, 73.00, 5.00, 78.00, 81.23, 5.41, 71.00, 60.00,
            78.50, 81.50, 60.00, 71.00, 16.50, 11.00, 11.00, 4.00, 4.00, 4.00, 100.00, 100.00,
            100.00, 100.00, 100.00, 100.00, 100.00, 100.00, 100.00, 100.00, 100.00, 100.00, 100.00,
            100.00, 100.00, 100.00, 100.00, 100.00
        );

        assert_eq!(a.xml_string(), "5821.04");
    }
}
