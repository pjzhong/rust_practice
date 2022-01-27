use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

pub const ENTER: &str = "Enter";
pub const BACKSPACE: &str = "Backspace";

#[derive(PartialEq, Clone)]
pub enum KeyType {
    Letter(char),
    Enter,
    Backspace,
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
            _ => "unused",
        }.to_string()
    }
}

#[function_component(Key)]
pub fn key(key_pro: &KeyProps) -> Html {
    let key_id = match key_pro.def {
        KeyType::Letter(l) => l.to_string(),
        KeyType::Enter => "Enter".to_string(),
        KeyType::Backspace => "Backspace".to_string(),
    };

    let disp = match key_pro.def {
        KeyType::Letter(l) => l.to_string(),
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
            html! { <Key def={KeyType::Letter(c)} /> }
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
                                key_press.emit(KeyType::Letter(c))
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
