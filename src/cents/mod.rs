//! The cents datatype. Each cent is the smallest atomic unit of the asset type

#[cfg(test)]
mod test;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;
use std::ops;

pub const MAX_DECIMAL_PLACES: usize = 4;
const WHOLE_MULTIPLIER: u64 = 10_u64.pow(MAX_DECIMAL_PLACES as u32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cents(u64);

impl Cents {
    pub fn new(cents: u64) -> Self {
        Self(cents)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Checked add, returns None if overflowed
impl ops::Add<Cents> for Cents {
    type Output = Option<Cents>;

    fn add(self, rhs: Cents) -> Option<Cents> {
        self.value()
            .checked_add(rhs.value())
            .map(|res| Self::new(res))
    }
}

/// Checked sub, returns None if overflowed
impl ops::Sub<Cents> for Cents {
    type Output = Option<Cents>;

    fn sub(self, rhs: Cents) -> Option<Cents> {
        self.value()
            .checked_sub(rhs.value())
            .map(|res| Self::new(res))
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 / WHOLE_MULTIPLIER)?;
        write!(f, ".")?;
        let decimals = self.0 % WHOLE_MULTIPLIER;
        if decimals == 0 {
            return write!(f, "0000");
        }
        // leading zeros
        let mut decimals_copy = decimals;
        while decimals_copy < (WHOLE_MULTIPLIER / 10 - 1) {
            write!(f, "0")?;
            decimals_copy *= 10;
        }
        write!(f, "{}", decimals)
    }
}

struct CentsVisitor;

impl<'de> Visitor<'de> for CentsVisitor {
    type Value = Cents;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "A number with up to {} decimal places",
            MAX_DECIMAL_PLACES
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let decimal_index = match s.find(".") {
            Some(i) => i,
            None => {
                let whole: u64 = s.parse().map_err(de::Error::custom)?;
                return Ok(Self::Value::new(whole * WHOLE_MULTIPLIER));
            }
        };
        let decimal_places = s.len() - decimal_index - 1;
        if decimal_places > MAX_DECIMAL_PLACES {
            return Err(de::Error::custom(format!(
                "Too many decimal places: {}. Max {}",
                decimal_places, MAX_DECIMAL_PLACES,
            )));
        }
        let whole: u64 = s[..decimal_index].parse().map_err(de::Error::custom)?;
        let decimal: u64 = s[decimal_index + 1..].parse().map_err(de::Error::custom)?;
        let whole_cents = whole
            .checked_mul(WHOLE_MULTIPLIER)
            .ok_or_else(|| overflow_error(&format!("whole {} * {}", whole, WHOLE_MULTIPLIER)))?;
        let decimal_multiplier = 10_u64.pow((MAX_DECIMAL_PLACES - decimal_places) as u32);
        let decimal_cents = decimal * decimal_multiplier;
        let cents = whole_cents.checked_add(decimal_cents).ok_or_else(|| {
            overflow_error(&format!(
                "whole {} + decimal {}",
                whole_cents, decimal_cents
            ))
        })?;
        Ok(Self::Value::new(cents))
    }
}

impl<'de> Deserialize<'de> for Cents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CentsVisitor)
    }
}

fn overflow_error<E>(attempted_op: &str) -> E
where
    E: de::Error,
{
    de::Error::custom(&format!("Overflow error: {}", attempted_op))
}
