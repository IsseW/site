use yew::prelude::*;

pub enum Msg {}

#[derive(Debug, Default)]
pub struct App {}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <main>
                <h1>{"Work in Progress 🦀🦀🦀"}</h1>
            </main>
        }
    }
}
