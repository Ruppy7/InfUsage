use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command, time::Duration};
#[cfg(windows)]
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);
const USER_STATUS_METHOD: &str = "GetUserStatus";
const COMMAND_CONFIGS_METHOD: &str = "GetCommandModelConfigs";
const CLOUD_CODE_BASE_URLS: &[&str] = &[
    "https://daily-cloudcode-pa.googleapis.com",
    "https://cloudcode-pa.googleapis.com",
];
const FETCH_MODELS_PATH: &str = "/v1internal:fetchAvailableModels";
const LOAD_CODE_ASSIST_PATH: &str = "/v1internal:loadCodeAssist";
const RETRIEVE_QUOTA_PATH: &str = "/v1internal:retrieveUserQuota";
const GOOGLE_OAUTH_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CLIENT_ID_PARTS: &[&str] = &[
    "107100",
    "6060591-",
    "tmhssin2h21lcre",
    "235vtolojh4g403ep",
    ".apps.google",
    "usercontent.com",
];
const GOOGLE_CLIENT_SECRET_PARTS: &[&str] =
    &["GO", "CSP", "X-", "K58FWR486", "LdLJ1mLB8s", "XC4z6qDAf"];
const KEYRING_SERVICE: &str = "gemini";
const KEYRING_ACCOUNT: &str = "antigravity";
const WINDOWS_LEGACY_TARGET: &str = "gemini:antigravity";
const MODEL_BLACKLIST: &[&str] = &[
    "MODEL_CHAT_20706",
    "MODEL_CHAT_23310",
    "MODEL_GOOGLE_GEMINI_2_5_FLASH",
    "MODEL_GOOGLE_GEMINI_2_5_FLASH_THINKING",
    "MODEL_GOOGLE_GEMINI_2_5_FLASH_LITE",
    "MODEL_GOOGLE_GEMINI_2_5_PRO",
    "MODEL_PLACEHOLDER_M19",
    "MODEL_PLACEHOLDER_M9",
    "MODEL_PLACEHOLDER_M12",
];

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UsageSummary {
    pub plan_type: Option<String>,
    pub pools: Vec<UsagePool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsagePool {
    pub label: String,
    pub remaining_percent: f64,
    pub reset_at: Option<String>,
}

#[derive(Debug)]
pub enum AntigravityError {
    Discovery(String),
    Http(reqwest::Error),
    Json(serde_json::Error),
    NotRunning,
    AuthExpired,
    Unavailable,
}

impl From<reqwest::Error> for AntigravityError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<serde_json::Error> for AntigravityError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for AntigravityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discovery(_) => write!(formatter, "Antigravity process discovery failed"),
            Self::Http(_) => write!(formatter, "Antigravity local request failed"),
            Self::Json(_) => write!(formatter, "Antigravity usage data could not be parsed"),
            Self::NotRunning => write!(formatter, "Start Antigravity or run `agy` and try again"),
            Self::AuthExpired => write!(
                formatter,
                "Antigravity sign-in expired; open Antigravity or run agy to refresh"
            ),
            Self::Unavailable => write!(
                formatter,
                "Antigravity usage is temporarily unavailable; try again shortly"
            ),
        }
    }
}

impl std::error::Error for AntigravityError {}

#[derive(Debug, Clone, PartialEq)]
struct ProcessRow {
    pid: u32,
    name: String,
    command_line: String,
}

#[derive(Debug, Clone, PartialEq)]
struct LanguageServer {
    csrf: String,
    ports: Vec<u16>,
    extension_port: Option<u16>,
}

#[derive(Debug, Clone)]
struct ModelConfig {
    label: String,
    model_id: Option<String>,
    remaining_fraction: f64,
    reset_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct AuthToken {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

enum CloudCodeOutcome {
    Ok(String),
    AuthFailed,
    Unavailable,
}

enum TokenRefreshOutcome {
    Refreshed { access_token: String },
    AuthFailed,
    Unavailable,
}

pub fn fetch_usage_summary_json() -> Result<String, AntigravityError> {
    let ls_https_client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .danger_accept_invalid_certs(true)
        .build()?;
    let ls_http_client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
    let cloud_client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

    let servers = match discover_language_servers() {
        Ok(servers) => servers,
        Err(error) => {
            eprintln!("Antigravity language-server discovery failed: {error}");
            Vec::new()
        }
    };

    for server in servers {
        if let Some(summary) = fetch_from_server(&ls_https_client, &ls_http_client, &server)? {
            return Ok(serde_json::to_string(&summary)?);
        }
    }

    let summary = fetch_from_cloud_code(&cloud_client)?;
    Ok(serde_json::to_string(&summary)?)
}

fn fetch_from_server(
    https_client: &Client,
    http_client: &Client,
    server: &LanguageServer,
) -> Result<Option<UsageSummary>, AntigravityError> {
    for endpoint in endpoints(server) {
        let client = if endpoint.starts_with("https://") {
            https_client
        } else {
            http_client
        };
        if let Some(body) =
            post_language_server(client, &endpoint, &server.csrf, USER_STATUS_METHOD)?
        {
            if let Ok(Some(summary)) = parse_user_status_json(&body) {
                if !summary.pools.is_empty() {
                    return Ok(Some(summary));
                }
            }
        }

        if let Some(body) =
            post_language_server(client, &endpoint, &server.csrf, COMMAND_CONFIGS_METHOD)?
        {
            if let Ok(Some(configs)) = parse_command_model_configs_json(&body) {
                let summary = UsageSummary {
                    plan_type: None,
                    pools: build_pools(&configs),
                };
                if !summary.pools.is_empty() {
                    return Ok(Some(summary));
                }
            }
        }
    }

    Ok(None)
}

fn post_language_server(
    client: &Client,
    endpoint: &str,
    csrf: &str,
    method: &str,
) -> Result<Option<String>, AntigravityError> {
    let response = client
        .post(format!(
            "{endpoint}/exa.language_server_pb.LanguageServerService/{method}"
        ))
        .header("Content-Type", "application/json")
        .header("Connect-Protocol-Version", "1")
        .header("x-codeium-csrf-token", csrf)
        .json(&serde_json::json!({
            "metadata": {
                "ideName": "antigravity",
                "extensionName": "antigravity",
                "ideVersion": "unknown",
                "locale": "en"
            }
        }))
        .send();

    let Ok(response) = response else {
        return Ok(None);
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    Ok(Some(response.text()?))
}

fn fetch_from_cloud_code(client: &Client) -> Result<UsageSummary, AntigravityError> {
    let keyring_token = load_keyring_token();
    let mut tokens = Vec::new();

    if let Some(access_token) = keyring_token
        .as_ref()
        .and_then(|token| token.access_token.as_ref())
        .filter(|token| !token.trim().is_empty())
    {
        tokens.push(access_token.clone());
    }
    let has_credentials = !tokens.is_empty()
        || keyring_token
            .as_ref()
            .and_then(|token| token.refresh_token.as_ref())
            .is_some_and(|token| !token.trim().is_empty());

    let mut saw_auth_failure = false;
    for token in &tokens {
        match fetch_cloud_code_with_token(client, token)? {
            CloudCodeProbe::Success(summary) => return Ok(summary),
            CloudCodeProbe::AuthFailed => saw_auth_failure = true,
            CloudCodeProbe::Unavailable => {}
        }
    }

    if (saw_auth_failure || tokens.is_empty())
        && keyring_token
            .as_ref()
            .and_then(|token| token.refresh_token.as_ref())
            .is_some()
    {
        let refresh_token = keyring_token
            .as_ref()
            .and_then(|token| token.refresh_token.as_ref())
            .expect("checked above");
        return match refresh_google_token(client, refresh_token) {
            TokenRefreshOutcome::Refreshed { access_token } => {
                match fetch_cloud_code_with_token(client, &access_token)? {
                    CloudCodeProbe::Success(summary) => Ok(summary),
                    CloudCodeProbe::AuthFailed => Err(AntigravityError::AuthExpired),
                    CloudCodeProbe::Unavailable => Err(AntigravityError::Unavailable),
                }
            }
            TokenRefreshOutcome::AuthFailed => Err(AntigravityError::AuthExpired),
            TokenRefreshOutcome::Unavailable => Err(AntigravityError::Unavailable),
        };
    }

    if saw_auth_failure {
        return Err(AntigravityError::AuthExpired);
    }
    if has_credentials {
        return Err(AntigravityError::Unavailable);
    }
    Err(AntigravityError::NotRunning)
}

enum CloudCodeProbe {
    Success(UsageSummary),
    AuthFailed,
    Unavailable,
}

fn fetch_cloud_code_with_token(
    client: &Client,
    token: &str,
) -> Result<CloudCodeProbe, AntigravityError> {
    match cloud_code(
        client,
        FETCH_MODELS_PATH,
        token,
        "antigravity",
        &serde_json::json!({}),
    )? {
        CloudCodeOutcome::AuthFailed => return Ok(CloudCodeProbe::AuthFailed),
        CloudCodeOutcome::Ok(body) => {
            let pools = build_pools(&parse_cloud_code_models_json(&body)?);
            if !pools.is_empty() {
                return Ok(CloudCodeProbe::Success(UsageSummary {
                    plan_type: load_cloud_code_plan(client, token)?,
                    pools,
                }));
            }
        }
        CloudCodeOutcome::Unavailable => {}
    }

    let mut plan = None;
    let mut project = None;
    match cloud_code(
        client,
        LOAD_CODE_ASSIST_PATH,
        token,
        "agy",
        &serde_json::json!({}),
    )? {
        CloudCodeOutcome::AuthFailed => return Ok(CloudCodeProbe::AuthFailed),
        CloudCodeOutcome::Ok(body) => {
            let load = parse_load_code_assist_json(&body)?;
            plan = load.plan_type;
            project = load.project;
        }
        CloudCodeOutcome::Unavailable => {}
    }

    let quota_body = project
        .as_ref()
        .map(|project| serde_json::json!({ "project": project }))
        .unwrap_or_else(|| serde_json::json!({}));
    let mut quota = cloud_code(client, RETRIEVE_QUOTA_PATH, token, "agy", &quota_body)?;
    if matches!(quota, CloudCodeOutcome::Unavailable) && project.is_some() {
        quota = cloud_code(
            client,
            RETRIEVE_QUOTA_PATH,
            token,
            "agy",
            &serde_json::json!({}),
        )?;
    }

    match quota {
        CloudCodeOutcome::AuthFailed => Ok(CloudCodeProbe::AuthFailed),
        CloudCodeOutcome::Ok(body) => {
            let pools = build_pools(&parse_quota_buckets_json(&body)?);
            if pools.is_empty() {
                Ok(CloudCodeProbe::Unavailable)
            } else {
                Ok(CloudCodeProbe::Success(UsageSummary {
                    plan_type: plan,
                    pools,
                }))
            }
        }
        CloudCodeOutcome::Unavailable => Ok(CloudCodeProbe::Unavailable),
    }
}

fn load_cloud_code_plan(client: &Client, token: &str) -> Result<Option<String>, AntigravityError> {
    match cloud_code(
        client,
        LOAD_CODE_ASSIST_PATH,
        token,
        "agy",
        &serde_json::json!({}),
    )? {
        CloudCodeOutcome::Ok(body) => Ok(parse_load_code_assist_json(&body)?.plan_type),
        CloudCodeOutcome::AuthFailed | CloudCodeOutcome::Unavailable => Ok(None),
    }
}

fn cloud_code(
    client: &Client,
    path: &str,
    token: &str,
    user_agent: &str,
    body: &serde_json::Value,
) -> Result<CloudCodeOutcome, AntigravityError> {
    for base in CLOUD_CODE_BASE_URLS {
        let response = client
            .post(format!("{base}{path}"))
            .bearer_auth(token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent)
            .json(body)
            .send();

        let Ok(response) = response else {
            continue;
        };

        if matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            return Ok(CloudCodeOutcome::AuthFailed);
        }
        if response.status().is_success() {
            return Ok(CloudCodeOutcome::Ok(response.text()?));
        }
    }
    Ok(CloudCodeOutcome::Unavailable)
}

fn refresh_google_token(client: &Client, refresh_token: &str) -> TokenRefreshOutcome {
    let client_id = GOOGLE_CLIENT_ID_PARTS.concat();
    let client_secret = GOOGLE_CLIENT_SECRET_PARTS.concat();
    let response = client
        .post(GOOGLE_OAUTH_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send();

    let Ok(response) = response else {
        return TokenRefreshOutcome::Unavailable;
    };

    match response.status() {
        status if status.is_success() => {
            let Ok(body) = response.json::<GoogleTokenResponse>() else {
                return TokenRefreshOutcome::Unavailable;
            };
            match body.access_token.filter(|token| !token.trim().is_empty()) {
                Some(access_token) => TokenRefreshOutcome::Refreshed { access_token },
                None => TokenRefreshOutcome::Unavailable,
            }
        }
        StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS => {
            TokenRefreshOutcome::Unavailable
        }
        status if status.is_client_error() => TokenRefreshOutcome::AuthFailed,
        _ => TokenRefreshOutcome::Unavailable,
    }
}

fn endpoints(server: &LanguageServer) -> Vec<String> {
    let mut endpoints = Vec::new();
    for port in &server.ports {
        endpoints.push(format!("https://127.0.0.1:{port}"));
        endpoints.push(format!("http://127.0.0.1:{port}"));
    }
    if let Some(port) = server.extension_port {
        endpoints.push(format!("http://127.0.0.1:{port}"));
    }
    endpoints.dedup();
    endpoints
}

fn discover_language_servers() -> Result<Vec<LanguageServer>, AntigravityError> {
    let process_rows = read_windows_process_rows()?;
    let mut servers = Vec::new();

    if let Some((pid, command)) = ranked_candidate(
        &process_rows,
        "language_server",
        &["antigravity", "antigravity-ide"],
    ) {
        if let Some(csrf) = extract_flag(&command, "--csrf_token") {
            let ports = listening_ports(pid)?;
            let extension_port = extract_flag(&command, "--extension_server_port")
                .and_then(|value| value.parse::<u16>().ok());
            if !ports.is_empty() || extension_port.is_some() {
                servers.push(LanguageServer {
                    csrf,
                    ports,
                    extension_port,
                });
            }
        }
    }

    if let Some((pid, _command)) = ranked_candidate(&process_rows, "agy", &[]) {
        let ports = listening_ports(pid)?;
        if !ports.is_empty() {
            servers.push(LanguageServer {
                csrf: String::new(),
                ports,
                extension_port: None,
            });
        }
    }

    Ok(servers)
}

fn read_windows_process_rows() -> Result<Vec<ProcessRow>, AntigravityError> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$ErrorActionPreference='Stop'; Get-CimInstance Win32_Process | Where-Object { $_.Name -in @('language_server.exe','language_server','agy.exe','agy') } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|error| AntigravityError::Discovery(error.to_string()))?;

    if !output.status.success() {
        return Err(AntigravityError::Discovery(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    parse_process_rows(&String::from_utf8_lossy(&output.stdout))
}

fn parse_process_rows(input: &str) -> Result<Vec<ProcessRow>, AntigravityError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let value: serde_json::Value = serde_json::from_str(trimmed)?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        item => vec![item],
    };

    Ok(items
        .into_iter()
        .filter_map(|item| {
            let pid = item.get("ProcessId")?.as_u64()? as u32;
            let name = item.get("Name")?.as_str()?.to_string();
            let command_line = item
                .get("CommandLine")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            Some(ProcessRow {
                pid,
                name,
                command_line,
            })
        })
        .collect())
}

fn ranked_candidate(
    rows: &[ProcessRow],
    process_name: &str,
    markers: &[&str],
) -> Option<(u32, String)> {
    let process_name = process_name.to_lowercase();
    let markers = markers
        .iter()
        .map(|marker| marker.to_lowercase())
        .collect::<Vec<_>>();

    let mut ranked = rows
        .iter()
        .filter_map(|row| {
            let command = if row.command_line.trim().is_empty() {
                row.name.as_str()
            } else {
                row.command_line.as_str()
            };
            if !command_matches_process(command, &row.name, &process_name) {
                return None;
            }
            marker_rank(command, &markers).map(|rank| (rank, row.pid, command.to_string()))
        })
        .collect::<Vec<_>>();

    ranked.sort_by_key(|(rank, _, _)| *rank);
    ranked
        .into_iter()
        .next()
        .map(|(_, pid, command)| (pid, command))
}

fn command_matches_process(command: &str, name: &str, process_name: &str) -> bool {
    let exe = argv0(command)
        .split(['\\', '/'])
        .next_back()
        .unwrap_or("")
        .to_lowercase();
    let exe = trim_exe_suffix(&exe);
    let process_name = trim_exe_suffix(process_name);
    let name_lower = name.to_lowercase();
    let name = trim_exe_suffix(&name_lower);

    if exe == process_name || name == process_name {
        return true;
    }
    command.to_lowercase().contains(process_name)
}

fn trim_exe_suffix(value: &str) -> &str {
    value.strip_suffix(".exe").unwrap_or(value)
}

fn marker_rank(command: &str, markers: &[String]) -> Option<u8> {
    if markers.is_empty() {
        return Some(0);
    }

    let ide_name = extract_flag(command, "--ide_name").map(|value| value.to_lowercase());
    let override_ide_name =
        extract_flag(command, "--override_ide_name").map(|value| value.to_lowercase());
    let app_data = extract_flag(command, "--app_data_dir").map(|value| value.to_lowercase());
    if ide_name.is_some() || override_ide_name.is_some() || app_data.is_some() {
        let matches = markers.iter().any(|marker| {
            ide_name.as_deref() == Some(marker.as_str())
                || override_ide_name.as_deref() == Some(marker.as_str())
                || app_data.as_deref() == Some(marker.as_str())
        });
        return matches.then_some(0);
    }

    let normalized = command.replace('\\', "/").to_lowercase();
    markers
        .iter()
        .any(|marker| normalized.contains(&format!("/{marker}/")))
        .then_some(1)
}

fn argv0(command: &str) -> &str {
    let trimmed = command.trim_start();
    let Some(quote) = trimmed
        .chars()
        .next()
        .filter(|char| *char == '"' || *char == '\'')
    else {
        return trimmed.split_whitespace().next().unwrap_or("");
    };
    let rest = &trimmed[quote.len_utf8()..];
    rest.find(quote)
        .map(|index| &rest[..index])
        .unwrap_or(trimmed)
}

fn extract_flag(command: &str, flag: &str) -> Option<String> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let flag_eq = format!("{flag}=");
    for (index, part) in parts.iter().enumerate() {
        if *part == flag {
            return parts.get(index + 1).map(|value| (*value).to_string());
        }
        if let Some(value) = part.strip_prefix(&flag_eq) {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn listening_ports(pid: u32) -> Result<Vec<u16>, AntigravityError> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
        .map_err(|error| AntigravityError::Discovery(error.to_string()))?;

    if !output.status.success() {
        return Err(AntigravityError::Discovery(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(parse_netstat_ports(
        &String::from_utf8_lossy(&output.stdout),
        pid,
    ))
}

fn parse_netstat_ports(output: &str, pid: u32) -> Vec<u16> {
    let mut ports = output
        .lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 5 || !parts[0].eq_ignore_ascii_case("TCP") {
                return None;
            }
            if !parts[3].eq_ignore_ascii_case("LISTENING") || parts[4] != pid.to_string() {
                return None;
            }
            parts[1]
                .rsplit_once(':')
                .and_then(|(_, port)| port.parse::<u16>().ok())
        })
        .collect::<Vec<_>>();
    ports.sort_unstable();
    ports.dedup();
    ports
}

fn parse_user_status_json(input: &str) -> Result<Option<UsageSummary>, AntigravityError> {
    let envelope = serde_json::from_str::<LSUserStatusEnvelope>(input)?;
    let Some(status) = envelope.user_status else {
        return Ok(None);
    };

    let plan = status
        .user_tier
        .and_then(|tier| tier.name)
        .or_else(|| {
            status
                .plan_status
                .and_then(|status| status.plan_info)
                .and_then(|info| info.plan_name)
        })
        .and_then(format_plan);

    let configs = status
        .cascade_model_config_data
        .and_then(|data| data.client_model_configs)
        .unwrap_or_default()
        .into_iter()
        .filter_map(config_from_ls)
        .collect::<Vec<_>>();

    Ok(Some(UsageSummary {
        plan_type: plan,
        pools: build_pools(&configs),
    }))
}

fn parse_command_model_configs_json(
    input: &str,
) -> Result<Option<Vec<ModelConfig>>, AntigravityError> {
    let envelope = serde_json::from_str::<LSCommandConfigsEnvelope>(input)?;
    Ok(envelope.client_model_configs.map(|configs| {
        configs
            .into_iter()
            .filter_map(config_from_ls)
            .collect::<Vec<_>>()
    }))
}

fn parse_cloud_code_models_json(input: &str) -> Result<Vec<ModelConfig>, AntigravityError> {
    let envelope = serde_json::from_str::<CCModelsEnvelope>(input)?;
    Ok(envelope
        .models
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, model)| {
            if model.is_internal == Some(true) {
                return None;
            }
            let label = model
                .display_name
                .or(model.label)
                .map(|label| label.trim().to_string())
                .filter(|label| !label.is_empty())?;
            Some(ModelConfig {
                label,
                model_id: model
                    .model
                    .filter(|value| !value.trim().is_empty())
                    .or(Some(key)),
                remaining_fraction: model
                    .quota_info
                    .as_ref()
                    .and_then(|quota| quota.remaining_fraction)
                    .unwrap_or(0.0),
                reset_at: model.quota_info.and_then(|quota| quota.reset_time),
            })
        })
        .collect())
}

fn parse_quota_buckets_json(input: &str) -> Result<Vec<ModelConfig>, AntigravityError> {
    let envelope = serde_json::from_str::<CCQuotaEnvelope>(input)?;
    Ok(envelope
        .buckets
        .unwrap_or_default()
        .into_iter()
        .filter_map(|bucket| {
            let id = bucket.model_id?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            Some(ModelConfig {
                label: id.clone(),
                model_id: Some(id),
                remaining_fraction: bucket.remaining_fraction.unwrap_or(0.0),
                reset_at: bucket.reset_time,
            })
        })
        .collect())
}

#[derive(Debug, PartialEq)]
struct LoadCodeAssist {
    plan_type: Option<String>,
    project: Option<String>,
}

fn parse_load_code_assist_json(input: &str) -> Result<LoadCodeAssist, AntigravityError> {
    let envelope = serde_json::from_str::<CCLoadEnvelope>(input)?;
    Ok(LoadCodeAssist {
        plan_type: envelope
            .paid_tier
            .and_then(|tier| tier.name)
            .or_else(|| envelope.current_tier.and_then(|tier| tier.name))
            .and_then(format_plan),
        project: envelope
            .cloudaicompanion_project
            .map(|project| project.trim().to_string())
            .filter(|project| !project.is_empty()),
    })
}

fn config_from_ls(model: LSModelConfig) -> Option<ModelConfig> {
    let label = model.label?.trim().to_string();
    if label.is_empty() {
        return None;
    }
    let quota = model.quota_info;
    Some(ModelConfig {
        label,
        model_id: model.model_or_alias.and_then(|model| model.model),
        remaining_fraction: quota
            .as_ref()
            .and_then(|quota| quota.remaining_fraction)
            .unwrap_or(0.0),
        reset_at: quota.and_then(|quota| quota.reset_time),
    })
}

fn build_pools(configs: &[ModelConfig]) -> Vec<UsagePool> {
    let mut pooled: HashMap<&'static str, (f64, Option<String>)> = HashMap::new();

    for config in configs {
        if config
            .model_id
            .as_ref()
            .is_some_and(|id| MODEL_BLACKLIST.contains(&id.as_str()))
        {
            continue;
        }

        let label = normalize_label(&config.label);
        if label.is_empty() {
            continue;
        }
        let pool = pool_label(&label);
        let fraction = config.remaining_fraction.clamp(0.0, 1.0);
        match pooled.get(pool) {
            Some((existing, _)) if *existing <= fraction => {}
            _ => {
                pooled.insert(pool, (fraction, config.reset_at.clone()));
            }
        }
    }

    ["Gemini Pro", "Gemini Flash", "Claude"]
        .into_iter()
        .filter_map(|label| {
            pooled.get(label).map(|(fraction, reset_at)| UsagePool {
                label: label.to_string(),
                remaining_percent: (fraction * 100.0).round(),
                reset_at: reset_at.clone(),
            })
        })
        .collect()
}

fn normalize_label(label: &str) -> String {
    let trimmed = label.trim();
    if let Some(index) = trimmed.rfind(" (") {
        if trimmed.ends_with(')') {
            return trimmed[..index].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn pool_label(label: &str) -> &'static str {
    let lower = label.to_lowercase();
    if lower.contains("gemini") {
        return if lower.contains("flash") {
            "Gemini Flash"
        } else {
            "Gemini Pro"
        };
    }
    "Claude"
}

fn format_plan(raw: String) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(tail) = trimmed.strip_prefix("Google AI ") {
        return Some(title_case_words(tail));
    }
    for keyword in ["Ultra", "Pro", "Free"] {
        if trimmed.to_lowercase().contains(&keyword.to_lowercase()) {
            return Some(keyword.to_string());
        }
    }
    Some(title_case_words(trimmed))
}

fn title_case_words(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_keyring_token() -> Option<AuthToken> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .ok()
        .and_then(|entry| entry.get_password().ok())
        .or_else(load_windows_legacy_credential)
        .and_then(|raw| extract_token_from_keyring_raw(&raw))
}

#[cfg(windows)]
fn load_windows_legacy_credential() -> Option<String> {
    // `agy` writes a legacy generic credential visible as `gemini:antigravity` in
    // Windows Credential Manager. The Rust keyring crate does not read that
    // target shape, so read the known target directly as a fallback.
    const CRED_TYPE_GENERIC: u32 = 1;

    #[repr(C)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        credential_type: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut std::ffi::c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    extern "system" {
        fn CredReadW(
            target_name: *const u16,
            credential_type: u32,
            flags: u32,
            credential: *mut *mut CredentialW,
        ) -> i32;
        fn CredFree(buffer: *mut std::ffi::c_void);
    }

    let target = OsStr::new(WINDOWS_LEGACY_TARGET)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut credential = ptr::null_mut();

    let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    if ok == 0 || credential.is_null() {
        return None;
    }

    let raw = unsafe {
        let credential_ref = &*credential;
        let size = credential_ref.credential_blob_size as usize;
        let blob = credential_ref.credential_blob;
        if blob.is_null() || size == 0 {
            None
        } else {
            let bytes = std::slice::from_raw_parts(blob, size);
            decode_windows_credential_blob(bytes)
        }
    };

    unsafe { CredFree(credential.cast()) };
    raw
}

#[cfg(windows)]
fn decode_windows_credential_blob(bytes: &[u8]) -> Option<String> {
    let looks_utf16le = bytes.len() >= 2
        && bytes.len() % 2 == 0
        && bytes.chunks_exact(2).all(|chunk| chunk[1] == 0);

    if looks_utf16le {
        let words = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|word| *word != 0)
            .collect::<Vec<_>>();
        return String::from_utf16(&words).ok();
    }

    String::from_utf8(bytes.to_vec()).ok()
}

#[cfg(not(windows))]
fn load_windows_legacy_credential() -> Option<String> {
    None
}

fn extract_token_from_keyring_raw(raw: &str) -> Option<AuthToken> {
    let text = unwrap_go_keyring(raw)?;
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(token) = token_from_json_value(&value) {
            return Some(token);
        }
        if let Some(token) = value
            .as_str()
            .map(str::to_string)
            .and_then(non_empty_string)
        {
            return Some(AuthToken {
                access_token: Some(token),
                refresh_token: None,
            });
        }
    }

    let token = text
        .strip_prefix("Bearer ")
        .unwrap_or(&text)
        .trim()
        .to_string();
    non_empty_string(token).map(|token| AuthToken {
        access_token: Some(token),
        refresh_token: None,
    })
}

fn unwrap_go_keyring(raw: &str) -> Option<String> {
    let text = raw.trim();
    let decoded = if let Some(encoded) = text.strip_prefix("go-keyring-base64:") {
        let bytes = general_purpose::STANDARD.decode(encoded.trim()).ok()?;
        String::from_utf8(bytes).ok()?
    } else {
        text.to_string()
    };
    non_empty_string(decoded.trim().to_string())
}

fn token_from_json_value(value: &serde_json::Value) -> Option<AuthToken> {
    let source = value.get("token").unwrap_or(value);
    let access_token = first_string(
        source,
        &[
            "access_token",
            "accessToken",
            "token",
            "id_token",
            "idToken",
            "bearerToken",
            "auth_token",
            "authToken",
        ],
    );
    let refresh_token = first_string(source, &["refresh_token", "refreshToken"]);

    if access_token.is_some() || refresh_token.is_some() {
        return Some(AuthToken {
            access_token,
            refresh_token,
        });
    }

    for key in ["tokens", "oauth", "oauth2", "credentials", "auth"] {
        if let Some(token) = value.get(key).and_then(token_from_json_value) {
            return Some(token);
        }
    }
    None
}

fn first_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|value| value.as_str()))
        .map(|value| value.trim().to_string())
        .and_then(non_empty_string)
}

fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AntigravityQuotaInfo {
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LSModelConfig {
    label: Option<String>,
    model_or_alias: Option<ModelOrAlias>,
    quota_info: Option<AntigravityQuotaInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelOrAlias {
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LSUserStatusEnvelope {
    user_status: Option<LSUserStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LSUserStatus {
    user_tier: Option<Tier>,
    plan_status: Option<PlanStatus>,
    cascade_model_config_data: Option<CascadeModelConfigData>,
}

#[derive(Debug, Deserialize)]
struct Tier {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanStatus {
    plan_info: Option<PlanInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanInfo {
    plan_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeModelConfigData {
    client_model_configs: Option<Vec<LSModelConfig>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LSCommandConfigsEnvelope {
    client_model_configs: Option<Vec<LSModelConfig>>,
}

#[derive(Debug, Deserialize)]
struct CCModelsEnvelope {
    models: Option<HashMap<String, CCModel>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCModel {
    model: Option<String>,
    display_name: Option<String>,
    label: Option<String>,
    is_internal: Option<bool>,
    quota_info: Option<AntigravityQuotaInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCQuotaEnvelope {
    buckets: Option<Vec<CCQuotaBucket>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCQuotaBucket {
    model_id: Option<String>,
    remaining_fraction: Option<f64>,
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCLoadEnvelope {
    cloudaicompanion_project: Option<String>,
    current_tier: Option<Tier>,
    paid_tier: Option<Tier>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_process_rows_from_powershell_json() {
        let rows = parse_process_rows(
            r#"{"ProcessId":42,"Name":"language_server.exe","CommandLine":"C:\\app\\language_server.exe --ide_name antigravity --csrf_token token --extension_server_port=1234"}"#,
        )
        .expect("rows should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pid, 42);
        assert_eq!(
            extract_flag(&rows[0].command_line, "--csrf_token"),
            Some("token".to_string())
        );
    }

    #[test]
    fn ranks_antigravity_language_server_candidates() {
        let rows = vec![
            ProcessRow {
                pid: 1,
                name: "language_server.exe".to_string(),
                command_line: "language_server.exe --app_data_dir other --csrf_token nope"
                    .to_string(),
            },
            ProcessRow {
                pid: 2,
                name: "language_server.exe".to_string(),
                command_line: "language_server.exe --ide_name antigravity --csrf_token yes"
                    .to_string(),
            },
        ];

        let candidate = ranked_candidate(&rows, "language_server", &["antigravity"]);

        assert_eq!(candidate, Some((2, rows[1].command_line.clone())));
    }

    #[test]
    fn parses_netstat_listening_ports_for_pid() {
        let ports = parse_netstat_ports(
            r#"
  TCP    127.0.0.1:52168        0.0.0.0:0              LISTENING       99
  TCP    [::1]:52169            [::]:0                 LISTENING       99
  TCP    127.0.0.1:55555        0.0.0.0:0              ESTABLISHED     99
  TCP    127.0.0.1:60000        0.0.0.0:0              LISTENING       10
"#,
            99,
        );

        assert_eq!(ports, vec![52168, 52169]);
    }

    #[test]
    fn parses_user_status_and_collapses_model_pools() {
        let summary = parse_user_status_json(
            r#"
            {
              "userStatus": {
                "userTier": { "name": "Google AI Pro" },
                "cascadeModelConfigData": {
                  "clientModelConfigs": [
                    {
                      "label": "Gemini 3 Pro (High)",
                      "modelOrAlias": { "model": "gemini-3-pro-preview" },
                      "quotaInfo": { "remainingFraction": 0.8, "resetTime": "2099-01-01T00:00:00Z" }
                    },
                    {
                      "label": "Gemini 2.5 Flash",
                      "modelOrAlias": { "model": "gemini-flash" },
                      "quotaInfo": { "remainingFraction": 0.9 }
                    },
                    {
                      "label": "Claude Sonnet",
                      "modelOrAlias": { "model": "claude-weekly" },
                      "quotaInfo": { "remainingFraction": 0.4 }
                    },
                    {
                      "label": "Gemini duplicate",
                      "modelOrAlias": { "model": "MODEL_GOOGLE_GEMINI_2_5_PRO" },
                      "quotaInfo": { "remainingFraction": 0.1 }
                    }
                  ]
                }
              }
            }
            "#,
        )
        .expect("json should parse")
        .expect("user status should exist");

        assert_eq!(summary.plan_type, Some("Pro".to_string()));
        assert_eq!(
            summary.pools,
            vec![
                UsagePool {
                    label: "Gemini Pro".to_string(),
                    remaining_percent: 80.0,
                    reset_at: Some("2099-01-01T00:00:00Z".to_string()),
                },
                UsagePool {
                    label: "Gemini Flash".to_string(),
                    remaining_percent: 90.0,
                    reset_at: None,
                },
                UsagePool {
                    label: "Claude".to_string(),
                    remaining_percent: 40.0,
                    reset_at: None,
                },
            ]
        );
    }

    #[test]
    fn empty_user_status_has_no_pools() {
        let summary = parse_user_status_json(
            r#"
            {
              "userStatus": {
                "userTier": { "name": "Google AI Pro" },
                "cascadeModelConfigData": {
                  "clientModelConfigs": []
                }
              }
            }
            "#,
        )
        .expect("json should parse")
        .expect("user status should exist");

        assert_eq!(summary.plan_type, Some("Pro".to_string()));
        assert!(summary.pools.is_empty());
    }

    #[test]
    fn extracts_go_keyring_wrapped_antigravity_token() {
        let wrapped = format!(
            "go-keyring-base64:{}",
            general_purpose::STANDARD
                .encode(r#"{"token":{"access_token":"access-1","refresh_token":"refresh-1"}}"#)
        );

        let token = extract_token_from_keyring_raw(&wrapped).expect("token should parse");

        assert_eq!(
            token,
            AuthToken {
                access_token: Some("access-1".to_string()),
                refresh_token: Some("refresh-1".to_string()),
            }
        );
    }

    #[test]
    fn extracts_nested_antigravity_refresh_token() {
        let token = extract_token_from_keyring_raw(
            r#"{"credentials":{"accessToken":"access-2","refreshToken":"refresh-2"}}"#,
        )
        .expect("nested token should parse");

        assert_eq!(token.access_token, Some("access-2".to_string()));
        assert_eq!(token.refresh_token, Some("refresh-2".to_string()));
    }

    #[cfg(windows)]
    #[test]
    fn decodes_windows_credential_blob_as_utf8_or_utf16() {
        assert_eq!(
            decode_windows_credential_blob(b"go-keyring-base64:test"),
            Some("go-keyring-base64:test".to_string())
        );

        let utf16 = "go-keyring-base64:test"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        assert_eq!(
            decode_windows_credential_blob(&utf16),
            Some("go-keyring-base64:test".to_string())
        );
    }

    #[test]
    fn parses_cloud_code_models_and_filters_internal_models() {
        let configs = parse_cloud_code_models_json(
            r#"
            {
              "models": {
                "gemini-3-pro": {
                  "displayName": "Gemini 3 Pro",
                  "model": "gemini-3-pro",
                  "quotaInfo": { "remainingFraction": 0.7, "resetTime": "2099-01-01T00:00:00Z" }
                },
                "gemini-flash": {
                  "displayName": "Gemini 3 Flash",
                  "model": "gemini-flash",
                  "quotaInfo": { "remainingFraction": 0.6 }
                },
                "tab-flash": {
                  "displayName": "Gemini internal",
                  "isInternal": true,
                  "quotaInfo": { "remainingFraction": 0.1 }
                },
                "claude-sonnet": {
                  "label": "Claude Sonnet",
                  "quotaInfo": { "remainingFraction": 0.3 }
                }
              }
            }
            "#,
        )
        .expect("cloud models should parse");

        let pools = build_pools(&configs);

        assert_eq!(
            pools,
            vec![
                UsagePool {
                    label: "Gemini Pro".to_string(),
                    remaining_percent: 70.0,
                    reset_at: Some("2099-01-01T00:00:00Z".to_string()),
                },
                UsagePool {
                    label: "Gemini Flash".to_string(),
                    remaining_percent: 60.0,
                    reset_at: None,
                },
                UsagePool {
                    label: "Claude".to_string(),
                    remaining_percent: 30.0,
                    reset_at: None,
                },
            ]
        );
    }

    #[test]
    fn parses_cloud_code_plan_and_project() {
        let parsed = parse_load_code_assist_json(
            r#"
            {
              "cloudaicompanionProject": "project-1",
              "currentTier": { "name": "Gemini Code Assist Free" },
              "paidTier": { "name": "Gemini Code Assist in Google One AI Ultra" }
            }
            "#,
        )
        .expect("loadCodeAssist should parse");

        assert_eq!(
            parsed,
            LoadCodeAssist {
                plan_type: Some("Ultra".to_string()),
                project: Some("project-1".to_string()),
            }
        );
    }
}
