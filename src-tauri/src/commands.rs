use crate::{plugin_host, providers::deepseek, secrets};

struct DeepSeekHost {
    balance_json: String,
}

impl plugin_host::Host for DeepSeekHost {
    fn app_name(&self) -> &'static str {
        "InfUsage"
    }

    fn deepseek_balance_json(&self) -> String {
        self.balance_json.clone()
    }
}

#[tauri::command]
pub fn save_deepseek_api_key(api_key: String) -> Result<Vec<secrets::DeepSeekKeySlot>, String> {
    let trimmed = api_key.trim();

    if trimmed.is_empty() {
        return Err("DeepSeek API key must not be empty".to_string());
    }

    if secrets::load_deepseek_api_keys().len() >= secrets::MAX_DEEPSEEK_KEYS as usize {
        return Err("Delete the saved DeepSeek key before adding a new one".to_string());
    }

    secrets::save_deepseek_api_key(trimmed).map_err(|error| error.to_string())?;
    Ok(secrets::list_deepseek_key_slots())
}

#[tauri::command]
pub fn list_deepseek_api_keys() -> Vec<secrets::DeepSeekKeySlot> {
    secrets::list_deepseek_key_slots()
}

#[tauri::command]
pub fn delete_deepseek_api_key(slot: u8) -> Result<Vec<secrets::DeepSeekKeySlot>, String> {
    if !(1..=secrets::MAX_DEEPSEEK_KEYS).contains(&slot) {
        return Err("Unknown DeepSeek key slot".to_string());
    }

    secrets::delete_deepseek_api_key(slot).map_err(|error| error.to_string())?;
    Ok(secrets::list_deepseek_key_slots())
}

#[tauri::command]
pub fn refresh_deepseek() -> Result<plugin_host::ProviderSnapshot, String> {
    let api_keys = secrets::load_deepseek_api_keys();

    if api_keys.is_empty() {
        return Err("DeepSeek API key is not saved".to_string());
    }

    let mut usd_remaining = 0.0;

    for (slot, api_key) in api_keys {
        let balance_json = deepseek::fetch_balance_json(&api_key)
            .map_err(|error| format!("DeepSeek key {slot}: {error}"))?;
        let balance = deepseek::parse_balance_json(&balance_json)
            .map_err(|error| format!("DeepSeek key {slot}: {error}"))?;

        usd_remaining += deepseek::usd_total_balance(&balance);
    }

    let balance_json =
        deepseek::usd_balance_json(usd_remaining).map_err(|error| error.to_string())?;

    plugin_host::run_deepseek_provider(&DeepSeekHost { balance_json })
        .map_err(|error| error.to_string())
}
