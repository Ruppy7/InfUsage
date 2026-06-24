use serde::{Deserialize, Serialize};

const SERVICE: &str = "InfUsage";
const LEGACY_DEEPSEEK_USER: &str = "deepseek-api-key";
const OPENCODE_QUOTA_SESSION_USER: &str = "opencode-quota-session";
pub const MAX_DEEPSEEK_KEYS: u8 = 1;

#[derive(Debug, Serialize)]
pub struct DeepSeekKeySlot {
    pub id: u8,
    pub has_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenCodeQuotaSession {
    pub cookie: String,
    pub workspace_id: String,
}

pub fn save_opencode_quota_session(session: &OpenCodeQuotaSession) -> Result<(), keyring::Error> {
    let json = serde_json::to_string(session)
        .map_err(|error| keyring::Error::PlatformFailure(Box::new(error)))?;
    opencode_quota_session_entry()?.set_password(&json)
}

pub fn load_opencode_quota_session() -> Option<OpenCodeQuotaSession> {
    let json = opencode_quota_session_entry().ok()?.get_password().ok()?;
    serde_json::from_str(&json).ok()
}

pub fn delete_opencode_quota_session() -> Result<(), keyring::Error> {
    match opencode_quota_session_entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn has_opencode_quota_session() -> bool {
    load_opencode_quota_session().is_some()
}

fn opencode_quota_session_entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, OPENCODE_QUOTA_SESSION_USER)
}

pub fn save_deepseek_api_key(api_key: &str) -> Result<u8, keyring::Error> {
    let slot = first_empty_deepseek_slot().unwrap_or(MAX_DEEPSEEK_KEYS);
    deepseek_entry(slot)?.set_password(api_key)?;
    Ok(slot)
}

pub fn delete_deepseek_api_key(slot: u8) -> Result<(), keyring::Error> {
    let deleted_legacy = slot == 1
        && legacy_deepseek_entry()
            .and_then(|entry| entry.delete_credential())
            .is_ok();

    match deepseek_entry(slot)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(_) if deleted_legacy => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn list_deepseek_key_slots() -> Vec<DeepSeekKeySlot> {
    (1..=MAX_DEEPSEEK_KEYS)
        .map(|id| DeepSeekKeySlot {
            id,
            has_key: load_deepseek_api_key(id).is_ok(),
        })
        .collect()
}

pub fn load_deepseek_api_keys() -> Vec<(u8, String)> {
    (1..=MAX_DEEPSEEK_KEYS)
        .filter_map(|id| load_deepseek_api_key(id).ok().map(|api_key| (id, api_key)))
        .collect()
}

fn first_empty_deepseek_slot() -> Option<u8> {
    (1..=MAX_DEEPSEEK_KEYS).find(|id| load_deepseek_api_key(*id).is_err())
}

fn load_deepseek_api_key(slot: u8) -> Result<String, keyring::Error> {
    if slot == 1 {
        legacy_deepseek_entry()
            .and_then(|entry| entry.get_password())
            .or_else(|_| deepseek_entry(slot)?.get_password())
    } else {
        deepseek_entry(slot)?.get_password()
    }
}

fn deepseek_entry(slot: u8) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, &format!("deepseek-api-key-{slot}"))
}

fn legacy_deepseek_entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, LEGACY_DEEPSEEK_USER)
}
