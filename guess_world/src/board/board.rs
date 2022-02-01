use yew::prelude::function_component;
use yew::{html, prelude::*, Html};

use crate::CellValue;

use super::row::Row;

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub guessing: String,
    pub guessed: Vec<Vec<CellValue>>,
}

#[function_component(Board)]
pub fn view(pros: &BoardProps) -> Html {
    let mut rows = vec![vec![CellValue::Empty; 5]; 6];
    let mut guessed = pros.guessed.clone();

    if guessed.len() < rows.len() {
        let mut guess_row = vec![CellValue::Empty; 5];
        for (idx, c) in pros.guessing.chars().enumerate() {
            guess_row[idx] = CellValue::Typing(c);
        }
        guessed.push(guess_row);
    }

    for (i, val) in guessed.into_iter().enumerate() {
        rows[i] = val;
    }

    html! {
        <div class="wrapper">
            <div class="game">
                {
                    rows.into_iter()
                        .enumerate()
                        .map(|(_, r)| {
                              html! { <Row values={r}/> }
                        }).collect::<Html>()
                }
           </div>
        </div>
    }
}
