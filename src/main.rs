use dioxus::prelude::*;

mod backend;
mod frontend;

use backend::SudokuGame;
use frontend::{SudokuGrid, NumberInput, GameControls, Instructions, WinMessage, DifficultySelector};

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
                            let mut game_state = game.write();
                            if let Some((row, col)) = game_state.selected_cell {
                                if row > 0 {
                                    game_state.select_cell(row - 1, col);
                                }
                            }
                        }
                        Key::ArrowDown => {
                            let mut game_state = game.write();
                            if let Some((row, col)) = game_state.selected_cell {
                                if row < 8 {
                                    game_state.select_cell(row + 1, col);
                                }
                            }
                        }
                        Key::ArrowLeft => {
                            let mut game_state = game.write();
                            if let Some((row, col)) = game_state.selected_cell {
                                if col > 0 {
                                    game_state.select_cell(row, col - 1);
                                }
                            }
                        }
                        Key::ArrowRight => {
                            let mut game_state = game.write();
                            if let Some((row, col)) = game_state.selected_cell {
                                if col < 8 {
                                    game_state.select_cell(row, col + 1);
                                }
                            }
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
            
            SudokuGrid { game: game }
            NumberInput { game: game }
            GameControls { game: game }
            Instructions {}
        }
    }
}
