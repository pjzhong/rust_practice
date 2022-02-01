use yew::prelude::function_component;
use yew::{html, prelude::*, Html};

#[derive(PartialEq, Clone)]
pub enum CellValue {
    Empty,
    Typing(char),
    Absent(char),
    Present(char),
    Correct(char),
    Bad(char),
}

impl CellValue {
    pub fn score_char(&self) -> char {
        match self {
            Self::Present(_) => '🟨',
            Self::Correct(_) => '🟩',
            _ => '⬜',
        }
    }
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
        CellValue::Correct(v) => html! {
                <div data-status="correct" class="tile">{v}</div>
        },
        CellValue::Present(v) => html! {
                <div data-status="present" class="tile">{v}</div>
        },
        CellValue::Absent(v) => html! {
                <div data-status="absent" class="tile">{v}</div>
        },
        CellValue::Bad(v) => html! {
                <div data-status="wrong" class="tile">{v}</div>
        },
        _ => html! {
                <div data-status="empty" class="tile"/>
        },
    }
}
