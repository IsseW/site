#![feature(once_cell_try)]
mod app;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<app::App>::new().render();
}
