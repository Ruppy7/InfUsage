use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{io, time::Duration};

const WORKSPACE_BASE: &str = "https://opencode.ai/workspace";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// One usage window (rolling / weekly / monthly) from the OpenCode Go console.
///
/// Shape verified 2026-06-24 from the SSR'd `/workspace/{id}/go` page:
/// `{ status: "ok", resetInSec: 18000, usagePercent: 0 }`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UsageWindow {
    pub status: Option<String>,
    pub reset_in_sec: Option<i64>,
    pub usage_percent: Option<f64>,
}

/// Sanitized OpenCode Go subscription usage. No auth material — safe to hand to
/// the plugin sandbox and the UI.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UsageSummary {
    pub use_balance: Option<bool>,
    pub rolling: Option<UsageWindow>,
    pub weekly: Option<UsageWindow>,
    pub monthly: Option<UsageWindow>,
}

#[derive(Debug)]
pub enum OpenCodeError {
    Http(reqwest::Error),
    Io(io::Error),
    Json(serde_json::Error),
    MissingSession,
    Unauthorized,
    /// The page loaded but no usage block could be extracted — likely the
    /// console payload shape changed. Fail visibly (PLAN R3) rather than guess.
    UnexpectedShape,
}

impl From<reqwest::Error> for OpenCodeError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<io::Error> for OpenCodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for OpenCodeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for OpenCodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(formatter, "OpenCode HTTP error: {error}"),
            Self::Io(error) => write!(formatter, "OpenCode session error: {error}"),
            Self::Json(error) => write!(formatter, "OpenCode JSON error: {error}"),
            Self::MissingSession => {
                write!(formatter, "OpenCode console session is not connected")
            }
            Self::Unauthorized => {
                write!(formatter, "OpenCode session expired; sign in to the console again")
            }
            Self::UnexpectedShape => {
                write!(formatter, "OpenCode usage page did not contain a recognizable usage block")
            }
        }
    }
}

impl std::error::Error for OpenCodeError {}

/// Fetch and sanitize OpenCode Go usage for a workspace, authenticated by the
/// caller-supplied console session cookie (e.g. `auth=...; other=...`). The
/// cookie stays in the trusted host; only the sanitized summary JSON leaves.
pub fn fetch_usage_summary_json(
    session_cookie: &str,
    workspace_id: &str,
) -> Result<String, OpenCodeError> {
    if session_cookie.trim().is_empty() || workspace_id.trim().is_empty() {
        return Err(OpenCodeError::MissingSession);
    }

    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
    let url = format!("{WORKSPACE_BASE}/{workspace_id}/go");

    let response = client
        .get(url)
        .header(reqwest::header::COOKIE, session_cookie)
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .send()?;

    let status = response.status();
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        return Err(OpenCodeError::Unauthorized);
    }

    let body = response.error_for_status()?.text()?;
    let summary = parse_usage_from_page(&body)?;
    Ok(serde_json::to_string(&summary)?)
}

pub fn parse_usage_summary(json: &str) -> Result<UsageSummary, OpenCodeError> {
    Ok(serde_json::from_str(json)?)
}

/// Extract the usage block from the SSR'd console page. The data arrives as a
/// Seroval-serialized object literal embedded in the HTML, e.g.
/// `useBalance: !1, rollingUsage: $R[35] = { status: "ok", resetInSec: 18000, usagePercent: 0 }`.
/// We locate fields by their stable API names rather than parsing the whole
/// document, and fail visibly if none are found.
pub fn parse_usage_from_page(page: &str) -> Result<UsageSummary, OpenCodeError> {
    let rolling = window_for_label(page, "rollingUsage");
    let weekly = window_for_label(page, "weeklyUsage");
    let monthly = window_for_label(page, "monthlyUsage");
    let use_balance = bool_after(page, "useBalance");

    if rolling.is_none() && weekly.is_none() && monthly.is_none() {
        return Err(OpenCodeError::UnexpectedShape);
    }

    Ok(UsageSummary {
        use_balance,
        rolling,
        weekly,
        monthly,
    })
}

/// The three usage windows share inner field names, so bound each window's scan
/// to the slice between its own label and the next window label.
fn window_for_label(page: &str, label: &str) -> Option<UsageWindow> {
    const LABELS: [&str; 3] = ["rollingUsage", "weeklyUsage", "monthlyUsage"];

    let start = page.find(label)? + label.len();
    let end = LABELS
        .iter()
        .filter(|other| **other != label)
        .filter_map(|other| page[start..].find(other).map(|pos| start + pos))
        .min()
        .unwrap_or(page.len());
    let slice = &page[start..end];

    let window = UsageWindow {
        status: string_after(slice, "status"),
        reset_in_sec: number_after(slice, "resetInSec").and_then(|raw| raw.parse::<i64>().ok()),
        usage_percent: number_after(slice, "usagePercent").and_then(|raw| raw.parse::<f64>().ok()),
    };

    // Only treat the window as present if it carried at least one real field.
    if window == UsageWindow::default() {
        None
    } else {
        Some(window)
    }
}

/// Return the raw token following `key:` (whitespace-trimmed, up to the next
/// `,`, `}` or newline). Does not strip quotes.
fn raw_value_after<'a>(slice: &'a str, key: &str) -> Option<&'a str> {
    let pos = slice.find(key)? + key.len();
    let after = slice[pos..].trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let end = after
        .find(|c| c == ',' || c == '}' || c == '\n' || c == ';')
        .unwrap_or(after.len());
    Some(after[..end].trim())
}

fn number_after(slice: &str, key: &str) -> Option<String> {
    let raw = raw_value_after(slice, key)?;
    let number: String = raw
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    if number.is_empty() {
        None
    } else {
        Some(number)
    }
}

fn string_after(slice: &str, key: &str) -> Option<String> {
    let raw = raw_value_after(slice, key)?;
    let trimmed = raw.trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse a JS boolean that may be serialized as `true`/`false` or the minified
/// `!0` (true) / `!1` (false).
fn bool_after(slice: &str, key: &str) -> Option<bool> {
    match raw_value_after(slice, key)? {
        "true" | "!0" => Some(true),
        "false" | "!1" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The real captured payload (2026-06-24), Seroval-serialized in the page.
    const SAMPLE_PAGE: &str = r#"
        $R[28]($R[18], $R[34] = {
            mine: !0,
            useBalance: !1,
            rollingUsage: $R[35] = {
                status: "ok",
                resetInSec: 18000,
                usagePercent: 0
            },
            weeklyUsage: $R[36] = {
                status: "ok",
                resetInSec: 451207,
                usagePercent: 2
            },
            monthlyUsage: $R[37] = {
                status: "ok",
                resetInSec: 1194765,
                usagePercent: 9
            }
        });
    "#;

    #[test]
    fn parses_real_console_payload() {
        let summary = parse_usage_from_page(SAMPLE_PAGE).expect("sample should parse");

        assert_eq!(
            summary,
            UsageSummary {
                use_balance: Some(false),
                rolling: Some(UsageWindow {
                    status: Some("ok".to_string()),
                    reset_in_sec: Some(18000),
                    usage_percent: Some(0.0),
                }),
                weekly: Some(UsageWindow {
                    status: Some("ok".to_string()),
                    reset_in_sec: Some(451207),
                    usage_percent: Some(2.0),
                }),
                monthly: Some(UsageWindow {
                    status: Some("ok".to_string()),
                    reset_in_sec: Some(1194765),
                    usage_percent: Some(9.0),
                }),
            }
        );
    }

    #[test]
    fn parses_fractional_percent_and_balance_true() {
        let page = r#"useBalance: !0, weeklyUsage = { status: "ok", resetInSec: 100, usagePercent: 12.5 }"#;
        let summary = parse_usage_from_page(page).expect("should parse");

        assert_eq!(summary.use_balance, Some(true));
        let weekly = summary.weekly.expect("weekly present");
        assert_eq!(weekly.usage_percent, Some(12.5));
        assert_eq!(weekly.reset_in_sec, Some(100));
        assert!(summary.rolling.is_none());
        assert!(summary.monthly.is_none());
    }

    #[test]
    fn does_not_bleed_fields_across_windows() {
        // rollingUsage has no resetInSec; it must not borrow weekly's.
        let page = r#"rollingUsage = { status: "error" }, weeklyUsage = { status: "ok", resetInSec: 999, usagePercent: 5 }"#;
        let summary = parse_usage_from_page(page).expect("should parse");

        let rolling = summary.rolling.expect("rolling present");
        assert_eq!(rolling.status, Some("error".to_string()));
        assert_eq!(rolling.reset_in_sec, None);
        assert_eq!(summary.weekly.unwrap().reset_in_sec, Some(999));
    }

    #[test]
    fn fails_visibly_when_no_usage_block() {
        let error = parse_usage_from_page("<html>signed out</html>")
            .expect_err("missing usage block should error");
        assert!(matches!(error, OpenCodeError::UnexpectedShape));
    }

    #[test]
    fn round_trips_summary_json() {
        let summary = parse_usage_from_page(SAMPLE_PAGE).unwrap();
        let json = serde_json::to_string(&summary).unwrap();
        assert_eq!(parse_usage_summary(&json).unwrap(), summary);
    }

    #[test]
    fn rejects_empty_session() {
        assert!(matches!(
            fetch_usage_summary_json("", "wrk_123"),
            Err(OpenCodeError::MissingSession)
        ));
        assert!(matches!(
            fetch_usage_summary_json("auth=x", ""),
            Err(OpenCodeError::MissingSession)
        ));
    }
}
