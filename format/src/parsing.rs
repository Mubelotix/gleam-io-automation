use crate::prelude::*;
use string_tools::get_all_between_strict as gabs;

#[derive(Debug)]
pub enum ParseError {
    GiveawayJsonNotFound,
    ContestantJsonNotFound,
    InvalidEntryCount(std::num::ParseIntError),
    GiveawayFormatError(serde_json::Error),
    ContestantFormatError(serde_json::Error),
}

pub fn parse_html(html: &str) -> Result<(Giveaway, InitContestant, Option<usize>), ParseError> {
    let json = match gabs(&html, " ng-init='initCampaign(", ")'>") {
        Some(json) => json,
        None => return Err(ParseError::GiveawayJsonNotFound),
    };
    let json = json.replace("&quot;", "\"").replace("&#39;", "'");
    
    let giveaway = match serde_json::from_str::<Giveaway>(&json) {
        Ok(g) => g,
        Err(e) => return Err(ParseError::GiveawayFormatError(e)),
    };

    let entry_count = match gabs(html, "initEntryCount(", ")") {
        Some(count) if !count.is_empty() => match count.parse() {
            Ok(count) => Some(count),
            Err(e) => return Err(ParseError::InvalidEntryCount(e)),
        }
        _ => None,
    };

    let contestant_json = match gabs(&html, " ng-init='initContestant(", ");") {
        Some(json) => json,
        None => return Err(ParseError::ContestantJsonNotFound),
    };
    let contestant_json = contestant_json.replace("&quot;", "\"").replace("&#39;", "'");

    let init_contestant = match serde_json::from_str::<InitContestant>(&contestant_json) {
        Ok(g) => g,
        Err(e) => return Err(ParseError::ContestantFormatError(e)),
    };
    
    Ok((giveaway, init_contestant, entry_count))
}