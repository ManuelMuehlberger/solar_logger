use dioxus::prelude::*;
use dioxus_web::Config;

mod components;
mod pages;
mod utils;

fn main() {
    // Initialize logging for development
    wasm_logger::init(wasm_logger::Config::default());
    
    let config = Config::default();
    launch(app);
}

fn app() -> Element {
    rsx! {
        // Global styles - load directly in index.html instead
        div {
            class: "app-container",
            
            // We'll add these components back once they're implemented
            // components::header::Header {}
            
            div {
                class: "main-content",
                
                // components::sidebar::Sidebar {}
                
                div {
                    class: "content",
                    // pages::home::HomePage {}
                    "Hello World!" // Temporary placeholder
                }
            }
        }
    }
}