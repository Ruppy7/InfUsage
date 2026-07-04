use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::util::{file_modified_at, non_empty_env_path, write_json_atomic};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const PROFILE_URL: &str = "https://api.anthropic.com/api/oauth/profile";
const REFRESH_URL: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const SCOPES: &str =
    "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const REFRESH_BUFFER_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ClaudeOauth {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<u64>,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
    scopes: Option<Vec<String>>,
}

#[derive(Debug)]
struct ClaudeAuth {
    path: PathBuf,
    modified_at: Option<SystemTime>,
    json: Value,
    oauth: ClaudeOauth,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UsageSummary {
    pub plan_type: Option<String>,
    pub session_remaining_percent: Option<f64>,
    pub session_reset_at: Option<String>,
    pub weekly_remaining_percent: Option<f64>,
    pub weekly_reset_at: Option<String>,
    pub fable_remaining_percent: Option<f64>,
    pub fable_reset_at: Option<String>,
}

#[derive(Debug)]
pub enum ClaudeError {
    Http(reqwest::Error),
    Io(io::Error),
    Json(serde_json::Error),
    MissingAuth,
    MissingTokens,
    RateLimited,
    Unauthorized,
}

impl From<reqwest::Error> for ClaudeError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<io::Error> for ClaudeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ClaudeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for ClaudeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(_) => write!(formatter, "Claude network request failed"),
            Self::Io(_) => write!(
                formatter,
                "Claude credentials file could not be read or updated"
            ),
            Self::Json(_) => write!(formatter, "Claude data could not be parsed"),
            Self::MissingAuth => write!(formatter, "Claude credentials were not found"),
            Self::MissingTokens => write!(
                formatter,
                "Claude credentials do not contain Claude Code OAuth tokens"
            ),
            Self::RateLimited => write!(formatter, "Claude is rate limited; try again later"),
            Self::Unauthorized => write!(formatter, "Claude login expired; run claude to sign in"),
        }
    }
}

impl std::error::Error for ClaudeError {}

pub fn fetch_usage_summary_json(cached_plan: Option<String>) -> Result<String, ClaudeError> {
    let mut auth = load_auth()?;
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
    let cached_plan = cached_plan.filter(|value| !value.trim().is_empty());

    if needs_refresh(&auth.oauth) {
        refresh_auth(&client, &mut auth)?;
    }

    if !has_profile_scope(&auth.oauth) {
        return Ok(serde_json::to_string(&UsageSummary {
            plan_type: plan_label(&auth.oauth).or(cached_plan),
            session_remaining_percent: None,
            session_reset_at: None,
            weekly_remaining_percent: None,
            weekly_reset_at: None,
            fable_remaining_percent: None,
            fable_reset_at: None,
        })?);
    }

    let profile_plan = plan_label(&auth.oauth)
        .or_else(|| cached_plan.clone())
        .or_else(|| fetch_profile_plan(&client, &auth.oauth).ok().flatten());

    match fetch_usage_summary(&client, &auth.oauth, profile_plan)? {
        FetchResult::Ok(summary) => Ok(serde_json::to_string(&summary)?),
        FetchResult::Unauthorized => {
            refresh_auth(&client, &mut auth)?;
            let profile_plan = plan_label(&auth.oauth)
                .or(cached_plan)
                .or_else(|| fetch_profile_plan(&client, &auth.oauth).ok().flatten());
            match fetch_usage_summary(&client, &auth.oauth, profile_plan)? {
                FetchResult::Ok(summary) => Ok(serde_json::to_string(&summary)?),
                FetchResult::Unauthorized => Err(ClaudeError::Unauthorized),
            }
        }
    }
}

enum FetchResult {
    Ok(UsageSummary),
    Unauthorized,
}

fn fetch_usage_summary(
    client: &Client,
    oauth: &ClaudeOauth,
    profile_plan: Option<String>,
) -> Result<FetchResult, ClaudeError> {
    let response = client
        .get(USAGE_URL)
        .bearer_auth(&oauth.access_token)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code/2.1.69")
        .send()?;
    let status = response.status();

    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        return Ok(FetchResult::Unauthorized);
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(ClaudeError::RateLimited);
    }

    let body = response.error_for_status()?.json::<Value>()?;

    Ok(FetchResult::Ok(summarize_usage(&body, oauth, profile_plan)))
}

fn fetch_profile_plan(client: &Client, oauth: &ClaudeOauth) -> Result<Option<String>, ClaudeError> {
    let response = client
        .get(PROFILE_URL)
        .bearer_auth(&oauth.access_token)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code/2.1.69")
        .send()?;

    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Ok(None);
    }
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(ClaudeError::RateLimited);
    }

    let body = response.error_for_status()?.json::<Value>()?;
    Ok(profile_plan_label(&body))
}

fn refresh_auth(client: &Client, auth: &mut ClaudeAuth) -> Result<(), ClaudeError> {
    let refresh_token = auth
        .oauth
        .refresh_token
        .as_ref()
        .ok_or(ClaudeError::Unauthorized)?;

    let response = client
        .post(REFRESH_URL)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": CLIENT_ID,
            "scope": SCOPES,
        }))
        .send()?;

    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::BAD_REQUEST
    ) {
        return Err(ClaudeError::Unauthorized);
    }
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(ClaudeError::RateLimited);
    }

    let refreshed = response.error_for_status()?.json::<Value>()?;
    let access_token = refreshed
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or(ClaudeError::MissingTokens)?
        .to_string();

    auth.json["claudeAiOauth"]["accessToken"] = Value::String(access_token);

    if let Some(refresh_token) = refreshed.get("refresh_token").and_then(Value::as_str) {
        auth.json["claudeAiOauth"]["refreshToken"] = Value::String(refresh_token.to_string());
    }

    if let Some(expires_in) = refreshed.get("expires_in").and_then(Value::as_u64) {
        auth.json["claudeAiOauth"]["expiresAt"] =
            Value::from(now_ms().saturating_add(expires_in * 1000));
    }

    auth.oauth = oauth_from_json(&auth.json)?;
    if auth_file_changed(auth)? {
        let json = serde_json::from_slice::<Value>(&fs::read(&auth.path)?)?;
        auth.oauth = oauth_from_json(&json)?;
        auth.json = json;
        auth.modified_at = file_modified_at(&auth.path);
        return Ok(());
    }

    write_json_atomic(&auth.path, &serde_json::to_vec(&auth.json)?)?;
    auth.modified_at = file_modified_at(&auth.path);

    Ok(())
}

fn load_auth() -> Result<ClaudeAuth, ClaudeError> {
    let path = auth_path().ok_or(ClaudeError::MissingAuth)?;

    if !path.exists() {
        return Err(ClaudeError::MissingAuth);
    }

    let modified_at = file_modified_at(&path);
    let json = serde_json::from_slice::<Value>(&fs::read(&path)?)?;
    let oauth = oauth_from_json(&json)?;

    Ok(ClaudeAuth {
        path,
        modified_at,
        json,
        oauth,
    })
}

fn auth_path() -> Option<PathBuf> {
    if let Some(path) = non_empty_env_path("CLAUDE_CREDENTIALS_FILE") {
        return Some(path);
    }

    let mut candidates = Vec::new();

    if let Some(config_dir) = non_empty_env_path("CLAUDE_CONFIG_DIR") {
        candidates.push(config_dir.join(".credentials.json"));
    }

    if let Some(home) = non_empty_env_path("USERPROFILE").or_else(|| non_empty_env_path("HOME")) {
        candidates.push(home.join(".claude").join(".credentials.json"));
    }

    newest_existing(candidates)
}

fn auth_file_changed(auth: &ClaudeAuth) -> Result<bool, ClaudeError> {
    Ok(file_modified_at(&auth.path) != auth.modified_at)
}

fn newest_existing(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates
        .into_iter()
        .filter(|path| path.exists())
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(UNIX_EPOCH)
        })
}

fn oauth_from_json(json: &Value) -> Result<ClaudeOauth, ClaudeError> {
    serde_json::from_value(
        json.get("claudeAiOauth")
            .cloned()
            .ok_or(ClaudeError::MissingTokens)?,
    )
    .map_err(|_| ClaudeError::MissingTokens)
}

fn needs_refresh(oauth: &ClaudeOauth) -> bool {
    oauth
        .expires_at
        .is_some_and(|expires_at| expires_at <= now_ms().saturating_add(REFRESH_BUFFER_MS))
}

fn has_profile_scope(oauth: &ClaudeOauth) -> bool {
    oauth
        .scopes
        .as_ref()
        .map(|scopes| scopes.iter().any(|scope| scope == "user:profile"))
        .unwrap_or(true)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn summarize_usage(
    body: &Value,
    oauth: &ClaudeOauth,
    profile_plan: Option<String>,
) -> UsageSummary {
    let session_used_percent = nested_f64(body, &["five_hour", "utilization"]);
    let weekly_used_percent = nested_f64(body, &["seven_day", "utilization"]);
    let fable_limit = find_named_limit(body, &["fable"]);

    UsageSummary {
        plan_type: plan_label(oauth).or(profile_plan),
        session_remaining_percent: session_used_percent.map(remaining_percent),
        session_reset_at: nested_string(body, &["five_hour", "resets_at"]),
        weekly_remaining_percent: weekly_used_percent.map(remaining_percent),
        weekly_reset_at: nested_string(body, &["seven_day", "resets_at"]),
        fable_remaining_percent: fable_limit.and_then(limit_remaining_percent),
        fable_reset_at: fable_limit.and_then(limit_reset_at),
    }
}

fn profile_plan_label(body: &Value) -> Option<String> {
    let organization = body.get("organization")?;
    let seat_tier = organization.get("seat_tier").and_then(Value::as_str);
    let organization_type = organization
        .get("organization_type")
        .and_then(Value::as_str);

    seat_tier
        .and_then(plan_from_tier)
        .or_else(|| organization_type.and_then(plan_from_tier))
}

fn plan_from_tier(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized.contains("team") {
        return Some("Team".to_string());
    }
    if normalized.contains("pro") {
        return Some("Pro".to_string());
    }
    if normalized.contains("max") {
        return Some("Max".to_string());
    }
    None
}

fn plan_label(oauth: &ClaudeOauth) -> Option<String> {
    let subscription = oauth.subscription_type.as_ref()?.trim();
    if subscription.is_empty() {
        return None;
    }

    let mut label = subscription.to_string();

    if let Some(tier) = oauth
        .rate_limit_tier
        .as_ref()
        .and_then(|tier| tier_suffix(tier))
    {
        label.push(' ');
        label.push_str(&tier);
    }

    Some(label)
}

fn tier_suffix(tier: &str) -> Option<String> {
    let start = tier.find(|character: char| character.is_ascii_digit())?;
    let end = tier[start..]
        .find('x')
        .map(|index| start + index + 1)
        .unwrap_or(tier.len());

    Some(tier[start..end].to_string())
}

fn remaining_percent(used_percent: f64) -> f64 {
    (100.0 - used_percent).clamp(0.0, 100.0)
}

fn nested_f64(value: &Value, path: &[&str]) -> Option<f64> {
    path.iter()
        .try_fold(value, |next, key| next.get(*key))
        .and_then(json_f64)
}

fn nested_string(value: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |next, key| next.get(*key))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn find_named_limit<'a>(value: &'a Value, aliases: &[&str]) -> Option<&'a Value> {
    match value {
        Value::Object(object) => {
            if value_contains_alias(value, aliases) && limit_remaining_percent(value).is_some() {
                return Some(value);
            }
            for (key, child) in object {
                if aliases
                    .iter()
                    .any(|alias| key.to_ascii_lowercase().contains(alias))
                    && limit_remaining_percent(child).is_some()
                {
                    return Some(child);
                }
                if object_name_matches(child, aliases) && limit_remaining_percent(child).is_some() {
                    return Some(child);
                }
                if let Some(found) = find_named_limit(child, aliases) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items
            .iter()
            .find_map(|item| find_named_limit(item, aliases)),
        _ => None,
    }
}

fn object_name_matches(value: &Value, aliases: &[&str]) -> bool {
    [
        "name",
        "model",
        "model_name",
        "modelName",
        "display_name",
        "displayName",
        "label",
        "id",
    ]
    .iter()
    .filter_map(|key| value.get(*key).and_then(Value::as_str))
    .any(|name| {
        let lower = name.to_ascii_lowercase();
        aliases.iter().any(|alias| lower.contains(alias))
    })
}

fn value_contains_alias(value: &Value, aliases: &[&str]) -> bool {
    object_name_matches(value, aliases)
        || match value {
            Value::Object(object) => object
                .values()
                .any(|child| value_contains_alias(child, aliases)),
            Value::Array(items) => items
                .iter()
                .any(|child| value_contains_alias(child, aliases)),
            Value::String(text) => {
                let lower = text.to_ascii_lowercase();
                aliases.iter().any(|alias| lower.contains(alias))
            }
            _ => false,
        }
}

fn limit_remaining_percent(value: &Value) -> Option<f64> {
    nested_f64(value, &["remaining_percent"])
        .or_else(|| nested_f64(value, &["remainingPercent"]))
        .or_else(|| nested_f64(value, &["remaining"]))
        .map(normalize_remaining_percent)
        .or_else(|| {
            nested_f64(value, &["utilization"])
                .or_else(|| nested_f64(value, &["used_percent"]))
                .or_else(|| nested_f64(value, &["usedPercent"]))
                .or_else(|| nested_f64(value, &["percent"]))
                .map(remaining_percent)
        })
}

fn normalize_remaining_percent(value: f64) -> f64 {
    if value <= 1.0 {
        (value * 100.0).clamp(0.0, 100.0)
    } else {
        value.clamp(0.0, 100.0)
    }
}

fn limit_reset_at(value: &Value) -> Option<String> {
    nested_string(value, &["resets_at"])
        .or_else(|| nested_string(value, &["reset_at"]))
        .or_else(|| nested_string(value, &["resetAt"]))
}

fn json_f64(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_usage_windows() {
        let oauth = ClaudeOauth {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
            subscription_type: Some("pro".to_string()),
            rate_limit_tier: Some("tier_5x".to_string()),
            scopes: None,
        };
        let body = serde_json::json!({
            "five_hour": {
                "utilization": 25,
                "resets_at": "2099-01-01T00:00:00.000Z"
            },
            "seven_day": {
                "utilization": "40",
                "resets_at": "2099-01-07T00:00:00.000Z"
            },
            "model_limits": {
                "claude_fable_5": {
                    "utilization": 20,
                    "resets_at": "2099-01-02T00:00:00.000Z"
                }
            }
        });

        assert_eq!(
            summarize_usage(&body, &oauth, None),
            UsageSummary {
                plan_type: Some("pro 5x".to_string()),
                session_remaining_percent: Some(75.0),
                session_reset_at: Some("2099-01-01T00:00:00.000Z".to_string()),
                weekly_remaining_percent: Some(60.0),
                weekly_reset_at: Some("2099-01-07T00:00:00.000Z".to_string()),
                fable_remaining_percent: Some(80.0),
                fable_reset_at: Some("2099-01-02T00:00:00.000Z".to_string()),
            }
        );
    }

    #[test]
    fn finds_fable_limit_from_model_array() {
        let oauth = ClaudeOauth {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
            subscription_type: None,
            rate_limit_tier: None,
            scopes: None,
        };
        let body = serde_json::json!({
            "limits": [
                { "model": "claude-sonnet", "remaining_percent": 90 },
                { "modelName": "Claude Fable 5", "remaining": 0.42, "resetAt": "2099-01-03T00:00:00.000Z" }
            ]
        });

        let summary = summarize_usage(&body, &oauth, None);

        assert_eq!(summary.fable_remaining_percent, Some(42.0));
        assert_eq!(
            summary.fable_reset_at,
            Some("2099-01-03T00:00:00.000Z".to_string())
        );
    }

    #[test]
    fn finds_fable_limit_from_scoped_weekly_limit() {
        let oauth = ClaudeOauth {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
            subscription_type: None,
            rate_limit_tier: None,
            scopes: None,
        };
        let body = serde_json::json!({
            "limits": [
                {
                    "kind": "weekly_all",
                    "group": "weekly",
                    "percent": 3,
                    "resets_at": "2099-01-07T20:00:00.000Z",
                    "scope": null
                },
                {
                    "kind": "weekly_scoped",
                    "group": "weekly",
                    "percent": 2,
                    "resets_at": "2099-01-07T20:00:00.000Z",
                    "scope": {
                        "model": {
                            "id": null,
                            "display_name": "Fable"
                        },
                        "surface": null
                    }
                }
            ]
        });

        let summary = summarize_usage(&body, &oauth, None);

        assert_eq!(summary.fable_remaining_percent, Some(98.0));
        assert_eq!(
            summary.fable_reset_at,
            Some("2099-01-07T20:00:00.000Z".to_string())
        );
    }

    #[test]
    fn uses_profile_plan_when_credentials_do_not_have_plan() {
        let oauth = ClaudeOauth {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
            subscription_type: None,
            rate_limit_tier: None,
            scopes: None,
        };
        let body = serde_json::json!({});

        assert_eq!(
            summarize_usage(&body, &oauth, Some("Team".to_string())).plan_type,
            Some("Team".to_string())
        );
    }

    #[test]
    fn maps_claude_profile_tier_to_plan() {
        let body = serde_json::json!({
            "organization": {
                "organization_type": "claude_team",
                "seat_tier": "team_standard",
                "rate_limit_tier": "default_raven"
            }
        });

        assert_eq!(profile_plan_label(&body), Some("Team".to_string()));
    }

    #[test]
    fn respects_missing_profile_scope() {
        let oauth = ClaudeOauth {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
            subscription_type: None,
            rate_limit_tier: None,
            scopes: Some(vec!["user:inference".to_string()]),
        };

        assert!(!has_profile_scope(&oauth));
    }
}
