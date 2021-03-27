use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::Node;
use yew::virtual_dom::VNode;

pub fn get_random_placeholder() -> &'static str {
    use web_sys::*;

    let window = window().unwrap();
    let crypto = window.crypto().unwrap();
    let mut random = [0];
    crypto.get_random_values_with_u8_array(&mut random).unwrap();
    match random[0] % 50 {
        0 => "rp",
        1 => "vbucks",
        2 => "dollars",
        3 => "euros",
        4 => "bitcoin",
        5 => "steam",
        6 => "paypal",
        7 => "amazon",
        8 => "epic game",
        9 => "xbox",
        10 => "PS4",
        11 => "PS5",
        12 => "gift card",
        13 => "steam gift gard",
        14 => "amazon gift gard",
        15 => "GTA",
        16 => "minecraft",
        17 => "animal crossing",
        18 => "fortnite",
        19 => "iphone",
        21 => "game",
        22 => "AMD",
        23 => "nintendo switch",
        24 => "robux",
        25 => "PC",
        26 => "nvidia",
        27 => "geforce",
        28 => "neuralink's brain interface",
        29 => "pubg",
        31 => "netflix",
        32 => "mobile",
        33 => "phone",
        34 => "CPU",
        35 => "RPG",
        36 => "skin",
        37 => "google",
        38 => "coin",
        39 => "controler",
        40 => "zelda",
        41 => "nintendo",
        42 => "sony",
        43 => "headset",
        44 => "microphone",
        45 => "sims",
        46 => "wireless",
        47 => "speaker",
        48 => "royal pass",
        _ => "steam key",
    }
}

pub fn unescaped_html(html: &str) -> VNode {
    let element = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    element.set_inner_html(html);

    VNode::VRef(Node::from(element))
}

#[wasm_bindgen(module = "/src/js.js")]
extern "C" {
    fn get_timestamp_js() -> u32;
}

pub fn get_timestamp() -> u64 {
    get_timestamp_js() as u64
}

pub fn seconds_to_string(mut seconds: i64, compact: bool) -> String {
    let ago = seconds < 0;

    let mut remaining = seconds%86400;
    let mut days = (seconds - remaining) / 86400;
    seconds = remaining;
    remaining = seconds%3600;
    let mut hours = (seconds - remaining) / 3600;
    seconds = remaining;
    remaining = seconds%60;
    let mut minutes = (seconds - remaining) / 60;
    seconds = remaining;

    if ago {
        days *= -1;
        hours *= -1;
        minutes *= -1;
        seconds *= -1;
    }

    let mut result = String::new();
    if days != 0 {
        result.push_str(&days.to_string());
        result.push_str(" days");
        if hours != 0 && !compact {
            result.push_str(" and ");
            result.push_str(&hours.to_string());
            result.push_str(" hours");
        }
    } else if hours != 0 {
        result.push_str(&hours.to_string());
        result.push_str(" hours");
        if minutes != 0 && !compact {
            result.push_str(" and ");
            result.push_str(&minutes.to_string());
            result.push_str(" minutes");
        }
    } else if minutes != 0 {
        result.push_str(&minutes.to_string());
        result.push_str(" minutes");
        if seconds != 0 && !compact {
            result.push_str(" and ");
            result.push_str(&seconds.to_string());
            result.push_str(" seconds");
        }
    } else if seconds != 0 {
        result.push_str(&seconds.to_string());
        result.push_str(" seconds");
    } else {
        result.push_str("now");
    }

    if ago {
        result.push_str(" ago");
    }

    result
}