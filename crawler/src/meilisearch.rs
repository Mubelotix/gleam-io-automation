use format::giveaway::SearchResult;
use std::collections::HashMap;
use crate::{config::Config, database::read_database};
use meilisearch_sdk::{client::Client, errors::Error as MeilisearchError};

pub(crate) async fn init_meilisearch(config: &Config) {
    if let Some(meilisearch_config) = &config.meilisearch {
        let mut giveaways = HashMap::new();
        read_database(&mut giveaways, &config);

        let client = Client::new(&meilisearch_config.host, &meilisearch_config.key);
        let _ = client.assume_index(&meilisearch_config.index).delete().await;

        let index = client.create_index(&meilisearch_config.index, Some("key")).await.expect("Failed to create meilisearch index");
        index.set_searchable_attributes(&["name", "site_url", "site_name", "incentive_name", "incentive_description"]).await.expect("Failed to set searchable attributes");
        index.set_stop_words(&["the", "to", "of", "a", "in", "it", "on", "at", "an"]).await.expect("Failed to set stop words");
        index.set_attributes_for_faceting(&["incentive_type", "campaign_type", "language"]).await.expect("Failed to set attributes for faceting");
        index.set_displayed_attributes(&["starts_at", "ends_at", "key", "name", "language", "site_url", "site_name", "finished", "paused", "login_types", "stand_alone_url", "campaign_type", "terms_and_conditions", "announce_winners", "entry_methods", "incentive_name", "incentive_description", "incentive_type", "last_updated", "referers", "entry_count", "entry_evolution"]).await.expect("Failed to set attributes for faceting");

        index.add_or_replace(&giveaways.drain().map(|(_k, g)| g).collect::<Vec<SearchResult>>(), Some("key")).await.expect("Failed to add documents");
    } else {
        panic!("Unable to init MeiliSearch index: incomplete configuration file.")
    }
}

pub(crate) async fn update_meilisearch(mut giveaways: HashMap<String, SearchResult>, config: &Config, outdated_meilisearch: Vec<String>) -> Result<bool, MeilisearchError> {
    if let Some(config) = &config.meilisearch {
        let client = Client::new(&config.host, &config.key);
        let index = client.get_index(&config.index).await?;
        let mut to_replace = Vec::new();
        let mut to_remove = Vec::new();

        for key in outdated_meilisearch {
            match giveaways.remove(&key) {
                Some(giveaway) => {
                    to_replace.push(giveaway);
                },
                None => {
                    to_remove.push(key);
                }
            }
        }

        index.add_or_replace(&to_replace, None).await?;
        index.delete_documents(&to_remove).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}