use yew::{Component, Context, html, Html};
use yew::prelude::*;

use super::cell::Cell;
use super::cell::CellValue;

#[derive(Properties, PartialEq)]
pub struct RowProps {
    pub values: Vec<CellValue>,
}

pub struct Row;

impl Component for Row {
    type Message = ();
    type Properties = RowProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let classes = vec!["row"];
        let props = ctx.props().values.clone();
        html! {
            <div class={classes!(classes)}>
              {props.into_iter().map(|c|  html! { <Cell value={c}/> }).collect::<Html>()}
            </div>
        }
    }
}
