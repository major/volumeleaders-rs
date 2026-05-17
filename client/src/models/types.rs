//! Custom serde types for VolumeLeaders API quirks.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Sentinel epoch milliseconds for .NET `DateTime.MinValue`.
const DATETIME_MIN_EPOCH_MILLIS: i64 = -62_135_596_800_000;

/// Sentinel epoch milliseconds for the year 1900 placeholder.
const DATE_1900_EPOCH_MILLIS: i64 = -2_208_988_800_000;

/// A nullable datetime that deserializes ASP.NET `/Date(epoch_ms)/` JSON
/// strings returned by the VolumeLeaders API.
///
/// Sentinels (.NET `DateTime.MinValue`, year-1900 placeholder), empty strings,
/// and JSON `null` all deserialize to `AspNetDate(None)`. Valid timestamps
/// produce `AspNetDate(Some(datetime))`.
///
/// Serializes `Some(dt)` as an RFC 3339 string and `None` as JSON `null`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspNetDate(pub Option<DateTime<Utc>>);

impl AspNetDate {
    /// Returns the contained datetime, if any.
    pub fn value(&self) -> Option<&DateTime<Utc>> {
        self.0.as_ref()
    }
}

impl<'de> Deserialize<'de> for AspNetDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AspNetDateVisitor)
    }
}

/// Visitor that handles JSON strings in `/Date(ms)/` format, null, and empty
/// strings.
struct AspNetDateVisitor;

impl<'de> Visitor<'de> for AspNetDateVisitor {
    type Value = AspNetDate;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string in /Date(epoch_ms)/ format, empty string, or null")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(AspNetDate(None))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        if value.is_empty() {
            return Ok(AspNetDate(None));
        }

        let inner = value
            .strip_prefix("/Date(")
            .and_then(|s| s.strip_suffix(")/"))
            .ok_or_else(|| de::Error::custom(format!("invalid ASP.NET date format: {value:?}")))?;

        let epoch_millis: i64 = inner.parse().map_err(|_| {
            de::Error::custom(format!(
                "invalid epoch milliseconds in ASP.NET date: {inner:?}"
            ))
        })?;

        if epoch_millis == DATETIME_MIN_EPOCH_MILLIS || epoch_millis == DATE_1900_EPOCH_MILLIS {
            return Ok(AspNetDate(None));
        }

        let dt = DateTime::from_timestamp_millis(epoch_millis)
            .ok_or_else(|| de::Error::custom(format!("out-of-range timestamp: {epoch_millis}")))?;

        Ok(AspNetDate(Some(dt)))
    }
}

impl Serialize for AspNetDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Some(dt) => serializer.serialize_str(&dt.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }
}

/// A boolean that deserializes from JSON `true`/`false`, `0`/`1`, or `null`.
///
/// The VolumeLeaders API returns boolean fields inconsistently as native JSON
/// booleans or as 0/1 integers. `FlexBool` normalizes all variants.
///
/// Serializes `Some(b)` as a JSON boolean and `None` as `null`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlexBool(pub Option<bool>);

impl FlexBool {
    /// Returns the contained boolean value, if any.
    pub fn value(&self) -> Option<bool> {
        self.0
    }
}

impl<'de> Deserialize<'de> for FlexBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FlexBoolVisitor)
    }
}

/// Visitor that handles JSON booleans, 0/1 integers, and null.
struct FlexBoolVisitor;

impl<'de> Visitor<'de> for FlexBoolVisitor {
    type Value = FlexBool;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a boolean, 0, 1, or null")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(FlexBool(Some(value)))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        match value {
            0 => Ok(FlexBool(Some(false))),
            1 => Ok(FlexBool(Some(true))),
            other => Err(de::Error::custom(format!("expected 0 or 1, got {other}"))),
        }
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        match value {
            0 => Ok(FlexBool(Some(false))),
            1 => Ok(FlexBool(Some(true))),
            other => Err(de::Error::custom(format!("expected 0 or 1, got {other}"))),
        }
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(FlexBool(None))
    }
}

impl Serialize for FlexBool {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Some(b) => serializer.serialize_bool(b),
            None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- AspNetDate deserialize --

    #[test]
    fn aspnet_date_valid() {
        let d: AspNetDate = serde_json::from_str(r#""/Date(1767225600000)/""#).unwrap();
        let expected = DateTime::from_timestamp_millis(1_767_225_600_000).unwrap();
        assert_eq!(d, AspNetDate(Some(expected)));
    }

    #[test]
    fn aspnet_date_null() {
        let d: AspNetDate = serde_json::from_str("null").unwrap();
        assert_eq!(d, AspNetDate(None));
    }

    #[test]
    fn aspnet_date_empty_string() {
        let d: AspNetDate = serde_json::from_str(r#""""#).unwrap();
        assert_eq!(d, AspNetDate(None));
    }

    #[test]
    fn aspnet_date_dotnet_min() {
        let d: AspNetDate = serde_json::from_str(r#""/Date(-62135596800000)/""#).unwrap();
        assert_eq!(d, AspNetDate(None));
    }

    #[test]
    fn aspnet_date_1900_sentinel() {
        let d: AspNetDate = serde_json::from_str(r#""/Date(-2208988800000)/""#).unwrap();
        assert_eq!(d, AspNetDate(None));
    }

    #[test]
    fn aspnet_date_invalid_format() {
        let result: Result<AspNetDate, _> = serde_json::from_str(r#""2026-01-01""#);
        assert!(result.is_err());
    }

    #[test]
    fn aspnet_date_invalid_type() {
        let result: Result<AspNetDate, _> = serde_json::from_str("42");
        assert!(result.is_err());
    }

    // -- AspNetDate serialize --

    #[test]
    fn aspnet_date_serialize_some() {
        let dt = DateTime::from_timestamp(1_767_225_600, 0).unwrap();
        let d = AspNetDate(Some(dt));
        let json = serde_json::to_string(&d).unwrap();
        assert_eq!(json, format!(r#""{}""#, dt.to_rfc3339()));
    }

    #[test]
    fn aspnet_date_serialize_none() {
        let json = serde_json::to_string(&AspNetDate(None)).unwrap();
        assert_eq!(json, "null");
    }

    // -- FlexBool deserialize --

    #[test]
    fn flex_bool_true() {
        let b: FlexBool = serde_json::from_str("true").unwrap();
        assert_eq!(b, FlexBool(Some(true)));
    }

    #[test]
    fn flex_bool_false() {
        let b: FlexBool = serde_json::from_str("false").unwrap();
        assert_eq!(b, FlexBool(Some(false)));
    }

    #[test]
    fn flex_bool_one() {
        let b: FlexBool = serde_json::from_str("1").unwrap();
        assert_eq!(b, FlexBool(Some(true)));
    }

    #[test]
    fn flex_bool_zero() {
        let b: FlexBool = serde_json::from_str("0").unwrap();
        assert_eq!(b, FlexBool(Some(false)));
    }

    #[test]
    fn flex_bool_null() {
        let b: FlexBool = serde_json::from_str("null").unwrap();
        assert_eq!(b, FlexBool(None));
    }

    #[test]
    fn flex_bool_invalid_string() {
        let result: Result<FlexBool, _> = serde_json::from_str(r#""true""#);
        assert!(result.is_err());
    }

    #[test]
    fn flex_bool_invalid_number() {
        let result: Result<FlexBool, _> = serde_json::from_str("2");
        assert!(result.is_err());
    }

    // -- FlexBool serialize --

    #[test]
    fn flex_bool_serialize_true() {
        let json = serde_json::to_string(&FlexBool(Some(true))).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn flex_bool_serialize_false() {
        let json = serde_json::to_string(&FlexBool(Some(false))).unwrap();
        assert_eq!(json, "false");
    }

    #[test]
    fn flex_bool_serialize_none() {
        let json = serde_json::to_string(&FlexBool(None)).unwrap();
        assert_eq!(json, "null");
    }
}
