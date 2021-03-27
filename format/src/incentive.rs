use serde::{Serialize, Deserialize};
use serde_json::Value;

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Incentive {
    pub id: usize,
    pub name: String,
    pub actions_required: usize,
    pub description: String,
    pub data_type: String,
    pub input_type: Option<Value>,
    pub incentive_type: String,
    pub layout: String,
    #[serde(flatten)]
    pub image: Option<IncentiveImage>,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct IncentiveImage {
    pub url: String,
    pub medium_url: String,
    pub image_height: usize,
    pub image_width: usize,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteIncentive {
    #[serde(rename = "incentive_name")]
    pub name: String,
    #[serde(rename = "incentive_description")]
    pub description: String,
    pub incentive_type: String,
}

impl From<Incentive> for IncompleteIncentive {
    fn from(incentive: Incentive) -> Self {
        IncompleteIncentive {
            name: incentive.name,
            description: incentive.description,
            incentive_type: incentive.incentive_type,
        }
    }
}