use serde::{Serialize, Deserialize};
use serde_json::Value;

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatedEntry {
    c: usize,
    t: u64,
    ts: Value,
    w: usize,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct EntryMethod {
    pub id: String,
    pub entry_type: String,
    pub type_without_provider: String,
    pub config: Value,
    pub worth: usize,
    pub variable_worth: bool,
    pub provider: String,
    pub verified: bool,
    pub value_format: Option<Value>,
    pub must_verify: bool,
    pub requires_authentication: bool,
    pub can_authenticate: bool,
    pub requires_details: bool,
    pub display_information: Option<bool>,
    pub auth_for_details: bool,
    pub api_fallback: Option<bool>,
    pub auto_expandable: Option<bool>,
    pub expandable: bool,
    pub double_opt_in: bool,
    pub allowed_file_extensions: Vec<Value>,
    pub config1: Option<String>,
    pub config2: Option<String>,
    pub config3: Option<String>,
    pub config4: Option<String>,
    pub config5: Option<String>,
    pub config6: Option<String>,
    pub config7: Option<String>,
    pub config8: Option<String>,
    pub config9: Option<String>,
    pub config_selections: Vec<Value>,
    pub iframe_url: Option<String>,
    pub iframe_type: Option<Value>,
    pub accepts_file_types: Option<Value>,
    pub method_type: Option<String>,
    pub config_toggle: bool,
    pub interval_seconds: usize,
    pub next_interval_starts_at: usize,
    pub actions_required: usize,
    pub template: String,
    pub normal_icon: String,
    pub normal_icon_color: String,
    pub unlocked_icon: String,
    pub unlocked_icon_color: String,
    pub completable: bool,
    pub maxlength: Value,
    pub restrict: Option<Value>,
    pub mandatory: bool,
    pub workflow: Option<String>,
    pub timer_action: Option<Value>,
    pub limit: usize,
    pub always_require_email: bool,
    pub media_action: bool,
    pub preload_images: Vec<Value>,
    pub tiers: Vec<Value>,
    pub shows_content_after_entry: Option<bool>,
    pub kill_switch_message: Option<Value>,
    pub paid: bool,
    pub action_description: String,
    pub share_suffix: Option<String>,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteEntryMethod {
    pub type_without_provider: String,
    pub worth: usize,
    pub provider: String
}

impl From<EntryMethod> for IncompleteEntryMethod {
    fn from(entry_method: EntryMethod) -> IncompleteEntryMethod {
        IncompleteEntryMethod {
            type_without_provider: entry_method.type_without_provider,
            worth: entry_method.worth,
            provider: entry_method.provider
        }
    }
}