use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "text-align: center; padding: 50px; font-family: Arial, sans-serif;",
            h1 { "Hello, Dioxus World!" }
            p { "Welcome to PigSudoku - A Sudoku game built with Dioxus 0.6" }
            p { "This is a minimal Dioxus application that works on both desktop and web." }
        }
    }
}
