use serde::{Deserialize, Serialize};
use serde_json::Value;
use format::prelude::*;

#[derive(Debug, Deserialize)]
pub struct SetContestantResponse {
    pub contestant: Contestant,
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
