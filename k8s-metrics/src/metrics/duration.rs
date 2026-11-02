use std::fmt;
use std::time;

use serde::{de, ser};

struct DurationVisitor;

// Go's time.Duration is an i64 count of nanoseconds. Metric windows use
// std::time::Duration, so only its nonnegative range is representable here.
const MAX_NANOS: u64 = (1 << 63) - 1;

pub(super) fn deserialize<'de, D>(d: D) -> Result<time::Duration, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(DurationVisitor)
}

impl<'de> de::Visitor<'de> for DurationVisitor {
    type Value = time::Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Duration in Kubernetes notation")
    }

    fn visit_str<E>(self, text: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse(text).ok_or_else(|| de::Error::custom(format!("invalid duration: '{text}'")))
    }
}

// Jiff's friendly grammar differs from Go's: it rejects repeated or reordered
// units and fractional nonfinal components, while accepting non-Go unit names.
fn parse(text: &str) -> Option<time::Duration> {
    let (negative, mut remaining) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if remaining == "0" {
        return Some(time::Duration::ZERO);
    }
    if remaining.is_empty() {
        return None;
    }

    let mut nanos = 0_u64;
    while !remaining.is_empty() {
        let number_end = remaining.find(|c: char| !c.is_ascii_digit() && c != '.')?;
        let (number, rest) = remaining.split_at(number_end);
        let unit_end = rest
            .find(|c: char| c.is_ascii_digit() || c == '.')
            .unwrap_or(rest.len());
        let (unit, rest) = rest.split_at(unit_end);
        remaining = rest;
        let unit = match unit {
            "ns" => 1,
            "us" | "µs" | "μs" => 1_000,
            "ms" => 1_000_000,
            "s" => 1_000_000_000,
            "m" => 60_000_000_000,
            "h" => 3_600_000_000_000,
            _ => return None,
        };
        let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
        if whole.is_empty() && fraction.is_empty() {
            return None;
        }
        let whole = if whole.is_empty() {
            0
        } else {
            whole.parse::<u64>().ok()?
        };
        let component = whole
            .checked_mul(unit)?
            .checked_add(fractional_nanos(fraction, unit)?)?;
        nanos = nanos.checked_add(component).filter(|&n| n <= MAX_NANOS)?;
    }
    if negative && nanos != 0 {
        return None;
    }
    Some(time::Duration::from_nanos(nanos))
}

fn fractional_nanos(digits: &str, unit: u64) -> Option<u64> {
    // Match time.ParseDuration's fractional precision and floating-point
    // rounding, including fractions longer than nine decimal places.
    let mut value = 0_u64;
    let mut scale = 1_f64;
    let mut full = false;
    for digit in digits.bytes() {
        if !digit.is_ascii_digit() {
            return None;
        }
        if full {
            continue;
        }
        if value > MAX_NANOS / 10 {
            full = true;
            continue;
        }
        let next = value * 10 + u64::from(digit - b'0');
        if next > MAX_NANOS + 1 {
            full = true;
            continue;
        }
        value = next;
        scale *= 10.0;
    }
    // This is nonnegative and bounded by the largest unit (one hour).
    Some((value as f64 * (unit as f64 / scale)) as u64)
}

pub(super) fn serialize<S>(duration: &time::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    if duration.as_nanos() > u128::from(MAX_NANOS) {
        return Err(ser::Error::custom(
            "duration exceeds Go's time.Duration range",
        ));
    }
    serializer.collect_str(&format_args!(
        "{}.{:09}s",
        duration.as_secs(),
        duration.subsec_nanos()
    ))
}
