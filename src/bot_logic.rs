use crate::format::*;
use crate::request::*;
use crate::util::*;
use crate::{
    messages::Message,
    yew_app::{Model, Msg},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub twitter_username: String,
    pub total_entries: usize,
    pub ban_unknown_methods: bool,
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
            ban_unknown_methods: true,
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

pub enum Arg<'a> {
    IsNumber,
    IsExact(Option<&'a String>),
    IsIn(&'a [&'static str]),
    Is(&'static str),
    Exists,
    Lacks,
    IsEmpty,
    IsNotEmpty,
    Anything,
    IsUrl,
}

impl<'a> Arg<'a> {
    pub fn matches(&self, value: Option<&String>) -> Result<(), &'static str> {
        match self {
            Arg::IsNumber => {
                if let Some(value) = value {
                    if value.parse::<u64>().is_ok() {
                        Ok(())
                    } else {
                        Err("Expected Number, found String")
                    }
                } else {
                    Err("Expected Number, found Null")
                }
            }
            Arg::Is(expected_value) => {
                if value.map(|s| s.as_str()) == Some(expected_value) {
                    Ok(())
                } else {
                    Err("Expected a specific value, got something else")
                }
            }
            Arg::IsIn(values) => {
                for expected_value in values.iter() {
                    if Some(*expected_value) == value.map(|s| s.as_str()) {
                        return Ok(());
                    }
                }
                Err("Unexpected value")
            }
            Arg::Exists => {
                if value.is_some() {
                    Ok(())
                } else {
                    Err("Expected something, found Null")
                }
            }
            Arg::Lacks => {
                if value.is_none() {
                    Ok(())
                } else {
                    Err("Unexpected value")
                }
            }
            Arg::IsEmpty => {
                if value == Some(&"".to_string()) {
                    Ok(())
                } else {
                    Err("Expected an empty value, got something else")
                }
            }
            Arg::IsNotEmpty | Arg::IsUrl => {
                if let Some(value) = value {
                    if !value.is_empty() {
                        Ok(())
                    } else {
                        Err("Expected non-empty String")
                    }
                } else {
                    Err("Expected String, found Null")
                }
            }
            Arg::IsExact(expected_value) => {
                if &value == expected_value {
                    Ok(())
                } else {
                    Err("Expected a specific value, got something else")
                }
            }
            Arg::Anything => {
                Ok(())
            }
        }
    }
}

pub struct Verifyer<'a> {
    workflow: Arg<'a>,
    template: Arg<'a>,
    method_type: Arg<'a>,
    configs: [Arg<'a>; 9],
}

impl<'a> Verifyer<'a> {
    pub fn new(
        workflow: Arg<'a>,
        template: Arg<'a>,
        method_type: Arg<'a>,
        configs: [Arg<'a>; 9],
    ) -> Verifyer<'a> {
        Verifyer {
            workflow,
            template,
            method_type,
            configs,
        }
    }

    pub fn matches(&self, entry: &EntryMethod) -> Result<(), (&'static str, &'static str)> {
        self.workflow
            .matches(entry.workflow.as_ref())
            .map_err(|e| ("workflow", e))?;
        self.template
            .matches(Some(&entry.template))
            .map_err(|e| ("template", e))?;
        self.method_type
            .matches(entry.method_type.as_ref())
            .map_err(|e| ("method_type", e))?;
        self.configs[0]
            .matches(entry.config1.as_ref())
            .map_err(|e| ("1", e))?;
        self.configs[1]
            .matches(entry.config2.as_ref())
            .map_err(|e| ("2", e))?;
        self.configs[2]
            .matches(entry.config3.as_ref())
            .map_err(|e| ("3", e))?;
        self.configs[3]
            .matches(entry.config4.as_ref())
            .map_err(|e| ("4", e))?;
        self.configs[4]
            .matches(entry.config5.as_ref())
            .map_err(|e| ("5", e))?;
        self.configs[5]
            .matches(entry.config6.as_ref())
            .map_err(|e| ("6", e))?;
        self.configs[6]
            .matches(entry.config7.as_ref())
            .map_err(|e| ("7", e))?;
        self.configs[7]
            .matches(entry.config8.as_ref())
            .map_err(|e| ("8", e))?;
        self.configs[8]
            .matches(entry.config9.as_ref())
            .map_err(|e| ("9", e))?;
        Ok(())
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

    let twitter_value = json! {{
        "twitter_username": settings.borrow().twitter_username,
    }};

    let current: RefCell<f32> = RefCell::new(0.0);
    let len = giveaway.entry_methods.len() as f32;

    let next = || {
        use std::ops::AddAssign;
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize,
        ));
    };

    let err = |e: String| {
        use std::ops::AddAssign;
        link.send_message(Msg::LogMessage(Message::Error(e)));
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize,
        ));
    };

    let testing = |s: &'static str| {
        link.send_message(Msg::LogMessage(Message::Info(format!(
            "The bot tried a known entry method that has never been tested: {}. Did it work?",
            s
        ))));
    };

    let mut made_requests: Vec<&String> = Vec::new();
    let mut mandatory_entries: usize = 0;
    let mut completed_mandatory_entries: usize = 0;
    let mut entry_methods = Vec::new();
    let mut actions_number = 0;
    for entry in &giveaway.entry_methods {
        if entry.mandatory {
            entry_methods.insert(mandatory_entries, entry);
            mandatory_entries += 1;
            if contestant.entered.contains_key(&entry.id) {
                completed_mandatory_entries += 1;
            }
        } else {
            entry_methods.push(entry);
        }
    }

    if giveaway.campaign.additional_contestant_details {
        match request::<SetContestantRequest, SetContestantResponse>(
            "https://gleam.io/set-contestant",
            Method::Post(SetContestantRequest {
                additional_details: true,
                campaign_key: giveaway.campaign.key.clone(),
                contestant: StoredContestant {
                    competition_subscription: None,
                    date_of_birth: contestant
                        .stored_dob
                        .clone()
                        .unwrap_or_else(|| String::from("1950-01-01")),
                    email: contestant.email.clone(),
                    firstname: get_all_before(&contestant.name, " ").to_string(),
                    lastname: get_all_after(&contestant.name, " ").to_string(),
                    name: contestant.name.clone(),
                    send_confirmation: false,
                    stored_dob: contestant
                        .stored_dob
                        .clone()
                        .unwrap_or_else(|| String::from("1950-01-01")),
                },
            }),
            HashMap::new(),
            csrf,
        )
        .await
        {
            Ok(c) => contestant = c.contestant,
            Err(e) => {
                err(format!("Failed to set contestant: {:?}", e));
                panic!("");
            }
        }
    }

    for entry in entry_methods {
        log!("entry: {:#?}", entry);

        #[cfg(not(feature = "skip"))]
        if contestant.entered.contains_key(&entry.id) {
            log!("Already entered, skipping");
            next();
            continue;
        } else if !entry.mandatory && completed_mandatory_entries < mandatory_entries {
            link.send_message(Msg::LogMessage(Message::Warning(
                "Unable to try some entry methods because some mandatory entry methods were not successfully completed.".to_string()
            )));
            return Ok(());
        } else if entry.actions_required > actions_number {
            link.send_message(Msg::LogMessage(Message::Warning(
                "Unable to try an entry method because it requires more actions to be done."
                    .to_string(),
            )));
            next();
            continue;
        }

        let mut dbg: HashMap<&String, Value> = HashMap::new();
        for made_request in &made_requests {
            dbg.insert(made_request, Value::Null);
        }
        let mut frm = dbg.clone();
        for idx in 0..giveaway.entry_methods.len() {
            let entry = &giveaway.entry_methods[idx];
            if entry.requires_details {
                #[allow(clippy::single_match)]
                match entry.provider.as_str() {
                    "twitter" => {
                        frm.insert(&entry.id, twitter_value.clone());
                    }
                    _ => (),
                }
            }
        }

        let mut details = (Value::Null, false, false);
        use Arg::*;
        match entry.entry_type.as_str() {
            "twitter_followA" => {
                if settings.borrow().auto_follow_twitter {
                    let username = match &entry.config1 {
                        Some(username) => username,
                        None => {
                            link.send_message(Msg::LogMessage(Message::Error(
                                "Invalid gleam.io entry".to_string(),
                            )));
                            next();
                            continue;
                        }
                    };

                    let url = format!(
                        "https://twitter.com/intent/follow?screen_name={}&gleambot=true",
                        username
                    );

                    if let Err(e) = window.open_with_url(&url) {
                        link.send_message(Msg::LogMessage(Message::Error(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ))));
                        next();
                        continue;
                    } else {
                        sleep(Duration::from_secs(15)).await;
                    }

                //details = Some(twitter_value.clone());
                } else {
                    link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped twitter follow in accordance to your settings. Consider enabling auto-follow to get more entries.".to_string()
                )));
                    next();
                    continue;
                }
            }
            "twitter_retweetA" => {
                if settings.borrow().auto_retweet {
                    if let Some(config1) = &entry.config1 {
                        if let Some(id) = get_all_after_strict(&config1, "/status/") {
                            let url = format!(
                                "https://twitter.com/intent/retweet?tweet_id={}&gleambot=true",
                                id
                            );
                            if let Err(e) = window.open_with_url(&url) {
                                link.send_message(Msg::LogMessage(Message::Error(format!(
                                    "Failed to open a new window: {:?}",
                                    e
                                ))));
                                next();
                                continue;
                            } else {
                                sleep(Duration::from_secs(15)).await;
                                //details = Some(twitter_value.clone());
                            }
                        }
                    }
                } else {
                    link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped retweet in accordance to your settings. Consider enabling auto-retweet to get more entries.".to_string()
                )));
                    next();
                    continue;
                }
            }
            "twitter_tweetA" => {
                if settings.borrow().auto_tweet {
                    if let Some(text) = &entry.config1 {
                        let url = format!(
                            "https://twitter.com/intent/tweet?text={}&gleambot=true",
                            text
                        );
                        if let Err(e) = window.open_with_url(&url) {
                            link.send_message(Msg::LogMessage(Message::Error(format!(
                                "Failed to open a new window: {:?}",
                                e
                            ))));
                            next();
                            continue;
                        } else {
                            sleep(Duration::from_secs(15)).await;
                            //details = Some(twitter_value.clone());
                        }
                    }
                } else {
                    link.send_message(Msg::LogMessage(Message::Info(
                    "Skipped tweet in accordance to your settings. Consider enabling auto-tweet to get more entries.".to_string()
                )));
                    next();
                    continue;
                }
            }
            "custom_action" // Question
                if Verifyer::new(
                    Lacks,
                    Is("question"),
                    Is("Ask a question"),
                    [
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Is("50"),
                    ],
                )
                .matches(entry)
                .is_ok() || Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Is("Ask a question"),
                    [
                        IsNotEmpty,
                        Lacks,
                        IsNotEmpty,
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Is("0"),
                        Lacks,
                        Lacks,
                    ],
                )
                .matches(entry)
                .is_ok() =>
            {
                details = (
                    Value::String(settings.borrow().text_input_sentence.clone()),
                    true,
                    true,
                );
            }
            "custom_action" // Blog comment
                if Verifyer::new(
                    Lacks,
                    Is("blog_comment"),
                    Is("Allow question or tracking"),
                    [
                        IsNotEmpty,
                        Is("comment"),
                        Anything,
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String(settings.borrow().text_input_sentence.clone()), true, true);
            }
            "custom_action" // Basic action
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Is("None"),
                    [
                        IsNotEmpty,
                        Lacks,
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Lacks,
                        Is("0"),
                        Lacks,
                        Lacks,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("Done".to_string()), true, false);
            }
            "custom_action" // Visit website
                if Verifyer::new(
                    IsEmpty,
                    Is("visit"),
                    Is("Use tracking"),
                    [
                        IsNotEmpty,
                        IsNotEmpty,
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Is("simple"),
                        Lacks,
                        Anything, // IsNotEmpty or Lacks
                        Lacks,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("V".to_string()), false, false);
            }
            "custom_action" // Free bonus
                if Verifyer::new(
                    Lacks,
                    Is("bonus"),
                    Is("None"),
                    [
                        IsNotEmpty,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::Null, false, false);
            }
            "instagram_visit_profile"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        Lacks,
                        IsNotEmpty,
                        Lacks,
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() || Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        IsNumber,
                        IsNotEmpty,
                        IsNotEmpty,
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("V".to_string()), false, false);
            }
            "instagram_view_post"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() || Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        Lacks,
                        Lacks,
                        IsNumber,
                        IsNumber,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("Done".to_string()), true, false);
            }
            "facebook_visit"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        IsNotEmpty,
                        IsNumber,
                        Is("Complete"),
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("V".to_string()), false, false);
            }
            "facebook_view_post"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsUrl,
                        IsNotEmpty,
                        Is("post"),
                        Is("1"),
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::Null, false, false);
            }
            "pinterest_visit"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsNotEmpty,
                        Is("Complete"),
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("V".to_string()), false, false); // todo verify and optionnal merge
            }
            "pinterest_visit"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsNotEmpty,
                        Is("Follow"),
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() => 
            {
                details = (Value::String("V".to_string()), true, false);
            }
            "facebook_visit"
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsNotEmpty,
                        IsNotEmpty,
                        IsNumber,
                        Is("Like"),
                        Is("Complete"),
                        IsNumber,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() =>
            {
                details = (Value::String("V".to_string()), true, false);
            }
            "youtube_visit_channel" 
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsNotEmpty,
                        Anything,
                        Is("Complete"),
                        Is("5"),
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() =>
            {
                details = (Value::String("V".to_string()), false, false);
            }
            "twitchtv_follow" 
                if Verifyer::new(
                    Lacks,
                    IsEmpty,
                    Lacks,
                    [
                        IsNotEmpty,
                        IsNumber,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        Lacks,
                        IsEmpty,
                    ],
                )
                .matches(entry)
                .is_ok() =>
            {
                details = (Value::Null, false, false);
            }
            entry_type => {
                log!("unknown entry method");
                if settings.borrow().ban_unknown_methods {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Unsupported entry type: {:?}. Action skipped. You may contact me to request the support of this entry type.",
                        entry_type
                    ))));
                    next();
                    continue;
                } else {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Encountered an unknown entry type: {:?}. The bot will try to enter (since it is enabled in the settings). However, it will likely cause errors that will help gleam.io to detect the bot.",
                        entry_type
                    ))));
                    details = (Value::String("V".to_string()), false, false)
                }
            }
        }

        if entry.requires_details && details.0 == Value::Null {
            err("Null found but expected value".to_string());
            details.0 = Value::String("V".to_string());
        }

        log!("Request: {} -> {:?}", entry.entry_type, details);
        #[cfg(feature = "skip")]
        continue;

        /*if entry.requires_authentication {
            match request::<SetContestantRequest, Contestant>(
                "https://gleam.io/set-contestant",
                Method::Post(SetContestantRequest {
                    additional_details: true,
                    campaign_key: giveaway.campaign.key.clone(),
                    contestant: StoredContestant {
                        competition_subscription: None,
                        date_of_birth: contestant.stored_dob.clone().unwrap_or_else(|| String::from("1950-01-01")),
                        email: contestant.email.clone(),
                        firstname: get_all_before(&contestant.name, " ").to_string(),
                        lastname: get_all_after(&contestant.name, " ").to_string(),
                        name: contestant.name.clone(),
                        send_confirmation: false,
                        stored_dob: contestant.stored_dob.clone().unwrap_or_else(|| String::from("1950-01-01")),
                    },
                }),
                HashMap::new(),
                csrf,
            )
            .await
            {
                Ok(c) => contestant = c,
                Err(e) => {
                    err(format!("Failed to set contestant: {:?}", e));
                    break;
                },
            }
        }*/

        if let Some(timer) = entry.timer_action {
            sleep(Duration::from_secs(timer + 7)).await;
        }

        if details.1 {
            dbg.insert(&entry.id, details.0.clone());
        }
        if details.2 {
            frm.insert(&entry.id, details.0.clone());
        }
        made_requests.push(&entry.id);

        let body = if details.0 != Value::Null {
            json! ({
                "dbg": dbg,
                "details": details,
                "emid": entry.id,
                "f": fpr,
                "frm": frm,
                "grecaptcha_response": null,
                "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key))
            })
        } else {
            json! ({
                "dbg": dbg,
                "emid": entry.id,
                "f": fpr,
                "frm": frm,
                "grecaptcha_response": null,
                "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key))
            })
        };

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
                    "Unexpected response to HTTP request: {:?} {}",
                    e, entry.entry_type
                ))));
                next();
                continue;
            }
        };
        log!("Response: {:?}", rep);

        match rep {
            EntryResponse::Error { error } => match error.as_str() {
                "error_auth_expired" => {
                    link.send_message(Msg::LogMessage(Message::Error(format!(
                        "Gleam.io is unable to check the action. Please login to {}.",
                        entry.provider
                    ))))
                }
                error => {
                    err(format!(
                    "An unknown error occured while trying get entries with the method {:?}: {:?}",
                    entry.entry_type,
                    error));
                    break;
                }
            },
            EntryResponse::RefreshRequired {
                require_campaign_refresh,
            } => {
                if require_campaign_refresh {
                    return Err(Message::Danger("I'm sorry. The bot made a mistake. I think this kind of mistake may result in a fraud suspicion. You should stop using the bot for a little while.".to_string()));
                }
            }
            EntryResponse::AlreadyEntered { .. } => link.send_message(Msg::LogMessage(
                Message::Warning("Already entered!".to_string()),
            )),
            EntryResponse::Success { worth, .. } => {
                if entry.mandatory {
                    completed_mandatory_entries += 1;
                }
                actions_number += 1;
                settings.borrow_mut().total_entries += worth;
                settings.borrow().save();
            }
            EntryResponse::BotSpotted { cheater } => {
                if cheater {
                    return Err(Message::Danger("I'm sorry. Gleam.io detected the bot. You should stop using it for a while. Your account may have been banned for a few weeks.".to_string()));
                }
            }
            EntryResponse::IpBan { ip_ban } => {
                if ip_ban {
                    return Err(Message::Danger(
                        "I'm sorry. Gleam.io banned your IP. There is nothing you can do."
                            .to_string(),
                    ));
                }
            }
        }

        if frm.get(&entry.id).is_none() {
            frm.insert(&entry.id, json! {null});
        }
        dbg.insert(&entry.id, json! {null});

        next();
        sleep(Duration::from_secs(7)).await;
    }

    link.send_message(Msg::Done);

    Ok(())
}
