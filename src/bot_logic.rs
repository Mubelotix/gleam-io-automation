use crate::classifier::classify;
use crate::{
    format::*,
    messages::{Message, Message::*},
    request::*,
    settings::*,
    util::*,
    yew_app::{Model, Msg},
};
use serde_json::{from_str, json, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};
use string_tools::*;
use web_sys::window;
use yew::prelude::*;
use crate::classifier::{EntryType, RequestType};

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

pub async fn run(
    link: Rc<ComponentLink<Model>>,
    settings: Rc<RefCell<Settings>>,
) -> Result<(), Message<String>> {
    let window = window().unwrap();
    let location = window.location();
    let href = location.href().unwrap();

    // Load the giveaway and initialize data
    let main_page = request_str::<()>(&href, Method::Get, "").await.unwrap();
    let fpr = get_fraud();
    let csrf =
        get_all_between_strict(&main_page, "<meta name=\"csrf-token\" content=\"", "\"").unwrap();

    // Get the giveaway object
    let giveaway_json = get_all_between_strict(&main_page, " ng-init='initCampaign(", ")'>")
        .unwrap()
        .replace("&quot;", "\"");
    let giveaway = match from_str::<Giveaway>(&giveaway_json) {
        Ok(g) => g,
        Err(e) => {
            return Err(Error(format!(
                "Failed to parse giveaway {}: {:?}.",
                giveaway_json, e
            )))
        }
    };
    log!("Giveaway: {:#?}", giveaway);

    // Get the contestant object
    let contestant_json = get_all_between_strict(&main_page, " ng-init='initContestant(", ");")
        .unwrap()
        .replace("&quot;", "\"");
    let init_contestant = match from_str::<InitContestant>(&contestant_json) {
        Ok(g) => g,
        Err(e) => {
            return Err(Error(format!(
                "Failed to parse contestant {}: {:?}.",
                contestant_json, e
            )))
        }
    };
    log!("Contestant: {:#?}", init_contestant);
    let mut contestant = init_contestant.contestant;

    // Update the contestant if needed
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
            csrf,
        )
        .await
        {
            Ok(c) => contestant = c.contestant,
            Err(e) => {
                return Err(Error(format!("Failed to set contestant: {:?}", e)));
            }
        }
    }

    // Initialize variables
    let twitter_value = json! ({
        "twitter_username": settings.borrow().twitter_username,
    });
    let current: RefCell<f32> = RefCell::new(0.0);
    let len = giveaway.entry_methods.len() as f32;
    let mut made_requests: Vec<&String> = Vec::new();
    let mut completed_mandatory_entries: usize = 0;
    let mut actions_number = 0;

    // Order the entries
    let mut mandatory_entries: usize = 0;
    let mut entry_methods = Vec::new();
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

    // Get the linked accounts
    let mut auths = HashMap::new();
    for authentification in &contestant.authentications {
        auths.insert(authentification.provider.as_str(), authentification.expired);
    }

    // Create a closure to update the progress bar
    let next = || {
        use std::ops::AddAssign;
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize,
        ));
    };

    // Create a closure displaying a message
    let notify = |m: Message<String>| {
        link.send_message(Msg::LogMessage(m));
    };

    // Create a closure updating the progress bar and displaying an error message
    let err_next = |e: String| {
        elog!("ERROR: {}", e);
        use std::ops::AddAssign;
        notify(Error(e));
        current.borrow_mut().add_assign(1.0);
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * *current.borrow()).floor() as usize,
        ));
    };

    // Create a closure displaying a warning
    let warn = |m: String| {
        log!("Warning: {}", m);
        link.send_message(Msg::LogMessage(Warning(m)));
    };

    // Check settings and display warnings
    {
        let settings = settings.borrow();
        if settings.twitter_username.is_empty()
            && (settings.auto_retweet || settings.auto_tweet || settings.auto_follow_twitter)
        {
            warn("Please specify your Twitter username in the settings.".to_string());
        }

        if settings.text_input_sentence.is_empty() {
            warn("Please specify a default \"text input\" value in the settings.".to_string());
        }
    }

    // Iterate on entries and validate them
    for entry in entry_methods {
        log!("Entry: {:#?}", entry);

        // Check if we can validate this entry
        #[cfg(not(feature = "norequest"))]
        if contestant.entered.contains_key(&entry.id) {
            log!("Already entered, skipping");
            next();
            continue;
        } else if !entry.mandatory && completed_mandatory_entries < mandatory_entries {
            warn("Unable to try some entry methods because some mandatory entry methods were not successfully completed.".to_string());
            return Ok(());
        } else if entry.actions_required > actions_number {
            warn("Unable to try an entry method because it requires more actions to be done.".to_string());
            next();
            continue;
        }

        // Analyse the entry
        let entry_type = match classify(entry) {
            Some(entry_type) => entry_type,
            None => {
                log!("Unknown entry method:\n\tworkflow: {:?}\n\ttemplate: {:?}\n\tmethod_type: {:?}\n\tconfigs: [{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}]", entry.workflow, entry.template, entry.method_type, entry.config1, entry.config2, entry.config3, entry.config4, entry.config5, entry.config6, entry.config7, entry.config8, entry.config9);
                notify(Warning(format!(
                    "Unsupported entry type: {:?}. Action skipped. You may contact me to request the support of this entry type.",
                    entry.entry_type
                )));
                next();
                continue;
            }
        };

        // Generate a the root of a validation request
        let details: (Value, bool, bool) = match entry_type.get_request_type() {
            RequestType::TwitterFollow => {
                if settings.borrow().auto_follow_twitter {
                    let username = match &entry.config1 {
                        Some(username) => username,
                        None => {
                            err_next("Invalid twitter entry method 00".to_string());
                            continue;
                        }
                    };
    
                    let url = format!(
                        "https://twitter.com/intent/follow?screen_name={}&gleambot=true",
                        username
                    );
    
                    if let Err(e) = window.open_with_url(&url) {
                        err_next(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ));
                        continue;
                    } else {
                        sleep(Duration::from_secs(15)).await;
                    }
    
                    (twitter_value.clone(), true, true)
                } else {
                    next();
                    continue;
                }
            }
            RequestType::TwitterRetweet => {
                if settings.borrow().auto_retweet {
                    let url = match &entry.config1 {
                        Some(url) => url,
                        None => {
                            err_next("Invalid twitter entry method 01".to_string());
                            continue;
                        }
                    };
    
                    let id = match get_all_after_strict(&url, "/status/") {
                        Some(id) => id,
                        None => {
                            err_next("Invalid twitter entry method 02".to_string());
                            continue;
                        }
                    };
    
                    let url = format!(
                        "https://twitter.com/intent/retweet?tweet_id={}&gleambot=true",
                        get_all_before(id, "?")
                    );
    
                    if let Err(e) = window.open_with_url(&url) {
                        err_next(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ));
                        continue;
                    }
    
                    sleep(Duration::from_secs(15)).await;
                    (twitter_value.clone(), true, true)
                } else {
                    next();
                    continue;
                }
            }
            RequestType::TwitterTweet => {
                if settings.borrow().auto_tweet {
                    let text = match &entry.config1 {
                        Some(text) => text,
                        None => {
                            err_next("Invalid twitter entry method 03".to_string());
                            continue;
                        }
                    };
                    
                    let url = format!(
                        "https://twitter.com/intent/tweet?text={} {}&gleambot=true",
                        text.replace("&#39;", "'"),
                        href
                    );
                    
                    if let Err(e) = window.open_with_url(&url) {
                        err_next(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ));
                        continue;
                    }
    
                    sleep(Duration::from_secs(15)).await;
                    (twitter_value.clone(), true, true)
                } else {
                    next();
                    continue;
                }
            }
            RequestType::Answer(separator, i) => {   
                let config = match i {
                    1 => entry.config1.as_ref(),
                    2 => entry.config2.as_ref(),
                    3 => entry.config3.as_ref(),
                    4 => entry.config4.as_ref(),
                    5 => entry.config5.as_ref(),
                    6 => entry.config6.as_ref(),
                    7 => entry.config7.as_ref(),
                    8 => entry.config8.as_ref(),
                    9 => entry.config9.as_ref(),
                    _ => panic!("Invalid entry index"),
                };

                if let Some(answer) = config.map(|a| a.split(separator).collect::<Vec<&str>>().get(0).map(|a| a.trim().to_string())).flatten() {
                    (
                        Value::String(answer.replace("&#39;", "'")),
                        true,
                        true,
                    )
                } else {
                    log!("WARNING: Answers not given {:?}", entry_type);
                    (
                        Value::String(settings.borrow().text_input_sentence.clone()),
                        true,
                        true,
                    )
                }
            }
            RequestType::TextInput => {
                (
                    Value::String(settings.borrow().text_input_sentence.clone()),
                    true,
                    true,
                )
            }
            RequestType::Simple(details, prop_dbg, prop_frm) => {
                if entry_type == EntryType::EmailSubscribe && !settings.borrow().auto_email_subscribe {
                    next();
                    continue;
                }
                (details, prop_dbg, prop_frm)
            }
            RequestType::Enter(service) => {
                match auths.get(service) {
                    Some(false) => (Value::Null, false, false),
                    Some(true) => {
                        warn(format!("Your {} authentification expired. Please login to this account again.", service));
                        next();
                        continue;
                    },
                    None => {
                        warn(format!("Please link your {} account to gleam.", service));
                        next();
                        continue;
                    }
                }
            }
        };

        // Check if we are not going to send an invalid request
        if entry.requires_details && details.0 == Value::Null {
            err_next("Null found but expected value".to_string());
            continue;
        }
        log!("Request: {} -> {:?}", entry.entry_type, details);

        // Sleep the required time
        if let Some(timer) = entry.timer_action {
            sleep(Duration::from_secs(timer + 7)).await;
        }

        // Avoid sending the request on testing environnement
        #[cfg(feature = "norequest")]
        continue;

        // Generate remaining parts of the request
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
        if details.1 {
            dbg.insert(&entry.id, details.0.clone());
        }
        if details.2 {
            frm.insert(&entry.id, details.0.clone());
        }
        made_requests.push(&entry.id);

        // Build the body of the request
        let body = if details.0 != Value::Null {
            json! ({
                "dbg": dbg,
                "details": details.0,
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

        // Send the request
        let response = match request::<Value, EntryResponse>(
            &format!(
                "https://gleam.io/enter/{}/{}",
                giveaway.campaign.key, entry.id
            ),
            Method::Post(body),
            csrf,
        )
        .await
        {
            Ok(response) => {
                log!("Response: {:?}", response);
                response
            }
            Err(()) => return Err(Error("Invalid response to HTTP request!".to_string())),
        };

        // Check the result
        match response {
            EntryResponse::Error { error } => match error.as_str() {
                "error_auth_expired" => {
                    notify(Error(format!(
                        "Gleam.io is unable to check the action. Please login to {}.",
                        entry.provider
                    )));
                }
                error => {
                    err_next(format!(
                        "An unknown error occured while trying get entries with the method {:?}: {:?}",
                        entry.entry_type,
                        error
                    ));
                    break;
                }
            },
            EntryResponse::RefreshRequired {
                require_campaign_refresh,
            } => {
                if require_campaign_refresh {
                    return Err(Danger("I'm sorry. The bot made a mistake. I think this kind of mistake may result in a fraud suspicion. You should stop using the bot for a little while.".to_string()));
                }
            }
            EntryResponse::AlreadyEntered { .. } => warn(
                "Gleam.io says you have already entered, but they are highly unlikely to be right.".to_string(),
            ),
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
                    return Err(Danger("I'm sorry. Gleam.io says you are a cheater. You should stop using the bot for a while. Your account may have been banned for a few weeks. If the problem persists, try changing your ip (use your 4g) and your account.".to_string()));
                }
            }
            EntryResponse::IpBan { ip_ban } => {
                if ip_ban {
                    return Err(Danger(
                        "I'm sorry. Gleam.io banned your IP. There is nothing you can do. If you are using a VPN, don't."
                            .to_string(),
                    ));
                }
            }
        }

        next();
        sleep(Duration::from_secs(7)).await;
    }

    link.send_message(Msg::Done);
    Ok(())
}
