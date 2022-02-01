use std::collections::HashMap;

use gloo_events::EventListener;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent, window};
use yew::{Component, Context, html, Html};

use board::board::Board;
use keyboard::keyboard::Keyboard;
use scoreboard::Scoreboard;

use crate::board::cell::CellValue;
use crate::keyboard::keyboard::{KeyboardStatus, KeyType, KeyValue};

mod board;
mod keyboard;
mod scoreboard;

const WORLD_LIST: &str = include_str!("awords.txt");

#[derive(PartialEq, Clone)]
pub enum GameState {
    InProgress,
    Won,
    Lost,
}

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
                    Some(KeyType::Letter(KeyValue::new(c)))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn evaluate_guess(word: &str, guessing: &str) -> Vec<CellValue> {
    if WORLD_LIST.contains(guessing) {
        let mut vals = Vec::with_capacity(word.len());
        let mut counts = word.chars().fold(HashMap::new(), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1usize;
            acc
        });

        // find correct characters
        for (w, g) in word.chars().zip(guessing.chars()) {
            let cell = if w == g {
                if let Some(count) = counts.get_mut(&g) {
                    *count = count.saturating_sub(1);
                }
                Some(CellValue::Correct(g))
            } else {
                None
            };
            vals.push(cell)
        }

        // categorize the rest of the characters
        for (idx, g) in guessing.chars().enumerate() {
            let cell = match (vals[idx].take(), counts.get(&g)) {
                (v @ Some(_), _) => v,
                (None, Some(f)) if *f > 0 => {
                    if let Some(count) = counts.get_mut(&g) {
                        *count = count.saturating_sub(1);
                    }
                    Some(CellValue::Present(g))
                }
                (_, _) => Some(CellValue::Absent(g)),
            };
            vals[idx] = cell;
        }

        vals.into_iter()
            .map(|v| v.unwrap_or(CellValue::Empty))
            .collect()
    } else {
        return word.chars().map(CellValue::Bad).collect::<Vec<_>>();
    }
}

struct WorldGuess {
    world: String,
    guessing: String,
    guessed: Vec<Vec<CellValue>>,
    key_board_state: KeyboardStatus,
    game_state: GameState,
    word_length: usize,
    max_guess: usize,
}

impl Component for WorldGuess {
    type Message = KeyType;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let word = WORLD_LIST
            .lines()
            .choose(&mut thread_rng())
            .unwrap()
            .to_string();
        Self {
            world: word,
            guessing: String::new(),
            guessed: vec![],
            game_state: GameState::InProgress,
            key_board_state: KeyboardStatus::new(),
            word_length: 5,
            max_guess: 6,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        if self.game_state != GameState::InProgress {
            return false;
        }

        match msg {
            KeyType::Letter(v) if self.guessing.len() < self.word_length => {
                self.guessing.push(v.letter.to_ascii_lowercase());
                true
            }
            KeyType::Backspace if !self.guessing.is_empty() => {
                self.guessing.pop();
                true
            }
            KeyType::Enter if self.guessing.len() == self.word_length => {
                let new_guess = evaluate_guess(&self.world, &self.guessing);
                let correct = new_guess.iter().all(|g| matches!(g, CellValue::Correct(_)));
                self.key_board_state.update_status(&new_guess);
                self.guessing = String::new();
                self.guessed.push(new_guess);
                if correct {
                    self.game_state = GameState::Won;
                } else if self.guessed.len() == self.max_guess {
                    self.game_state = GameState::Lost;
                }
                true
            }
            _ => false,
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
        })
            .forget();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = ctx.link().callback(|msg: KeyType| msg);

        if self.game_state == GameState::InProgress {
            return html! {
                <div class="page">
                    <Board
                          guessing={self.guessing.clone()}
                          guessed={self.guessed.clone()}
                     />
                    <Keyboard key_press={cb} keys={self.key_board_state.clone()}/>
                </div>
            };
        } else {
            return html! {
                <div class="page">
                    <Board
                          guessing={self.guessing.clone()}
                          guessed={self.guessed.clone()}
                     />
                    <Keyboard key_press={cb} keys={self.key_board_state.clone()}/>
                    <Scoreboard
                         word={self.world.clone()}
                         guessed={self.guessed.clone()}
                         max_guesses={self.max_guess}
                         game_state={self.game_state.clone()}
                    />
                </div>
            };
        }
    }
}

fn main() {
    yew::start_app::<WorldGuess>();
}
