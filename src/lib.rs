use std::panic;

use wasm_bindgen::prelude::*;

mod app;
mod camera;
mod instance;
mod light;
mod model;
mod state;
mod texture;
mod uniform;
mod vertex;

#[wasm_bindgen]
pub async fn wasm_main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    app::main().await;
}
