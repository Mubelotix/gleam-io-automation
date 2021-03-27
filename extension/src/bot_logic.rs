use format::classifier::classify;
use crate::{
    format::*,
    messages::{Message, Message::*},
    request::*,
    settings::*,
    util::*,
    yew_app::{Model, Msg},
};
use format::{prelude::*, contestant::MaybeUninitContestant};
use serde_json::{from_str, json, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};
use string_tools::*;
use web_sys::window;
use yew::prelude::*;
use format::classifier::{EntryType, RequestType};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
    #[cfg(debug_assertions)]
    log!("Giveaway: {:#?}", giveaway);

    // Get the contestant object
    let contestant_json = get_all_between_strict(&main_page, " ng-init='initContestant(", ");")
        .unwrap()
        .replace("&quot;", "\"");
    let init_contestant = match from_str::<InitContestant>(&contestant_json) {
        Ok(g) => g,
        Err(e) => {
            elog!("Contestant json: {}", contestant_json);
            return Err(Error(format!(
                "Failed to parse contestant {}: {:?}.",
                contestant_json, e
            )))
        }
    };
    #[cfg(debug_assertions)]
    log!("Contestant: {:#?}", init_contestant);
    let mut contestant: Contestant = match init_contestant.contestant {
        MaybeUninitContestant::Connected(contestant) => contestant,
        MaybeUninitContestant::Disconnected{..} => {
            return Err(Warning("You have to login to gleam.io to use the bot.".to_string()));
        }
    };

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
    let mut mandatory_entries = Vec::new();
    let mut additionnal_entries = Vec::new();
    for entry in &giveaway.entry_methods {
        if entry.mandatory {
            mandatory_entries.push(entry);
            if contestant.entered.contains_key(&entry.id) {
                completed_mandatory_entries += 1;
            }
        } else {
            additionnal_entries.push(entry);
        }
    }
    mandatory_entries.sort_by(|a, b| a.actions_required.cmp(&b.actions_required));
    additionnal_entries.sort_by(|a, b| a.actions_required.cmp(&b.actions_required));
    let mut entry_methods = mandatory_entries;
    let mandatory_entries: usize = entry_methods.len();
    entry_methods.extend(&additionnal_entries);

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
            return Err(Warning("You enabled a Twitter automation feature, but you must specify your Twitter username in your settings.\nPROCESS ABORTED: Awaiting user correction".to_string()));
        }

        if settings.text_input_sentence.is_empty() {
            warn("Please specify a default \"text input\" value in the settings.".to_string());
        }

        if (settings.auto_retweet || settings.auto_tweet || settings.auto_follow_twitter) && auths.get("twitter").is_none() {
            return Err(Warning("You enabled a Twitter automation feature in your settings, but you are not connected to Twitter on gleam.io. Please link your account before using these functionalities.\nPROCESS ABORTED: Awaiting user correction".to_string()));
        }

        if settings.auto_follow_twitch && auths.get("twitchtv").is_none() {
            return Err(Warning("You enabled automatic Twitch following, but you did not link your Twitch account on gleam.io. Please link your account before using this functionality.\nPROCESS ABORTED: Awaiting user correction".to_string()));
        }
    }

    // Iterate on entries and validate them
    for entry in entry_methods {
        #[cfg(debug_assertions)]
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
            Some(entry_type) => match entry_type {
                EntryType::TwitchFollow => {
                    if !settings.borrow().auto_follow_twitch {
                        warn("Ignored an entry since automatic twitch follow is disabled by your settings.".to_string());
                        next();
                        continue;
                    }
                    entry_type
                }
                entry_type => entry_type,
            },
            None => {
                log!("Unknown entry method: {}\nworkflow: {:?}\ntemplate: {:?}\nmethod_type: {:?}\nconfigs: [\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?}\n]", entry.entry_type, entry.workflow, entry.template, entry.method_type, entry.config1, entry.config2, entry.config3, entry.config4, entry.config5, entry.config6, entry.config7, entry.config8, entry.config9);
                if settings.borrow().display_dev_messages {
                    notify(Warning(format!(
                        "Unknown entry method: {}\nworkflow: {:?}\ntemplate: {:?}\nmethod_type: {:?}\nconfigs: [\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?},\n\t{:?}\n]", entry.entry_type, entry.workflow, entry.template, entry.method_type, entry.config1, entry.config2, entry.config3, entry.config4, entry.config5, entry.config6, entry.config7, entry.config8, entry.config9
                    )));
                }
                next();
                continue;
            }
        };

        let get_config = |index: u8| -> Option<&String> {
            match index {
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
            RequestType::TwitterTweet(data_included) => {
                if data_included && settings.borrow().auto_tweet {
                    // Get the text
                    let text = match &entry.config1 {
                        Some(text) => text,
                        None => {
                            err_next("Invalid twitter entry method 03".to_string());
                            continue;
                        }
                    };
                    
                    // Build the URL
                    let url = format!(
                        "https://twitter.com/intent/tweet?text={}%20{}&gleambot=true",
                        urlencoding::encode(&text.replace("&#39;", "'")),
                        href
                    );

                    // Try to tweet
                    if let Err(e) = window.open_with_url(&url) {
                        err_next(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ));
                        continue;
                    }
                    
                    sleep(Duration::from_secs(15)).await;
                    (twitter_value.clone(), true, true)
                } else if !data_included && settings.borrow().auto_tweet_share {
                    use format::shortener::Shortener;

                    // Get the shortener
                    let (shortener_url, username, api_key) = if let Shortener::WellKnown{ url, username, api_key } = &giveaway.campaign.shortener {
                        (url, username, api_key)
                    } else {
                        warn("The shortener of this giveaway is not supported.".to_string());
                        next();
                        continue;
                    };
                    let share_path = contestant.viral_share_paths.clone();
                    let share_path = match share_path.values().next() {
                        Some(sp) => sp,
                        None => {
                            err_next("No viral_share_path set in contestant object".to_string());
                            continue;
                        }
                    };

                    // Build the closure that will receive the data
                    let window = web_sys::window().unwrap();
                    let url: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
                    let url2 = Rc::clone(&url);
                    let closure = Closure::wrap(Box::new(move |data: JsValue| {
                        if let Ok(value) = data.into_serde::<Value>() {
                            if let Value::String(url) = value["data"]["url"].clone() {
                                *url2.borrow_mut() = Some(url);
                            } else {
                                elog!("Failed to get url");
                            }
                        } else {
                            elog!("Failed to parse json");
                        }
                    }) as Box<dyn FnMut(_)>);
                    js_sys::Reflect::set(&window, &wasm_bindgen::JsValue::from_str("jQuery35108330896762228792_1602615622282"), closure.as_ref().unchecked_ref()).unwrap();
                    closure.forget();

                    // Send the request to the shortener by adding a script element
                    let document = window.document().expect("No document");
                    let panel = document.create_element("script").unwrap();
                    panel.set_attribute("type", "text/javascript").unwrap();
                    panel.set_attribute("src", &format!("{}?callback=jQuery35108330896762228792_1602615622282&login={}&apiKey={}&longUrl=https://gleam.io{}&format=json&_=1602619622282", shortener_url, username, api_key, share_path)).unwrap();
                    document.body().unwrap().append_child(&panel).unwrap();
                    
                    // Wait for the request to succeed and get the response
                    sleep(Duration::from_secs(10)).await;
                    let url = match url.borrow_mut().take() {
                        Some(url) => url,
                        None => {
                            err_next("Failed to exec request to shortener".to_string());
                            continue;
                        }
                    };

                    // Build the tweet using the shortened link
                    let url = format!(
                        "https://twitter.com/intent/tweet?text=I%20found%20the%20best%20giveaway%21%20{}&gleambot=true",
                        url
                    );
                    
                    // Tweet on Twitter
                    if let Err(e) = window.open_with_url(&url) {
                        err_next(format!(
                            "Failed to open a new window: {:?}",
                            e
                        ));
                        continue;
                    }
                    
                    next();
                    continue;
                } else {
                    next();
                    continue;
                }
            }
            RequestType::Answer(separator, i) => {   
                if let Some(answer) = get_config(i).map(|a| a.split(separator).collect::<Vec<&str>>().get(0).map(|a| a.trim().to_string())).flatten() {
                    (
                        Value::String(answer.replace("&#39;", "'")),
                        true,
                        true,
                    )
                } else {
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
            RequestType::Simple(details, prop_dbg, prop_efd) => {
                if entry_type == EntryType::EmailSubscribe && !settings.borrow().auto_email_subscribe {
                    next();
                    continue;
                }
                (details, prop_dbg, prop_efd)
            }
            RequestType::SimpleWithDelay(details, delay) => {
                let delay = get_config(delay).unwrap().parse().unwrap(); // unwrap because values have been checked by the classifier
                sleep(Duration::from_secs(delay)).await;
                details
            }
            RequestType::Enter => (Value::Null, false, false),
            RequestType::Unimplemented(message) => {
                warn(message.to_string());
                next();
                continue;
            }
        };

        // If the entry requires authentification, check that we have a non-expired authentification
        if entry.requires_authentication && !matches!(auths.get(entry.provider.as_str()), Some(false)) {
            warn(format!("Gleam.io wants you to be logged to {}.", entry.provider));
            next();
            continue;
        }

        // Check if we are not going to send an invalid request
        if entry.requires_details && details.0 == Value::Null {
            err_next("Null found but expected value".to_string());
            continue;
        }
        #[cfg(debug_assertions)]
        log!("Request: {} -> {:?}", entry.entry_type, details);

        // Sleep the required time
        if let Some(Value::Number(timer)) = &entry.timer_action {
            if let Some(timer) = timer.as_u64() {
                sleep(Duration::from_secs(timer + 7)).await;
            }
        }

        // Generate remaining parts of the request
        let mut eds: HashMap<&String, Value> = HashMap::new();
        for made_request in &made_requests {
            eds.insert(made_request, Value::Null);
        }
        let mut efd = eds.clone();
        for idx in 0..giveaway.entry_methods.len() {
            let entry = &giveaway.entry_methods[idx];
            if entry.requires_details {
                #[allow(clippy::single_match)]
                match entry.provider.as_str() {
                    "twitter" => {
                        efd.insert(&entry.id, twitter_value.clone());
                    }
                    _ => (),
                }
            }
        }
        if details.1 {
            eds.insert(&entry.id, details.0.clone());
        }
        if details.2 {
            efd.insert(&entry.id, details.0.clone());
        }
        made_requests.push(&entry.id);

        // Build the body of the request
        let body = if details.0 != Value::Null {
            json! ({
                "details": details.0,
                "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key)),
                "grecaptcha_response": null,
                "dbg": {
                    "eds": eds,
                    "afd": {},
                    "efd": efd,
                    "dtc": details.0,
                    "car": false,
                },
                "dbge": {
                    "eed": "5", // sometimes 7, todo: check when
                    "csefr": "rnull",
                    "csefn": "#undefined:true",
                    "hed": format!("#{}:undefined:undefined:undefined", entry.id),
                    "ae": "elc",
                    "aebps": "ae#2",
                },
                "f": fpr,
            })
        } else {
            json! ({
                "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key)),
                "grecaptcha_response": null,
                "dbg": {
                    "eds": eds,
                    "afd": {},
                    "efd": efd,
                    "car": false,
                },
                "dbge": {
                    "eed": "5", // sometimes 7, todo: check when
                    "csefr": "rnull",
                    "csefn": "#undefined:true",
                    "hed": format!("#{}:undefined:undefined:undefined", entry.id),
                    "ae": "elc",
                    "aebps": "ae#2",
                },
                "f": fpr,
            })
        };

        // Avoid sending the request on testing environnement
        #[cfg(feature = "norequest")]
        {
            log!("would have sent:\n{}", body);
            continue;
        }

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
                #[cfg(debug_assertions)]
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
                        "An unknown error occurred while trying to get entries with the method {:?}: {:?}",
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

    link.send_message(Msg::Done(true));
    Ok(())
}
