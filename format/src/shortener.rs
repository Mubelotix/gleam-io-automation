use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[serde(untagged)]
pub enum Shortener {
    WellKnown {url: String, username: String, api_key: String},
    Strange {url: String, method: String, r#type: String },
    Unknown(Value),
}
