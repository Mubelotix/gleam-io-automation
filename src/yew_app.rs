use crate::{checkbox::*, format::*};
use crate::{bot_logic::{run, Settings}, messages::Message};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::*;
use web_sys::*;
use yew::prelude::*;

pub enum Tab {
    Main,
    Settings,
    Stats,
}

pub struct Model {
    link: Rc<ComponentLink<Self>>,
    settings: Arc<Mutex<Settings>>,
    storage: Storage,
    tab: Tab,
    progress: usize,
    progress_state: BotState,
    messages: Vec<Message<String>>,
}

pub enum Msg {
    Done,
    ProgressChange(usize),
    SettingsUpdate(&'static str, String),
    ChangeTab(Tab),
    LogMessage(Message<String>),
    Launch,
}

#[derive(PartialEq)]
pub enum BotState {
    Waiting,
    Running,
    Ended,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link = Rc::new(link);
        let window = window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        let settings = Arc::new(Mutex::new(Settings::load()));

        Self {
            link,
            settings,
            storage,
            tab: Tab::Main,
            progress: 0,
            progress_state: BotState::Waiting,
            messages: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Done => self.progress_state = BotState::Ended,
            Msg::SettingsUpdate(name, value) => {
                let mut settings = match self.settings.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };

                match name {
                    "twitter_username" => settings.twitter_username = value,
                    name => panic!("No field with the name {}", name),
                }
                settings.save();
            }
            Msg::ChangeTab(tab) => {
                self.tab = tab;
            }
            Msg::ProgressChange(p) => {
                self.progress = p;
            }
            Msg::Launch => {
                if self.progress_state == BotState::Waiting {
                    let link2 = Rc::clone(&self.link);
                    let settings2 = Arc::clone(&self.settings);
                    self.progress = 0;
                    self.progress_state = BotState::Running;
                    spawn_local(async move {
                        let link3 = Rc::clone(&link2);
                        match run(link2, settings2).await {
                            Ok(()) => (),
                            Err(msg) => {
                                link3.send_message(Msg::Done);
                                link3.send_message(Msg::LogMessage(msg));
                            }
                        }
                    })
                }
            }
            Msg::LogMessage(msg) => {
                self.messages.push(msg);
            }
        }
        true
    }

    fn view(&self) -> Html {
        let settings = match self.settings.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        match self.tab {
            Tab::Main => {
                html! {
                    <div>
                        <h2>{"Top Secret Control Panel"}</h2>
                        <p>
                            {"Thank you for using the bot!"}
                        </p>
                        <br/>
                        <div class=if self.progress_state != BotState::Waiting {"progress_bar in_progress"}else{"progress_bar"} >
                            <div style=format!("width: {}%", self.progress)>
                            </div>
                        </div>
                        <br/>
                        {
                            match self.progress_state {
                                BotState::Waiting => html! { <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::Launch)>{"Launch"}</button> },
                                BotState::Running => html! {
                                    { "The bot is running. HTTP requests completing entries are made in the background." }
                                },
                                BotState::Ended => html! {
                                    { "The bot has finished. Reload this page to see the result." }
                                },
                            }
                        }<br/><br/>
                        <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::ChangeTab(Tab::Settings))>{"Settings"}</button><br/><br/>
                        <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::ChangeTab(Tab::Stats))>{"Stats"}</button><br/><br/>
                        { for self.messages.iter().map(Message::as_html) }
                    </div>
                }
            }
            Tab::Settings => {
                html! {
                    <div>
                        <label>
                            {"Your Twitter username: "}
                            <input type="text" class="ng-pristine ng-untouched ng-valid ng-not-empty ng-valid-required ng-valid-pattern" placeholder="jack" oninput=self.link.callback(|e: InputData| Msg::SettingsUpdate("twitter_username", e.value))/>
                        </label><br/>

                        {"INFO: These options are a preview of the next update. For now it is not working at all."}<br/>
                        <br/>
                        <Checkbox<CheckboxId> id=CheckboxId::Twitter label="Follow on Twitch"/>
                        <Checkbox<CheckboxId> id=CheckboxId::Twitter label="Tweet"/>
                        <Checkbox<CheckboxId> id=CheckboxId::Twitter label="Retweet"/>
                        <Checkbox<CheckboxId> id=CheckboxId::Twitter label="Follow on twitter"/>

                        <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::ChangeTab(Tab::Main))>{"Save"}</button>
                    </div>
                }
            }
            Tab::Stats => {
                html! {
                    <div>
                        {format!("Total entries: {}", settings.total_entries)}<br/>
                        {"More stats will be available in the future."}<br/>
                        <br/>
                        <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::ChangeTab(Tab::Main))>{"Go back"}</button>
                    </div>
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}
