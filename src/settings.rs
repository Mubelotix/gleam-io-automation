use serde::{Deserialize, Serialize};
use serde_json::from_str;
use web_sys::window;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub twitter_username: String,
    pub total_entries: usize,
    pub auto_follow_twitter: bool,
    pub auto_retweet: bool,
    pub auto_tweet: bool,
    pub auto_email_subscribe: bool,
    pub text_input_sentence: String,
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings {
            twitter_username: String::new(),
            total_entries: 0,
            auto_follow_twitter: false,
            auto_retweet: false,
            auto_tweet: false,
            auto_email_subscribe: false,
            text_input_sentence: String::from(
                "Îmi pare rău, nu înțeleg ce ar trebui să scriu aici.",
            ),
        }
    }
}

impl Settings {
    pub fn save(&self) {
        let storage = window().unwrap().local_storage().unwrap().unwrap();
        storage
            .set("gleam_bot_settings", &serde_json::to_string(&self).unwrap())
            .unwrap();
    }

    pub fn load() -> Settings {
        let storage = window().unwrap().local_storage().unwrap().unwrap();
        match storage.get("gleam_bot_settings").ok().flatten() {
            Some(value) => from_str(&value).unwrap_or_default(),
            None => Settings::default(),
        }
    }
}
