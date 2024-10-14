use wasm_bindgen::prelude::*;
use std::sync::{Arc,Mutex};
use once_cell::sync::Lazy;
use futures_signals::signal::{Mutable, SignalExt};
use futures::stream::{SplitStream, SplitSink};
use futures::sink::{Sink};
use dominator::{Dom, class, html, clone, events};
use gloo_net::websocket::{Message, futures::WebSocket, WebSocketError, Message::Text, Message::Bytes };
use wasm_bindgen_futures::spawn_local;
use futures::{SinkExt, StreamExt};
use web_sys::console;


struct App {
    counter: Mutable<i32>,
    write: Mutex<SplitSink<WebSocket,Message>>,
}

impl App {
    fn new(insink: SplitSink<WebSocket,Message> ) -> Arc<Self> {
        return Arc::new(Self {
            counter: Mutable::new(0),
            write: Mutex::new(insink)
        });
    }

    fn render(state:  &mut Arc<Self>) -> Dom {
        // Define CSS styles
        static ROOT_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("display", "inline-block")
            .style("background-color", "black")
            .style("padding", "10px")
        });

        static TEXT_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("color", "white")
            .style("font-weight", "bold")
        });

        static BUTTON_CLASS: Lazy<String> = Lazy::new(|| class! {
            .style("display", "block")
            .style("width", "100px")
            .style("margin", "5px")
        });

        // Create the DOM nodes
        html!("div", {
            .class(&*ROOT_CLASS)

            .children(&mut [
                html!("div", {
                    .class(&*TEXT_CLASS)
                    .text_signal(state.counter.signal().map(|x| format!("Counter: {}", x)))
                }),

                html!("button", {
                    .class(&*BUTTON_CLASS)
                    .text("Increase")
                    .event(clone!(state => move |_: events::Click| {
                        // Increment the counter
                        spawn_local(clone!( state=> async move {
                            state.write.lock().unwrap().send(Message::Text(String::from("+"))).await.unwrap();}));
                    })) }),

                html!("button", {
                    .class(&*BUTTON_CLASS)
                    .text("Decrease")
                    .event(clone!(state => move |_: events::Click| {
                        // Decrement the counter
                        spawn_local(clone!( state=> async move {
                            state.write.lock().unwrap().send(Message::Text(String::from("-"))).await.unwrap();}));
                    }))
                }),

                html!("button", {
                    .class(&*BUTTON_CLASS)
                    .text("Reset")
                    .event(clone!(state => move |_: events::Click| {
                        spawn_local(clone!( state => async move {
                            state.write.lock().unwrap().send(Message::Text(String::from("0"))).await.unwrap();}));
                    }))
                }),
            ])
        })
    }
}




#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let mut ws = WebSocket::open("wss://echo.websocket.org").unwrap();
    let (write, mut read) = ws.split();
    let mut app = App::new(write);

    spawn_local(clone!(app => async move {
        while let Some(msg) = read.next().await {
            match msg.unwrap() {
                Text(str) => match str.as_str() {
                    "+" => {app.counter.replace_with(|x| *x +1); ()},
                    "-" => {app.counter.replace_with(|x| *x -1); ()},
                    "0" => {app.counter.set_neq(0); ()},
                    &_ => ()
                },
                Bytes(_) =>()
            }
        }
    }));

    dominator::append_dom(&dominator::body(), App::render(&mut app));

    Ok(())
}
