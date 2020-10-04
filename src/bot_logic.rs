use crate::enums::*;
use crate::util::*;
use crate::{
    messages::Message,
    yew_app::{Model, Msg},
};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use string_tools::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use crate::request::*;
use web_sys::window;
use yew::prelude::*;
use std::collections::HashMap;
use crate::format::*;
use serde_json::{Value, json};

fn get_fraud() -> String {
    js_sys::eval("fraudService.hashedFraud()").unwrap().as_string().unwrap()
}

fn jsmd5(input: &str) -> String {
    js_sys::eval(&format!(r#"jsmd5("{}")"#, input)).unwrap().as_string().unwrap()
}

pub async fn run(
    link: Rc<ComponentLink<Model>>,
    infos: Arc<Mutex<(String, String)>>,
) -> Result<(), Message<String>> {
    let window = window().unwrap();
    let location = window.location();
    let href = location.href().unwrap();
    
    let main_page = request_str::<()>(&href, Method::Get, HashMap::new(), "").await.unwrap();

    log!("{}", main_page);
    let fpr = get_fraud();
    log!("{}", fpr);

    let text = main_page;
    let csrf =
        string_tools::get_all_between_strict(&text, "<meta name=\"csrf-token\" content=\"", "\"")
            .unwrap();
    let json = string_tools::get_all_between_strict(
        &text,
        " ng-init='initCampaign(",
        ")'>",
    )
    .unwrap();
    let json = json.replace("&quot;", "\"");
    let giveaway = match serde_json::from_str::<Giveaway>(&json) {
        Ok(g) => g,
        Err(e) => {
            elog!("e {:?}", e);
            panic!("{}", &json[e.column() - 10..]);
        }
    };
    log!("giveaway: {:#?}", giveaway);

    let json = string_tools::get_all_between_strict(
        &text,
        " ng-init='initContestant(",
        ");",
    )
    .unwrap();
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

    let mut dbg: HashMap<&String, Value> = HashMap::new();
    let mut frm: HashMap<&String, Value> = HashMap::new();
    for idx in 0..giveaway.entry_methods.len() {
        let entry = &giveaway.entry_methods[idx];
        if entry.requires_details {
            match entry.provider.as_str() {
                "twitter" => {frm.insert(&entry.id, json!{{
                    "twitter_username": "UndefinedUndef9",
                }});},
                p => println!("unknown provider {}", p)
            }
        }
    }

    let mut current: f32 = 0.0;
    let len = giveaway.entry_methods.len() as f32;
    for idx in 0..giveaway.entry_methods.len() {
        let entry = &giveaway.entry_methods[idx];

        let body = json! {{
            "dbg": dbg,
            "details": "V",
            "emid": entry.id,
            "f": fpr,
            "frm": frm,
            "grecaptcha_response": null,
            "h": jsmd5(&format!("-{}-{}-{}-{}", contestant.id, entry.id, entry.entry_type, giveaway.campaign.key))
        }};
        println!("{}", body);

        let rep = request::<Value, EntryResponse>(&format!("https://gleam.io/enter/{}/{}", giveaway.campaign.key, entry.id), Method::Post(body), HashMap::new(), csrf).await.unwrap();

        println!("{:?}", rep);

        match rep {
            EntryResponse::Error{error: e} => link.send_message(Msg::LogMessage(Message::Error(format!("An error occured while trying to confirm an entry: {:?}", e)))),
            EntryResponse::RefreshRequired{ require_campaign_refresh: b } => link.send_message(Msg::LogMessage(Message::Warning(format!("Reload required: {}", b)))),
            EntryResponse::AlreadyEntered{difference,..} => link.send_message(Msg::LogMessage(Message::Warning(format!("Already entered {} minutes ago", difference / 3600)))),
            EntryResponse::Success{new: _, worth: _} => (),
        }

        if frm.get(&entry.id).is_none() {
            frm.insert(&entry.id, json!{null});
        }
        dbg.insert(&entry.id, json!{null});

        current += 1.0;
        link.send_message(Msg::ProgressChange(
            ((100.0 / len) * current).floor() as usize
        ));

        sleep(Duration::from_secs(5)).await;
    }

    link.send_message(Msg::Done);

    Ok(())
}
