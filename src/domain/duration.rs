use std::time::Duration;

use anyhow::{bail, Result};

/// Parse a human-readable duration string into `std::time::Duration`.
///
/// Supported formats: `"30s"`, `"15m"`, `"24h"`, `"7d"` (single unit).
pub fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty duration string");
    }

    let (digits, suffix) = split_number_suffix(s)?;
    let value: u64 = digits
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid number in duration: {s:?}"))?;

    let seconds = match suffix {
        "s" => Some(value),
        "m" => value.checked_mul(60),
        "h" => value.checked_mul(3600),
        "d" => value.checked_mul(86400),
        _ => bail!("unknown duration suffix {suffix:?} in {s:?}; expected s, m, h, or d"),
    };

    let seconds = seconds.ok_or_else(|| anyhow::anyhow!("duration overflow: {s:?}"))?;
    Ok(Duration::from_secs(seconds))
}

fn split_number_suffix(s: &str) -> Result<(&str, &str)> {
    let boundary = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| anyhow::anyhow!("missing unit suffix in duration: {s:?}"))?;
    let (digits, suffix) = s.split_at(boundary);
    if digits.is_empty() {
        bail!("missing number in duration: {s:?}");
    }
    Ok((digits, suffix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(parse_duration("15m").unwrap(), Duration::from_secs(900));
    }

    #[test]
    fn parse_hours() {
        assert_eq!(parse_duration("24h").unwrap(), Duration::from_secs(86400));
    }

    #[test]
    fn parse_days() {
        assert_eq!(parse_duration("7d").unwrap(), Duration::from_secs(604800));
    }

    #[test]
    fn parse_with_whitespace() {
        assert_eq!(parse_duration("  1h  ").unwrap(), Duration::from_secs(3600));
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn rejects_no_unit() {
        assert!(parse_duration("100").is_err());
    }

    #[test]
    fn rejects_no_number() {
        assert!(parse_duration("h").is_err());
    }

    #[test]
    fn rejects_unknown_unit() {
        assert!(parse_duration("5w").is_err());
    }
}
