use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::html::{ImplicitClone, IntoPropValue};
use yew::prelude::*;

use crate::CellValue;

pub const ENTER: &str = "Enter";
pub const BACKSPACE: &str = "Backspace";

#[derive(PartialEq, Clone, Debug)]
pub enum KeyStatus {
    Unused,
    Absent,
    Present,
    Correct,
}

#[derive(PartialEq, Clone)]
pub struct KeyboardStatus {
    keys: HashMap<char, KeyStatus>,
}

impl KeyboardStatus {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn update_status(&mut self, guess: &[CellValue]) {
        for cell in guess {
            match cell {
                CellValue::Absent(c) => {
                    self.keys.entry(*c).or_insert(KeyStatus::Absent);
                }
                CellValue::Present(c) => {
                    self.keys
                        .entry(*c)
                        .and_modify(|e| {
                            if *e != KeyStatus::Correct {
                                *e = KeyStatus::Present
                            }
                        })
                        .or_insert(KeyStatus::Present);
                }
                CellValue::Correct(c) => {
                    self.keys.entry(*c).or_insert(KeyStatus::Correct);
                }
                _ => {}
            }
        }
    }

    pub fn get_status(&self, letter: char) -> KeyType {
        let status = self.keys
            .get(&letter.to_ascii_lowercase())
            .cloned()
            .unwrap_or(KeyStatus::Unused);
        KeyType::Letter(KeyValue { letter, status })
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct KeyValue {
    pub letter: char,
    pub status: KeyStatus,
}

impl KeyValue {
    pub fn new(letter: char) -> Self {
        Self {
            letter,
            status: KeyStatus::Unused,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum KeyType {
    Letter(KeyValue),
    Enter,
    Backspace,
}

impl ImplicitClone for KeyType {}

impl IntoPropValue<KeyType> for KeyValue {
    fn into_prop_value(self) -> KeyType {
        KeyType::Letter(self)
    }
}

#[derive(Properties, PartialEq)]
pub struct KeyProps {
    pub def: KeyType,
}

impl KeyProps {
    fn class(&self) -> Classes {
        match self.def {
            KeyType::Enter | KeyType::Backspace => classes!("key", "special-key"),
            _ => classes!("key"),
        }
    }

    fn status_string(&self) -> String {
        match &self.def {
            KeyType::Letter(v) => match v.status {
                KeyStatus::Unused => "unused",
                KeyStatus::Absent => "absent",
                KeyStatus::Present => "present",
                KeyStatus::Correct => "correct"
            }
            _ => "unused",
        }
            .to_string()
    }
}

#[function_component(Key)]
pub fn key(key_pro: &KeyProps) -> Html {
    let key_id = match key_pro.def {
        KeyType::Letter(ref v) => v.letter.to_string(),
        KeyType::Enter => "Enter".to_string(),
        KeyType::Backspace => "Backspace".to_string(),
    };

    let disp = match key_pro.def {
        KeyType::Letter(ref v) => v.letter.to_string(),
        KeyType::Enter => "Enter".to_string(),
        KeyType::Backspace => "Del".to_string(),
    };

    html! {
        <div data-key-id={key_id} data-status={key_pro.status_string()} class={key_pro.class()}>
            {disp}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct KeyboardProperties {
    pub key_press: Callback<KeyType>,
    pub keys: KeyboardStatus,
}

pub struct Keyboard;

impl Component for Keyboard {
    type Message = ();
    type Properties = KeyboardProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let key = |c: char| {
            html! { <Key def={ctx.props().keys.get_status(c)} /> }
        };

        let row_one = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']
            .into_iter()
            .map(key)
            .collect::<Vec<_>>();
        let row_two = ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']
            .into_iter()
            .map(key)
            .collect::<Vec<_>>();
        let row_three = ['Z', 'X', 'C', 'V', 'B', 'N', 'M']
            .into_iter()
            .map(key)
            .collect::<Vec<_>>();

        let key_press = ctx.props().key_press.clone();
        let click = ctx.link().batch_callback(move |e: MouseEvent| {
            if let Some(t) = e.target() {
                if let Ok(div) = t.dyn_into::<HtmlElement>() {
                    if let Some(key) = div.get_attribute("data-key-id") {
                        if key.len() == 1 {
                            if let Some(c) = key.chars().next() {
                                key_press.emit(KeyType::Letter(KeyValue::new(c)))
                            }
                        }

                        if key == ENTER {
                            key_press.emit(KeyType::Enter)
                        }

                        if key == BACKSPACE {
                            key_press.emit(KeyType::Backspace)
                        }
                    }
                }
            }
            None
        });

        html! {
            <div>
                <div onclick={click}  class="keyboard">
                     <div class="keyboard-row">
                          {row_one}
                     </div>
                     <div class="keyboard-row">
                          {row_two}
                     </div>
                     <div class="keyboard-row">
                          <Key def={KeyType::Enter} />
                          {row_three}
                          <Key def={KeyType::Backspace} />
                     </div>
                </div>
            </div>
        }
    }
}
