use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent, window};
use yew::{Component, Context, html, Html};

use board::board::Board;
use keyboard::keyboard::Keyboard;

use crate::board::cell::CellValue;
use crate::keyboard::keyboard::KeyType;

mod board;
mod keyboard;

fn handle_key_press(e: KeyboardEvent) -> Option<KeyType> {
    if e.ctrl_key() || e.alt_key() || e.meta_key() || e.shift_key() {
        return None;
    }

    match e.key().as_str() {
        "Enter" => Some(KeyType::Enter),
        "Backspace" => Some(KeyType::Backspace),
        c if c.len() == 1 => {
            if let Some(c) = c.chars().next() {
                if c.is_alphabetic() {
                    Some(KeyType::Letter(c))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None
    }
}

struct WorldGuess {
    guessing: String,
    guessed: Vec<Vec<CellValue>>,
    word_length: usize,
}

impl Component for WorldGuess {
    type Message = KeyType;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            guessing: String::new(),
            guessed: vec![],
            word_length: 5,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            KeyType::Letter(c)  if self.guessing.len() < self.word_length => {
                self.guessing.push(c.to_ascii_lowercase());
                true
            }
            KeyType::Backspace  if !self.guessing.is_empty() => {
                self.guessing.pop();
                true
            }
            _ => {
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let on_key_press = ctx.link().batch_callback(handle_key_press);

        let window = window().expect("No window? Where am I?");

        EventListener::new(&window, "keydown", move |e: &Event| {
            if let Ok(e) = e.clone().dyn_into::<KeyboardEvent>() {
                on_key_press.emit(e);
            }
        }).forget();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = ctx.link().callback(|msg: KeyType| msg);

        html! {
            <div class="page">
                <Board
                      guessing={self.guessing.clone()}
                      guessed={self.guessed.clone()}
                 />
                <Keyboard key_press={cb}/>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<WorldGuess>();
}
