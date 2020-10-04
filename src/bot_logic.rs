use crate::format::*;
use crate::request::*;
use crate::util::*;
use crate::{
    messages::Message,
    yew_app::{Model, Msg},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use string_tools::*;
use web_sys::window;
use yew::prelude::*;

fn get_fraud() -> String {
    js_sys::eval("fraudService.hashedFraud()")
        .unwrap()
        .as_string()
        .unwrap()
}

fn jsmd5(input: &str) -> String {
    js_sys::eval(&format!(r#"jsmd5("{}")"#, input))
        .unwrap()
        .as_string()
        .unwrap()
}

fn true_function() -> bool {true}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub twitter_username: String,
    #[serde(default)]
    pub total_entries: usize,
    #[serde(default = "true_function")]
    pub ban_unknown_methods: bool,
    #[serde(default)]
    pub auto_follow_twitter: bool,
    #[serde(default)]
    pub auto_retweet: bool,
    #[serde(default)]
    pub auto_tweet: bool,
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings {
            twitter_username: String::new(),
            total_entries: 0,
            ban_unknown_methods: true,
            auto_follow_twitter: false,
            auto_retweet: false,
            auto_tweet: false,
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
            Some(value) => serde_json::from_str(&value).unwrap_or_default(),
            None => Settings::default(),
        }
    }
}

pub async fn run(
    link: Rc<ComponentLink<Model>>,
    settings: Rc<RefCell<Settings>>,
) -> Result<(), Message<String>> {
    let window = window().unwrap();
    let location = window.location();
    let href = location.href().unwrap();

    let main_page = request_str::<()>(&href, Method::Get, HashMap::new(), "")
        .await
        .unwrap();

    log!("{}", main_page);
    let fpr = get_fraud();
    log!("{}", fpr);

    let text = main_page;
    let csrf =
        string_tools::get_all_between_strict(&text, "<meta name=\"csrf-token\" content=\"", "\"")
            .unwrap();
    let json =
        string_tools::get_all_between_strict(&text, " ng-init='initCampaign(", ")'>").unwrap();
    let json = json.replace("&quot;", "\"");
    let giveaway = match serde_json::from_str::<Giveaway>(&json) {
        Ok(g) => g,
        Err(e) => {
            elog!("e {:?}", e);
            panic!("{}", &json[e.column() - 10..]);
        }
    };
    log!("giveaway: {:#?}", giveaway);

    let json =
        string_tools::get_all_between_strict(&text, " ng-init='initContestant(", ");").unwrap();
    let json = json.replace("&quot;", "\"");
    let init_contestant = match serde_json::from_str::<InitContestant>(&json) {
        Ok(g) => g,
        Err(e) => {
            elog!("e {:?} in {}", e, json);
            panic!("{}", &json[e.column() - 10..]);
        }
    };
    log!("contestant: {:#?}", init_contestant);
    let mut contestant = init_contestant.contestant;

    let mut dbg: HashMap<&String, Value> = HashMap::new();
    let mut frm: HashMap<&String, Value> = HashMap::new();
    for idx in 0..giveaway.entry_methods.len() {
        let entry = &giveaway.entry_methods[idx];
        if entry.requires_details {
            match entry.provider.as_str() {
                "twitter" => {
                    frm.insert(
                        &entry.id,
                        json! {{
                            "twitter_username": settings.borrow().twitter_username,
                        }},
                    );
                }
                p => println!("unknown provider {}", p),
            }
        }
    }

    let mut current: f32 = 0.0;
    let len = giveaway.entry_methods.len() as f32;
    for idx in 0..giveaway.entry_methods.len() {
        let entry = &giveaway.entry_methods[idx];

        if contestant.entered.contains_key(&entry.id) {
            log!("Already entered, skipping");
            continue;
        }

        match entry.entry_type.as_str() {
            "twitter_follow" => if settings.borrow().auto_follow_twitter {
                let username = match &entry.config1 {
                    Some(username) => username,
                    None => {
                        link.send_message(Msg::LogMessage(Message::Error(
                            "Invalid gleam.io entry".to_string(),
                        )));
                        continue;
                    }
                };

                let url = format!("https://twitter.com/intent/follow?screen_name={}&gleambot=true", username);
                
                if let Err(e) = window.open_with_url(&url) {
                    link.send_message(Msg::LogMessage(Message::Error(format!(
                        "Failed to open a new window: {:?}",
                        e
                    ))));
                    continue;
                } else {
                    sleep(Duration::from_secs(15)).await;
                }
            } else {
                link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped twitter follow in accordance to your settings. Considering enabling auto-follow to get more entries.".to_string()
                )));
                continue;
            },
            "twitter_retweet" => if settings.borrow().auto_retweet {
                if let Some(config1) = &entry.config1 {
                    if let Some(id) = get_all_after_strict(&config1, "/status/") {
                        let url = format!("https://twitter.com/intent/retweet?tweet_id={}&gleambot=true", id);
                        if let Err(e) = window.open_with_url(&url) {
                            link.send_message(Msg::LogMessage(Message::Error(format!(
                                "Failed to open a new window: {:?}",
                                e
                            ))));
                            continue;
                        } else {
                            sleep(Duration::from_secs(15)).await;
                        }
                    }
                }
            } else {
                link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped retweet in accordance to your settings. Considering enabling auto-retweet to get more entries.".to_string()
                )));
                continue;
            }
            "twitter_tweet" => if settings.borrow().auto_tweet {
                if let Some(text) = &entry.config1 {
                    let url = format!("https://twitter.com/intent/tweet?text={}&gleambot=true", text);
                    if let Err(e) = window.open_with_url(&url) {
                        link.send_message(Msg::LogMessage(Message::Error(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ))));
                        continue;
                    } else {
                        sleep(Duration::from_secs(15)).await;
                    }
                }
            } else {
                link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped tweet in accordance to your settings. Considering enabling auto-tweet to get more entries.".to_string()
                )));
                continue;
            }
            entry_type => {
                if settings.borrow().ban_unknown_methods {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Encountered an unknown entry type: {:?}. This entry method has been skipped. You can enable auto-entering for unknown entry methods in the settings, but it may not work properly.",
                        entry_type
                    ))));
                    continue;
                } else {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Encountered an unknown entry type: {:?}. The bot will try to enter (since it is enabled in the settings). Could you please report this unknown entry method by opening an issue on Github or by sending an email to mubelotix@gmail.com? Please mention the url of this giveaway in your report. That would help me a lot to extend and improve the bot support. Thank you very much.",
                        entry_type
                    ))));
                }
            },
        }

        let body = json! {{
            "dbg": dbg,
            "details": "V",
            "emid": entry.id,
            "f": fpr,
            "frm": frm,
            "grecaptcha_response": null,
            "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key))
        }};
        log!("request: {:?}", body);

        let rep = match request::<Value, EntryResponse>(
            &format!(
                "https://gleam.io/enter/{}/{}",
                giveaway.campaign.key, entry.id
            ),
            Method::Post(body),
            HashMap::new(),
            csrf,
        )
        .await
        {
            Ok(response) => response,
            Err(e) => {
                link.send_message(Msg::LogMessage(Message::Warning(format!(
                    "Unexpected response to HTTP request: {:?}",
                    e
                ))));
                continue;
            }
        };
        log!("response: {:?}", rep);

        match rep {
            EntryResponse::Error { error } => match error.as_str() {
                "not_logged_in" => {
                    match request::<SetContestantRequest, Contestant>(
                        "https://gleam.io/set-contestant",
                        Method::Post(SetContestantRequest {
                            additional_details: true,
                            campaign_key: giveaway.campaign.key.clone(),
                            contestant: StoredContestant {
                                competition_subscription: None,
                                date_of_birth: contestant.stored_dob.clone(),
                                email: contestant.email.clone(),
                                firstname: get_all_before(&contestant.name, " ").to_string(),
                                lastname: get_all_after(&contestant.name, " ").to_string(),
                                name: contestant.name.clone(),
                                send_confirmation: false,
                                stored_dob: contestant.stored_dob.clone(),
                            },
                        }),
                        HashMap::new(),
                        csrf,
                    )
                    .await
                    {
                        Ok(c) => contestant = c,
                        Err(e) => link.send_message(Msg::LogMessage(Message::Error(format!(
                            "Unable to auto login: {:?}",
                            e
                        )))),
                    }
                }
                "error_auth_expired" => {
                    link.send_message(Msg::LogMessage(Message::Error(format!(
                        "Gleam.io is unable to check the action. Please login to {}.",
                        entry.provider
                    ))))
                }
                error => link.send_message(Msg::LogMessage(Message::Error(format!(
                    "An unknown error occured while trying get entries: {:?}",
                    error
                )))),
            },
            EntryResponse::RefreshRequired {
                require_campaign_refresh: b,
            } => link.send_message(Msg::LogMessage(Message::Warning(format!(
                "Reload required: {}",
                b
            )))),
            EntryResponse::AlreadyEntered { .. } => link.send_message(Msg::LogMessage(
                Message::Warning("Already entered!".to_string()),
            )),
            EntryResponse::Success { worth, .. } => {
                settings.borrow_mut().total_entries += worth;
                settings.borrow().save();
            },
            EntryResponse::BotSpotted { cheater } => {
                if cheater {
                    return Err(Message::Danger("Gleam.io says you are creating too many entries. Shutting down.".to_string()))
                }
            }
        }

        if frm.get(&entry.id).is_none() {
            frm.insert(&entry.id, json! {null});
        }
        dbg.insert(&entry.id, json! {null});

        current += 1.0;
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * current).floor() as usize
        ));

        sleep(Duration::from_secs(5)).await;
    }

    link.send_message(Msg::Done);

    Ok(())
}
