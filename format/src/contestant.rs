use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::entry::ValidatedEntry;

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct InitContestant {
    pub contestant: MaybeUninitContestant,
    pub form: HashMap<String, Value>,
    pub location_allowed: bool,
    pub referrer_allowed: bool,
    pub trigger_auto_opt_in: bool,
    pub allow_autoticking: bool,
    pub amoeRequired: bool,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeUninitContestant {
    Connected(Contestant),
    Disconnected {
        entered: Value,
        claims: Value
    }
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Contestant {
    pub auth_key: Option<String>,
    pub authentications: Vec<Authentification>,
    pub banned: bool,
    pub claims: Value,
    pub competition_subscription: Option<bool>,
    pub completed_details: bool,
    pub email: String,
    pub entered: HashMap<String, Vec<ValidatedEntry>>,
    pub id: usize,
    pub name: String,
    pub share_key: String,
    pub stored_dob: Option<String>,
    pub viral_share_paths: HashMap<String, String>,
    pub send_confirmation: Option<bool>,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Authentification {
    #[serde(default)]
    pub expired: bool,
    pub id: u64,
    pub profile_url: Option<String>,
    pub provider: String,
    pub provider_type: Option<String>,
    pub reference: Option<String>,
    pub uid: String,
    pub updated_at: String,
    pub url: Option<String>,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Winner {
    entry_number: usize,
    name: String,
    image: String,
}
