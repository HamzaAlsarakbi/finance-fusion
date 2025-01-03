use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize NaiveDateTime as an ISO8601 string
///
/// # Arguments
///
/// * `value` - The NaiveDateTime to serialize
/// * `serializer` - The serializer
///
/// # Returns
///
/// The serialized NaiveDateTime as an ISO8601 string
pub fn serialize<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = value.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&s)
}

// Deserialize NaiveDateTime from an ISO8601 string
pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_serialize() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let mut buf = Vec::new();
        serialize(&date, &mut serde_json::Serializer::new(&mut buf)).unwrap();
        let serialized = String::from_utf8(buf).unwrap();
        assert_eq!(serialized, "\"2021-01-01 00:00:00\"");
    }

    #[test]
    fn test_deserialize() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let deserialized = deserialize(&mut serde_json::Deserializer::from_str(
            "\"2021-01-01 00:00:00\"",
        ))
        .unwrap();
        assert_eq!(deserialized, date);
    }
}
