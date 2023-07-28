use strum::Display;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Display)]
enum Route {
    #[at("/")]
    Home,
    #[at("/projects")]
    Projects,
    #[at("/about")]
    About,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    render_page(&route, || match route {
        Route::Home => html! { <Home /> },
        Route::Projects => html! { <Projects /> },
        Route::About => html! { <About /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    })
}

#[function_component]
pub fn App() -> Html {
    html! {
        // Using `HashRouter` while hosted on github pages, once self hosted this can be `BrowsingRouter`.
        <HashRouter>
            <Switch<Route> render={switch} />
        </HashRouter>
    }
}

fn render_page(route: &Route, render: impl Fn() -> Html) -> Html {
    let selectable = [Route::Home, Route::Projects, Route::About];
    let selectable_count = selectable.len();
    let page_buttons = selectable
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let first_last = if i == 0 {
                "rounded-bl-md"
            } else if i == selectable_count - 1 {
                "rounded-br-md"
            } else {
                ""
            };

            if r == *route {
                let class = format!("px-5 py-2 bg-sky-600 {first_last}");
                html! {
                    <span class={class}> {r} </span>
                }
            } else {
                let class = format!("px-5 py-2 bg-zinc-800 hover:bg-zinc-600 {first_last}");
                let name = r.to_string();
                html! {
                    <Link<Route> to={r}> <span class={class}> {name} </span> </Link<Route>>
                }
            }
        })
        .collect::<Html>();

    html! {
        <main>
            <div class="bg-black text-white h-screen">
            <div class="flex justify-between w-full">
                <h1 class="text-3xl pl-3 pt-1">{"Isse"}</h1>
                <div class="font-semibold select-none">
                    {page_buttons}
                </div>
                <a href="https://github.com/IsseW/site" target="_blank"
                    class="select-none px-5 py-2 font-semibold rounded-bl-md bg-purple-600 hover:bg-purple-500">
                    {"<> source"}
                </a>
            </div>
            {render()}
            </div>
        </main>
    }
}

#[function_component]
fn Home() -> Html {
    html! { "Hello World!" }
}

#[function_component]
fn Projects() -> Html {
    html! { "My projects" }
}

#[function_component]
fn About() -> Html {
    html! { "I am Isse" }
}
