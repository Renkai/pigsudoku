use dioxus::prelude::*;

mod backend;
mod frontend;

use backend::SudokuGame;
use frontend::{SudokuGrid, NumberInput, GameControls, Instructions, WinMessage};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let game = use_signal(|| SudokuGame::new());
    let is_complete = game.read().is_complete();
    
    rsx! {
        div {
            style: "text-align: center; padding: 20px; font-family: Arial, sans-serif; background-color: #f0f0f0; min-height: 100vh;",
            
            h1 { 
                style: "color: #333; margin-bottom: 20px;",
                "🐷 PigSudoku" 
            }
            
            if is_complete {
                WinMessage {}
            }
            
            SudokuGrid { game: game }
            NumberInput { game: game }
            GameControls { game: game }
            Instructions {}
        }
    }
}
