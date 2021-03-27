#![recursion_limit="1024"]

use format::prelude::*;
use meilisearch_sdk::{client::Client, indexes::Index, search::SearchResults};
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use yew::services::FetchService;
use yew::format::Nothing;
use yew::services::fetch::{Request, Response, FetchTask};
use serde_json::{from_str};
use anyhow::Error;
use std::rc::Rc;
use std::cell::RefCell;
use urlencoding::encode;
use wasm_bindgen_futures::spawn_local;

mod miscellaneous;
use crate::miscellaneous::*;

const CLIENT: Client = Client::new("https://ipv4.mubelotix.dev:7700", "321fde49d647cb8dd3ce20f75fb3b3afcd10be5995d560d4d8151abaa57e1580");

struct Model {
    link: Rc<ComponentLink<Self>>,
    index: Rc<Index<'static>>,
    results: Vec<SearchResult>,
    processing_time_ms: usize,
    query: String,

    latest_sent_request_id: usize,
    displayed_request_id: usize,
}

enum Msg {
    Input(String),
    Update{results: Vec<SearchResult>, processing_time_ms: usize, request_id: usize},
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: Rc::new(link),
            index: Rc::new(CLIENT.assume_index("giveaways")),
            results: Vec::new(),
            processing_time_ms: 0,
            query: String::new(),

            latest_sent_request_id: 0,
            displayed_request_id: 0,
        }
    }

    #[allow(unused_must_use)]
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Input(query) => {
                self.query = query.clone();
                let index = Rc::clone(&self.index);
                let link = Rc::clone(&self.link);
                self.latest_sent_request_id += 1;
                let request_id = self.latest_sent_request_id;

                // Spawn a task loading results
                spawn_local(async move {
                    // Load the results
                    let fresh_results: SearchResults<SearchResult> = index
                        .search()
                        .with_query(&query)
                        .with_attributes_to_highlight(meilisearch_sdk::search::Selectors::All)
                        .execute()
                        .await
                        .expect("Failed to execute query");

                    let mut fresh_formatted_results = Vec::new();
                    for result in fresh_results.hits {
                        fresh_formatted_results.push(result.formatted_result.unwrap());
                    }

                    // We send a new event with the up-to-date data so that we can update the results and display them.
                    link.send_message(Msg::Update{
                        results: fresh_formatted_results,
                        processing_time_ms: fresh_results.processing_time_ms,
                        request_id
                    });
                });
                false
            }
            Msg::Update{results, processing_time_ms, request_id} => {
                if request_id >= self.latest_sent_request_id {
                    self.results = results;
                    self.processing_time_ms = processing_time_ms;
                    self.displayed_request_id = request_id;
                    true
                } else {
                    // We are already displaying more up-to-date results.
                    // This request is too late so we cannot display these results to avoid rollbacks.
                    false
                }
            },
        }
    }

    fn view(&self) -> Html {
        let timestamp = get_timestamp();

        if self.query.is_empty() {
            html! {
                <main>
                    <label id="centered_form">
                        <h1>{"Googleam"}</h1>
                        <input autocomplete="off" type="text" placeholder=get_random_placeholder() oninput=self.link.callback(|data: InputData| Msg::Input(data.value))/>
                    </label>
                </main>
            }
        } else {
            html! {
                <main>
                    <label autocomplete="off" id="top_form">
                        <h1>{"Googleam"}</h1>
                        <input autocomplete="off" type="text" oninput=self.link.callback(|data: InputData| Msg::Input(data.value))/>
                    </label>
                    <div id="results">{
                        if self.latest_sent_request_id > self.displayed_request_id || !self.results.is_empty() {
                            html! {
                                {for self.results.iter().map(|result| html! {
                                    <a href=result.get_url()>
                                        <h2>{unescaped_html(&result.giveaway.incentive.name)}</h2>
                                        <p>{
                                            if !result.giveaway.incentive.description.is_empty() {
                                                unescaped_html(&result.giveaway.incentive.description)
                                            } else {
                                                unescaped_html(&result.giveaway.incentive.name)
                                            }
                                        }</p>
                                        <div>
                                            {
                                                if let Some(entry_count) = result.entry_count {
                                                    html! {<span class="entries">{format!("{} entries", entry_count)}</span>}
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            <span class="remaining_time">{"ending in "}{seconds_to_string(result.giveaway.campaign.ends_at as i64 - timestamp as i64, true)}</span>

                                        </div>
                                    </a>
                                })}
                            }
                        } else {
                            html! {
                                {
                                    format!("No result. What about \"{}\"?", get_random_placeholder())
                                }
                            }
                        }
                    }</div>
                </main>
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        true
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
}