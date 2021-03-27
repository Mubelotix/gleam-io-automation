use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::{prelude::*, incentive::IncompleteIncentive, entry::IncompleteEntryMethod, campaign::IncompleteCampaign};

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Giveaway {
    pub entry_state: Value,
    pub entry_details_state: Value,
    pub app_name: String,
    pub campaign: Campaign,
    #[serde(rename = "entry_methods")]
    pub entry_methods: Vec<EntryMethod>,
    pub incentive: Incentive,
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncompleteGiveaway {
    #[serde(flatten)]
    pub campaign: IncompleteCampaign,
    #[serde(rename = "entry_methods")]
    pub entry_methods: Vec<IncompleteEntryMethod>,
    #[serde(flatten)]
    pub incentive: IncompleteIncentive,
}

impl From<Giveaway> for IncompleteGiveaway {
    fn from(giveaway: Giveaway) -> IncompleteGiveaway {
        IncompleteGiveaway {
            campaign: giveaway.campaign.into(),
            entry_methods: {
                let mut incomplete_entry_methods = Vec::new();
                for entry_method in giveaway.entry_methods {
                    incomplete_entry_methods.push(entry_method.into())
                }
                incomplete_entry_methods
            },
            incentive: giveaway.incentive.into(),
        }
    }
}

#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub giveaway: IncompleteGiveaway,
    pub last_updated: u64,
    pub referers: Vec<String>,
    pub entry_count: Option<usize>,
    pub entry_evolution: Option<HashMap<String, usize>>,
}

impl std::ops::Add for SearchResult {
    type Output = SearchResult;

    fn add(self, mut rhs: SearchResult) -> Self::Output {
        let mut referers = self.referers;
        referers.append(&mut rhs.referers);
        referers.sort();
        referers.dedup();

        let entry_evolution = if let (Some(e1), Some(e2)) = (self.entry_evolution, rhs.entry_evolution) {
            if self.last_updated > rhs.last_updated {
                let mut entry_evolution = e2;
                for (time, entries) in e1.iter() {
                    entry_evolution.insert(time.clone(), *entries);
                }
                Some(entry_evolution)
            } else {
                let mut entry_evolution = e1;
                for (time, entries) in e2.iter() {
                    entry_evolution.insert(time.clone(), *entries);
                }
                Some(entry_evolution)
            }
        } else {
            None
        };

        if self.last_updated > rhs.last_updated {
            SearchResult {
                giveaway: self.giveaway,
                last_updated: self.last_updated,
                referers,
                entry_count: self.entry_count,
                entry_evolution,
            }
        } else {
            SearchResult {
                giveaway: rhs.giveaway,
                last_updated: rhs.last_updated,
                referers,
                entry_count: rhs.entry_count,
                entry_evolution,
            }
        }
    }
}

impl SearchResult {
    pub fn ends_at(&self) -> u64 {
        self.giveaway.campaign.ends_at
    }

    pub fn get_url(&self) -> String {
        format!("https://gleam.io/{}/-", self.giveaway.campaign.key)
    }

    pub fn get_name(&self) -> &str {
        &self.giveaway.campaign.name
    }
}

impl meilisearch_sdk::document::Document for SearchResult {
    type UIDType = String;
    fn get_uid(&self) -> &Self::UIDType {
        &self.giveaway.campaign.key
    }
}

#[test]
fn test() {
    println!("{}", serde_json::to_string_pretty(&SearchResult {
        giveaway: IncompleteGiveaway {
            campaign: IncompleteCampaign {
                starts_at: 0,
                ends_at: 0,
                key: String::from("abc"),
                name: String::from("campaign name"),
                language: String::from("en"),
                site_name: String::from("pseudo's website"),
                site_url: String::from("pseudo.com"),
                finished: false,
                paused: false,
                login_types: Vec::new(),
                stand_alone_url: String::from("https://gleam.io/dzdiz-zdqzddz-qdz"),
                campaign_type: String::from(""),
                terms_and_conditions: String::from("don't cheat"),
                announce_winners: true,
            },
            entry_methods: Vec::new(),
            incentive: IncompleteIncentive {
                name: String::from("hollow knight"),
                description: String::from("a beautiful game"),
                incentive_type: String::from("Prize")
            },
        },
        last_updated: 0,
        referers: Vec::new(),
        entry_count: None,
        entry_evolution: None,
    }).unwrap())
}