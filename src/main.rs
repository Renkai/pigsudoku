use dioxus::prelude::*;

mod backend;
mod frontend;

use backend::SudokuGame;
use frontend::{SudokuGrid, GameControls, Instructions, WinMessage, DifficultySelector, MoveLog, UndoRedoControls};

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
            tabindex: "0",
            onkeydown: {
                let mut game = game.clone();
                move |event: Event<KeyboardData>| {
                    use dioxus::events::Key;
                    match event.key() {
                        Key::Character(ch) if ch.len() == 1 => {
                            let ch = ch.chars().next().unwrap();
                            if ch.is_ascii_digit() && ch != '0' {
                                if let Some(num) = ch.to_digit(10) {
                                    game.write().input_number(num as u8);
                                }
                            } else if ch == '0' {
                                game.write().clear_selected_cell();
                            }
                        }
                        Key::Delete | Key::Backspace => {
                            game.write().clear_selected_cell();
                        }
                        Key::ArrowUp => {
                            // Undo functionality
                            game.write().undo();
                        }
                        Key::ArrowDown => {
                            // Redo functionality
                            game.write().redo();
                        }
                        Key::ArrowLeft => {
                            // Undo functionality (alternative)
                            game.write().undo();
                        }
                        Key::ArrowRight => {
                            // Redo functionality (alternative)
                            game.write().redo();
                        }
                        _ => {}
                    }
                }
            },
            
            h1 { 
                style: "color: #333; margin-bottom: 20px;",
                "🐷 PigSudoku" 
            }
            
            DifficultySelector { game: game }
            
            if is_complete {
                WinMessage {}
            }
            
            div {
                style: "display: flex; justify-content: center; gap: 40px; align-items: flex-start; max-width: 1200px; margin: 0 auto;",
                
                div {
                    style: "display: flex; flex-direction: column; align-items: center;",
                    SudokuGrid { game: game }
                    GameControls { game: game }
                    UndoRedoControls { game: game }
                }
                
                div {
                    style: "min-width: 300px;",
                    MoveLog { game: game }
                }
            }
            
            Instructions {}
        }
    }
}
