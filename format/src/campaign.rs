use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{shortener::Shortener, contestant::Winner};

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
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
    pub has_paid_entry_methods: bool,
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
    pub cookie_check_disabled: Option<Value>,
    pub winners: Option<Vec<Winner>>,
    pub shortener: Shortener,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteCampaign {
    pub starts_at: u64,
    pub ends_at: u64,
    pub key: String,
    pub name: String,
    pub language: String,
    pub site_url: String,
    pub site_name: String,
    pub finished: bool,
    pub paused: bool,
    pub login_types: Vec<String>,
    pub stand_alone_url: String,
    pub campaign_type: String,
    pub terms_and_conditions: String,
    pub announce_winners: bool
}

impl From<Campaign> for IncompleteCampaign {
    fn from(campaign: Campaign) -> IncompleteCampaign {
        IncompleteCampaign {
            starts_at: campaign.starts_at,
            ends_at: campaign.ends_at,
            key: campaign.key,
            name: campaign.name,
            language: campaign.language,
            site_url: campaign.site_url,
            site_name: campaign.site_name,
            finished: campaign.finished,
            paused: campaign.paused,
            login_types: campaign.login_types,
            stand_alone_url: campaign.stand_alone_url,
            campaign_type: campaign.campaign_type,
            terms_and_conditions: campaign.terms_and_conditions,
            announce_winners: campaign.announce_winners,
        }
    }
}