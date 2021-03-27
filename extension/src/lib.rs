#![recursion_limit = "1024"]
use wasm_bindgen::prelude::*;
use web_sys::*;
#[macro_use]
mod util;
mod bot_logic;
mod checkbox;
mod format;
mod messages;
mod request;
mod settings;
mod yew_app;
use yew::prelude::App;
use yew_app::*;

#[wasm_bindgen(start)]
pub async fn main() {
    log!("Bot working!");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().expect("No window");
    let document = window.document().expect("No document");

    let panel = document.create_element("div").unwrap();
    panel.set_attribute("id", "bot_panel").unwrap();

    let style = document.create_element("style").unwrap();
    style.set_attribute("scoped", "").unwrap();
    style.set_inner_html(include_str!("style.css"));

    let mut tries = 0;
    let panel_container = loop {
        if tries > 100 {
            panic!("Cannot find container");
        }
        match document
            .get_elements_by_class_name("incentive-description")
            .item(0) {
            Some(c) => break c,
            None => crate::util::sleep(std::time::Duration::from_millis(100)).await,
        }
        tries += 1;
    };
    panel_container.set_class_name("incentive-description center middle");
    panel_container.append_child(&style).unwrap();
    panel_container.append_child(&panel).unwrap();

    yew::initialize();
    App::<Model>::new().mount(panel);
}
