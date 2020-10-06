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
    let contestant = init_contestant.contestant;

    let twitter_value = json! {{
        "twitter_username": settings.borrow().twitter_username,
    }};

    let current: RefCell<f32> = RefCell::new(0.0);
    let len = giveaway.entry_methods.len() as f32;

    let next = || {
        use std::ops::AddAssign;
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize
        ));
    };

    let err = |e: String| {
        use std::ops::AddAssign;
        link.send_message(Msg::LogMessage(Message::Error(
            e
        )));
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize
        ));
    };

    let testing = |s: &'static str| {
        link.send_message(Msg::LogMessage(Message::Info(
            format!("The bot tried a known entry method that has never been tested: {}. Did it work?", s)
        )));
    };

    let mut made_requests: Vec<&String> = Vec::new();
    let mut mandatory_entries: usize = 0;
    let mut completed_mandatory_entries: usize = 0;
    let mut entry_methods = Vec::new();
    let mut entries_number = 0;
    let mut actions_number = 0;
    for entry in &giveaway.entry_methods {
        if entry.mandatory {
            entry_methods.insert(mandatory_entries, entry);
            mandatory_entries += 1;
        } else {
            entry_methods.push(entry);
        }
    }

    if giveaway.campaign.additional_contestant_details {
        link.send_message(Msg::LogMessage(Message::Error(
            "Unsupported giveaway. Just tell me one thing: does it requires you to enter contact info?".to_string()
        )));
        sleep(Duration::from_secs(5)).await;
        panic!("");
    }

    for entry in entry_methods {
        if contestant.entered.contains_key(&entry.id) {
            log!("Already entered, skipping");
            next();
            continue;
        } else if !entry.mandatory && completed_mandatory_entries < mandatory_entries {
            link.send_message(Msg::LogMessage(Message::Warning(
                "Unable to try some entry methods because some mandatory entry methods were not successfully completed.".to_string()
            )));
            return Ok(())
        } else if entry.actions_required > actions_number {
            link.send_message(Msg::LogMessage(Message::Warning(
                "Unable to try an entry method because it requires more actions to be done.".to_string()
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
                        frm.insert(
                            &entry.id, twitter_value.clone(),
                        );
                    },
                    _ => (),
                }
            }
        }

        let mut details = (Value::Null, false, false);
        match entry.entry_type.as_str() {
            "twitter_followA" => if settings.borrow().auto_follow_twitter {
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

                let url = format!("https://twitter.com/intent/follow?screen_name={}&gleambot=true", username);
                
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
            },
            "twitter_retweetA" => if settings.borrow().auto_retweet {
                if let Some(config1) = &entry.config1 {
                    if let Some(id) = get_all_after_strict(&config1, "/status/") {
                        let url = format!("https://twitter.com/intent/retweet?tweet_id={}&gleambot=true", id);
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
            "twitter_tweetA" => if settings.borrow().auto_tweet {
                if let Some(text) = &entry.config1 {
                    let url = format!("https://twitter.com/intent/tweet?text={}&gleambot=true", text);
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
            "custom_action" => {
                match entry.template.as_str() {
                    "choose_optionA" => {
                        let answers: Vec<&str> = match &entry.config6 {
                            Some(entries) => entries.split("\r\n").collect(),
                            None => {
                                err("A custom_action with the choose_option template is expected to store answers in config6.".to_string());
                                continue;
                            }
                        };

                        if entry.config2 != Some("unique".to_string()) {
                            err(format!("Unsupported parameter config2={:?} in a custom_action with the choose_option template.", entry.config2));
                            continue;
                        }

                        let answer = match answers.get(0) {
                            Some(answer) => answer,
                            None => {
                                err("The custom_action method with the choose_option template has 0 answer.".to_string());
                                continue;
                            }
                        };

                        //details = Some(Value::String(answer.to_string()));
                        testing("custom_action > choose_option > unique");
                    }
                    "visit" => {
                        match entry.workflow.as_deref() {
                            Some("VisitQuestionA") => {
                                //details = Some(Value::String("Îmi pare rău, nu înțeleg ce ar trebui să scriu aici.".to_string()));
                                testing("custom_action > visit > Some(\"VisitQuestion\")");
                            }
                            Some("VisitAuto") => {
                                details = (Value::String("V".to_string()), false, false);
                                testing("custom_action > visit > Some(\"VisitAuto\")");
                            }
                            Some("") => {
                                details = (Value::String("V".to_string()), false, false);
                            }
                            workflow => {
                                err(format!("custom_action with template visit has an unknown workflow: {:?}", workflow));
                                continue;
                            }
                        }
                    },
                    "" => {
                        details = (Value::String("Done".to_string()), true, false);
                    }
                    "question" => {
                        details = (Value::String("Îmi pare rău, nu înțeleg ce ar trebui să scriu aici.".to_string()), true, true);
                        testing("question");
                    }
                    "blog_comment" => {
                        details = (Value::String("Îmi pare rău, nu înțeleg ce ar trebui să scriu aici.".to_string()), true, true);
                    }
                    "bonus" => {
                        details = (Value::Null, false, false);
                        testing("bonus");
                    }
                    template => {
                        err(format!("custom_action with unknown template: {:?}", template));
                        continue
                    },
                }
            }
            "twitchtv_follow" => {
                details = (Value::Null, false, false);
                testing("twitchtv_follow");
            }
            "facebook_visit" => {
                details = (Value::String("V".to_string()), true, false);
            }
            "youtube_visit_channel" => {
                details = (Value::String("V".to_string()), false, false);
            }
            "instagram_visit_profile" => {
                details = (Value::String("V".to_string()), false, false);
            }
            "pinterest_visit" => {
                details = (Value::String("V".to_string()), false, false);
            }
            "facebook_view_post" => {
                details = (Value::Null, false, false);
            }
            "youtube_enter" => {
                details = (Value::Null, false, false);
                testing("youtube_enter");
            }
            "instagram_view_post" => {
                details = (Value::String("Done".to_string()), true, false);
            }
            entry_type => {
                if settings.borrow().ban_unknown_methods {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Encountered an unknown entry type: {:?}. This entry method has been skipped. You can enable auto-entering for unknown entry methods in the settings, but it may not work properly.",
                        entry_type
                    ))));
                    next();
                    continue;
                } else {
                    link.send_message(Msg::LogMessage(Message::Warning(format!(
                        "Encountered an unknown entry type: {:?}. The bot will try to enter (since it is enabled in the settings). Could you please report this unknown entry method by opening an issue on Github or by sending an email to mubelotix@gmail.com? Please mention the url of this giveaway in your report. That would help me a lot to extend and improve the bot support. Thank you very much.",
                        entry_type
                    ))));
                }
            },
        }

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
                    "Unexpected response to HTTP request: {:?} {}",
                    e, entry.entry_type
                ))));
                next();
                continue;
            }
        };
        log!("response: {:?}", rep);

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
                },
            },
            EntryResponse::RefreshRequired {
                require_campaign_refresh,
            } => {
                if require_campaign_refresh {
                    return Err(Message::Danger("I'm sorry. The bot made a mistake. I think this kind of mistake may result in a fraud suspicion. You should stop using the bot for a little while.".to_string()))
                }
            },
            EntryResponse::AlreadyEntered { .. } => link.send_message(Msg::LogMessage(
                Message::Warning("Already entered!".to_string()),
            )),
            EntryResponse::Success { worth, .. } => {
                if entry.mandatory {
                    completed_mandatory_entries += 1;
                }
                entries_number += worth;
                actions_number += 1;
                link.send_message(Msg::LogMessage(Message::Info(format!(
                    "{} worked!", entry.entry_type
                ))));
                settings.borrow_mut().total_entries += worth;
                settings.borrow().save();
            },
            EntryResponse::BotSpotted { cheater } => {
                if cheater {
                    return Err(Message::Danger("I'm sorry. Gleam.io detected the bot. You should stop using it for a while. Your account may have been banned for a few weeks.".to_string()))
                }
            }
            EntryResponse::IpBan { ip_ban } => {
                if ip_ban {
                    return Err(Message::Danger("I'm sorry. Gleam.io banned your IP. There is nothing you can do.".to_string()))
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
