use yew::prelude::*;

use crate::{CellValue, GameState};

#[derive(Properties, PartialEq)]
pub struct ScoreboardProps {
    pub word: String,
    pub guessed: Vec<Vec<CellValue>>,
    pub max_guesses: usize,
    pub game_state: GameState,
}

#[function_component(Scoreboard)]
pub fn scoreboard(props: &ScoreboardProps) -> Html {
    match props.game_state {
        GameState::Won => {
            return html! {
                <div class="scoreboard winner">
                     {"Winner:"}
                     <br/><br/>
                     {props.guessed.len()}{"/"}{props.max_guesses}
                     <br/><br/>
                     {
                          props.guessed.iter().map(|g|
                                html! {
                                 <>
                                    { g.iter().map(CellValue::score_char).collect::<String>()} <br/>
                                 </>
                               }
                         ).collect::<Html>()
                     }
                </div>
            };
        }
        GameState::Lost => {
            return html! {<div class="scoreboard">{"LOSER: The word was "}{props.word.clone()}</div>};
        }
        _ => return html! { <></> },
    };
}
