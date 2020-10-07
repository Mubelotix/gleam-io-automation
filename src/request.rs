use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

pub enum Method<T: Serialize> {
    Get,
    Post(T),
}

impl<T: Serialize> Into<&str> for &Method<T> {
    fn into(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post(_) => "POST",
        }
    }
}

impl<T: Serialize> Method<T> {
    pub fn body_to_string(&self) -> Option<String> {
        match self {
            Method::Get => None,
            Method::Post(v) => match serde_json::to_string(v) {
                Ok(v) => Some(v),
                Err(e) => {
                    elog!(
                        "Failed to serialize body data in the request! ERROR: {:?}",
                        e
                    );
                    None
                }
            },
        }
    }
}

pub async fn request_str<T: Serialize>(
    url: &str,
    method: Method<T>,
    csrf: &str,
) -> Result<String, ()> {
    let document: HtmlDocument = window().unwrap().document().unwrap().dyn_into().unwrap();
    let cookies = document.cookie().unwrap();
    let xsrf = string_tools::get_all_between(&cookies, "; XSRF-TOKEN=", ";");

    let hdrs = Headers::new().unwrap();
    hdrs.append("x-xsrf-token", xsrf).unwrap();
    if !csrf.is_empty() {
        hdrs.append("x-csrf-token", csrf).unwrap();
        hdrs.append("accept", "application/json, text/plain, */*")
            .unwrap();
        hdrs.append("content-type", "application/json;charset=UTF-8")
            .unwrap();
    }

    let mut r_init = RequestInit::new();
    r_init.method((&method).into());
    r_init.body(method.body_to_string().map(JsValue::from).as_ref());
    r_init.headers(hdrs.as_ref());

    let window = web_sys::window().unwrap();
    let response = match JsFuture::from(window.fetch_with_str_and_init(url, &r_init)).await {
        Ok(response) => Response::from(response),
        Err(e) => {
            elog!("Network error: {:?}", e);
            return Err(());
        }
    };

    let text = match response.text() {
        Ok(text) => match JsFuture::from(text).await {
            Ok(text) => text,
            Err(e) => {
                elog!("Invalid response: {:?}", e);
                return Err(());
            }
        },
        Err(e) => {
            elog!("Invalid response: {:?}", e);
            return Err(());
        }
    };

    let text = match text.as_string() {
        Some(text) => text,
        None => {
            elog!("Response is not a string!");
            return Err(());
        }
    };

    Ok(text)
}

pub async fn request<T: Serialize, V: DeserializeOwned>(
    url: &str,
    method: Method<T>,
    csrf: &str,
) -> Result<V, ()> {
    let text = request_str(url, method, csrf).await?;

    let value = match serde_json::from_str::<V>(&text) {
        Ok(value) => value,
        Err(e) => {
            elog!("Failed to parse response as json! ERROR: {:?}", e);
            return Err(());
        }
    };

    Ok(value)
}
