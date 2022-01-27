use yew::{html, Html, prelude::*};
use yew::prelude::function_component;

#[derive(PartialEq, Clone)]
pub enum CellValue {
    Empty,
    Typing(char),
    Absent(char),
    Present(char),
    Correct(char),
}

#[derive(Properties, PartialEq)]
pub struct CellProps {
    pub value: CellValue,
}

#[function_component(Cell)]
pub fn cell(props: &CellProps) -> Html {
    match props.value {
        CellValue::Typing(c) => html! {
                <div data-status="empty" class="tile">{c}</div>
        },
        _ => html! {
                <div data-status="empty" class="tile"/>
        },
    }
}
