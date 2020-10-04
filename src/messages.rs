use yew::prelude::*;

#[allow(dead_code)]
pub enum Message<T: std::fmt::Display> {
    Warning(T),
    Tip(T),
    Stat(T),
    Info(T),
    Other(T),
    Danger(T),
    Error(T),
}

impl<T: std::fmt::Display> Message<T> {
    pub fn as_html(&self) -> Html {
        match self {
            Message::Warning(message) => html! {
                <div><div class="warn_message"><b>{"Warning: "}</b>{message}</div><br/></div>
            },
            Message::Tip(message) => html! {
                <div><div class="tip_message"><b>{"Tip: "}</b>{message}</div><br/></div>
            },
            Message::Stat(message) => html! {
                <div><div class="stat_message">{message}</div><br/></div>
            },
            Message::Info(message) => html! {
                <div><div class="info_message"><b>{"Information: "}</b>{message}</div><br/></div>
            },
            Message::Other(message) => html! {
                <div><div class="unknown_message">{message}</div><br/></div>
            },
            Message::Danger(message) => html! {
                <div><div class="danger_message"><b>{"DANGER: "}</b>{message}</div><br/></div>
            },
            Message::Error(message) => html! {
                <div>
                    <div class="danger_message">
                        <b>{"ERROR: "}</b>
                        {message}<br/>
                        <b>{"HELP: "}</b>
                        {"The bot encountered an error. This error can be fixed, but I need information. Could you write a mail at "}<a href="mailto:mubelotix@gmail.com">{"mubelotix@gmail.com"}</a>{" or open an issue on "}<a href="https://github.com/Mubelotix/gleam.io-bot-extension/">{"the Github repository"}</a>{"? Please include the error message and the URL of the giveaway. Thank you very much. The dev"}
                    </div><br/>
                </div>
            },
        }
    }
}
