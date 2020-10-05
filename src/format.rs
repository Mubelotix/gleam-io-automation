use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Shortener {
    pub url: String,
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Campaign {
    pub starts_at: u64,
    pub ends_at: u64,
    pub key: String,
    pub banned: bool,
    pub tracking_pixels: Value,
    pub referral_link_id: u64,
    pub remove_branding: bool,
    pub widget_callbacks: Value,
    pub name: String,
    pub language: String,
    pub site_url: String,
    pub site_name: String,
    pub finished: bool,
    pub paused: bool,
    pub login_first: bool,
    pub auto_enter: bool,
    pub login_providers: Vec<String>,
    pub login_types: Vec<String>,
    pub all_possible_login_providers: Vec<String>,
    pub verified_login_providers: Vec<String>,
    pub details_first: bool,
    pub show_competition_subscription: bool,
    pub stand_alone_option: String,
    pub stand_alone_url: String,
    pub landing_page_override: Option<Value>,
    pub landing_page_styling: Option<Value>,
    pub hide_entry_title: bool,
    pub hide_social_logins: bool,
    pub campaign_type: String,
    pub landing_page: String,
    pub first_and_last_name: bool,
    pub messages: Value,
    pub additional_contestant_details: bool,
    pub splitted_fullname: Option<Value>,
    pub optional_lastname: Option<Value>,
    pub require_contact_info: bool,
    pub hide_total_entries: bool,
    pub entry_limit: Option<usize>,
    pub facebook_url: Option<String>,
    pub pin_url: String,
    pub share: bool,
    pub terms_and_conditions: String,
    pub announce_winners: bool,
    pub updating_worth: usize,
    pub loading_icon: String,
    pub multiple_shares: bool,
    pub contestant_details_groups: Value,
    pub contestant_steps: usize,
    pub post_entry_url: String,
    pub pinterest_app_banned: bool,
    pub event_mode: bool,
    pub suppress_redeem_display: bool,
    pub default_confirmation_email: bool,
    pub custom_confirmation_email: bool,
    pub trying_unpaid_features: bool,
    pub shortener: Shortener,
}

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
    pub requires_details: bool, // todo use this
    pub display_information: Option<bool>,
    pub auth_for_details: bool,
    pub api_fallback: Option<Value>,
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
    pub method_type: Option<Value>,
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
    pub timer_action: Option<u64>,
    pub limit: usize,
    pub always_require_email: bool,
    pub media_action: bool,
    pub preload_images: Vec<Value>,
    pub tiers: Vec<Value>,
    pub shows_content_after_entry: Option<bool>,
    pub kill_switch_message: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Incentive {
    pub id: usize,
    pub name: String,
    pub actions_required: usize,
    pub description: String,
    pub data_type: String,
    pub input_type: Option<Value>,
    pub incentive_type: String,
    pub layout: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Giveaway {
    pub entry_state: Value,
    pub entry_details_state: Value,
    pub app_name: String,
    pub campaign: Campaign,
    #[serde(rename = "entry_methods")]
    pub entry_methods: Vec<EntryMethod>,
}

#[derive(Debug, Deserialize)]
pub struct SetContestantResponse {
    pub allow_recovery: bool,
    pub require_other_login: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Contestant {
    pub auth_key: Option<String>,
    #[serde(default)]
    pub authentications: Vec<Value>,
    #[serde(default)]
    pub banned: bool,
    pub claims: Value,
    pub competition_subscription: Option<bool>,
    pub completed_details: bool,
    pub email: String,
    pub entered: HashMap<String, Vec<Value>>,
    pub id: usize,
    pub name: String,
    pub share_key: String,
    pub stored_dob: Option<String>,
    pub viral_share_paths: HashMap<String, String>,
    pub send_confirmation: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct InitContestant {
    pub contestant: Contestant,
    pub form: HashMap<String, Value>,
    pub location_allowed: bool,
    pub referrer_allowed: bool,
    pub trigger_auto_opt_in: bool,
    pub allow_autoticking: bool,
    pub amoeRequired: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecoverContestantResponse {
    pub contestant: Contestant,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EntryResponse {
    Success {
        new: u64,
        worth: usize,
    },
    AlreadyEntered {
        existing_at: u64,
        worth: usize,
        difference: u64,
        interval_seconds: usize,
    },
    RefreshRequired {
        require_campaign_refresh: bool,
    },
    Error {
        error: String,
    },
    BotSpotted {
        cheater: bool,
    },
    IpBan {
        ip_ban: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredContestant {
    pub competition_subscription: Option<Value>,
    pub date_of_birth: String,
    pub email: String,
    pub firstname: String,
    pub lastname: String,
    pub name: String,
    pub send_confirmation: bool,
    pub stored_dob: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetContestantRequest {
    pub additional_details: bool,
    pub campaign_key: String,
    pub contestant: StoredContestant,
}
