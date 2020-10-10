use crate::{bot_logic::run, checkbox::*, messages::Message, settings::Settings};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_futures::*;
use yew::prelude::*;

pub enum Tab {
    Main,
    Settings,
    Stats,
}

pub struct Model {
    link: Rc<ComponentLink<Self>>,
    settings: Rc<RefCell<Settings>>,
    tab: Tab,
    progress: usize,
    progress_state: BotState,
    messages: Vec<Message<String>>,
}

pub enum Msg {
    Done,
    ProgressChange(usize),
    SettingsUpdate(&'static str, String),
    CheckboxChange(CheckboxId, bool),
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
        let settings = Rc::new(RefCell::new(Settings::load()));

        Self {
            link,
            settings,
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
                match name {
                    "twitter_username" => self.settings.borrow_mut().twitter_username = value,
                    "text_input_sentence" => self.settings.borrow_mut().text_input_sentence = value,
                    name => panic!("No field with the name {}", name),
                }
                self.settings.borrow().save();
            }
            Msg::ChangeTab(tab) => {
                self.tab = tab;
            }
            Msg::ProgressChange(p) => {
                self.progress = p;
            }
            Msg::CheckboxChange(name, value) => {
                match name {
                    CheckboxId::TwitterFollow => {
                        self.settings.borrow_mut().auto_follow_twitter = value
                    }
                    CheckboxId::TwitterRetweet => self.settings.borrow_mut().auto_retweet = value,
                    CheckboxId::TwitterTweet => self.settings.borrow_mut().auto_tweet = value,
                    CheckboxId::EmailSubscribe => {
                        self.settings.borrow_mut().auto_email_subscribe = value
                    }
                }
                self.settings.borrow().save();
            }
            Msg::Launch => {
                if self.progress_state == BotState::Waiting {
                    let link2 = Rc::clone(&self.link);
                    let settings2 = Rc::clone(&self.settings);
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
                            <div class="circuit" style=format!("width: {}%", self.progress)>
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
                let check_clbk = self.link.callback(|(e, t)| Msg::CheckboxChange(e, t));
                let settings = self.settings.borrow();
                html! {
                    <div>
                        <label class="setting">
                            {"Your Twitter username: "}
                            <input type="text" class="ng-pristine ng-untouched ng-valid ng-not-empty ng-valid-required ng-valid-pattern" placeholder="jack" oninput=self.link.callback(|e: InputData| Msg::SettingsUpdate("twitter_username", e.value)) value=settings.twitter_username/><br/>
                            <span class="explanation">{ "Your Twitter username without the '@'. You must be connected to this account on this browser." }</span>
                        </label><br/>

                        <label class="setting">
                            {"Text input sentence: "}
                            <input type="text" class="ng-pristine ng-untouched ng-valid ng-not-empty ng-valid-required ng-valid-pattern" placeholder="I don't understand" oninput=self.link.callback(|e: InputData| Msg::SettingsUpdate("text_input_sentence", e.value)) value=settings.text_input_sentence/>
                            <br/>
                            <span class="explanation">{ "Sometimes, gleam.io is asking for a text input from the user. The questions asked to the user are determined by the competition owner, so it cannot be automated without an artificial intelligence. To bypass this limitation, the bot is programmed to answer with a generic sentence that could match to any question. By default, this is something like \"I don't understand\" translated in an uncommon language. The problem is that if everyone is using the default value, it will be very easy for gleam.io to detect the bot and ban you. So I recommend using your own unique sentence." }</span>
                        </label><br/>

                        <div class="setting">
                            <Checkbox<CheckboxId> id=CheckboxId::EmailSubscribe label="Subscribe to newsletters" onchange=&check_clbk checked=settings.auto_email_subscribe/>
                            <span class="explanation">{ "Allow the bot to subscribe to newsletters with your email." }</span>
                        </div><br/>

                        <div class="setting">
                            <Checkbox<CheckboxId> id=CheckboxId::TwitterFollow label="Follow on Twitter automatically" onchange=&check_clbk checked=settings.auto_follow_twitter/>
                            <span class="explanation">{ "Allow the bot to follow people on Twitter with your account." }</span>
                        </div><br/>

                        <div class="setting">
                            <Checkbox<CheckboxId> id=CheckboxId::TwitterTweet label="Automate tweets" onchange=&check_clbk checked=settings.auto_tweet/>
                            <span class="explanation">{ "Allow the bot to tweet things with your account." }</span>
                        </div><br/>

                        <div class="setting">
                            <Checkbox<CheckboxId> id=CheckboxId::TwitterRetweet label="Automate retweets" onchange=&check_clbk checked=settings.auto_retweet/>
                            <span class="explanation">{ "Allow the bot to retweet with your account." }</span>
                        </div><br/>

                        <button class="btn btn-primary ng-binding" onclick=self.link.callback(|e: _| Msg::ChangeTab(Tab::Main))>{"Save"}</button>
                    </div>
                }
            }
            Tab::Stats => {
                html! {
                    <div>
                        {format!("Total entries: {}", self.settings.borrow().total_entries)}<br/>
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
