//! Frontend module containing UI components and styling

use crate::backend::{SudokuGame, Difficulty};
use dioxus::prelude::*;

#[component]
pub fn SudokuGrid(game: Signal<SudokuGame>) -> Element {
    let popup_visible = use_signal(|| false);
    let popup_position = use_signal(|| (0, 0));
    let popup_cell = use_signal(|| None::<(usize, usize)>);
    
    let game_state = game.read();
    rsx! {
        div {
            style: "display: inline-block; border: 3px solid #333; background-color: white; margin-bottom: 20px;",

            for row in 0..9 {
                div {
                    style: "display: flex;",

                    for col in 0..9 {
                        {
                            let cell_value = game_state.grid[row][col];
                            let is_selected = game_state.selected_cell == Some((row, col));
                            let is_initial = game_state.is_initial_cell(row, col);
                            let is_highlighted = game_state.is_cell_highlighted(row, col);
                            let has_conflict = game_state.has_conflicts(row, col);

                            let mut cell_style = String::from(
                                "width: 50px; height: 50px; border: 1px solid #ccc; \
                                 display: flex; align-items: center; justify-content: center; \
                                 font-size: 18px; font-weight: bold; cursor: pointer; transition: all 0.2s; position: relative;"
                            );

                            // Add thick borders for 3x3 boxes
                            if row % 3 == 0 {
                                cell_style.push_str(" border-top: 2px solid #333;");
                            }
                            if col % 3 == 0 {
                                cell_style.push_str(" border-left: 2px solid #333;");
                            }
                            if row == 8 {
                                cell_style.push_str(" border-bottom: 2px solid #333;");
                            }
                            if col == 8 {
                                cell_style.push_str(" border-right: 2px solid #333;");
                            }

                            // Cell coloring - distinguish between initial and user input
                            if has_conflict {
                                // Conflict cells: red background/border to indicate error
                                if is_initial {
                                    cell_style.push_str(" background-color: #ffcdd2; color: #d32f2f; font-weight: 900; border: 2px solid #f44336;");
                                } else {
                                    cell_style.push_str(" background-color: #ffebee; color: #d32f2f; font-weight: 600; border: 2px solid #f44336;");
                                }
                            } else if is_selected {
                                if is_initial {
                                    cell_style.push_str(" background-color: #ffecb3; color: #333; font-weight: 900;");
                                } else {
                                    cell_style.push_str(" background-color: #e3f2fd; color: #1976D2;");
                                }
                            } else if is_highlighted {
                                // Highlighted cells: light yellow background
                                if is_initial {
                                    cell_style.push_str(" background-color: #fff9c4; color: #000; font-weight: 900;");
                                } else {
                                    cell_style.push_str(" background-color: #fff9c4; color: #1976D2; font-weight: 600;");
                                }
                            } else if is_initial {
                                // Given numbers: darker background, bold black text
                                cell_style.push_str(" background-color: #e0e0e0; color: #000; font-weight: 900;");
                            } else if cell_value.is_some() {
                                // User input numbers: light background, blue text
                                cell_style.push_str(" background-color: #f8f9fa; color: #1976D2; font-weight: 600;");
                            } else {
                                // Empty cells: white background
                                cell_style.push_str(" background-color: white; color: #666;");
                            }

                            rsx! {
                                div {
                                    style: "{cell_style}",
                                    onclick: {
                                        let mut game = game.clone();
                                        let mut popup_visible = popup_visible.clone();
                                        let mut popup_position = popup_position.clone();
                                        let mut popup_cell = popup_cell.clone();
                                        move |event: Event<MouseData>| {
                                            game.write().select_cell(row, col);
                                            
                                            // Show popup for empty cells
                                            if cell_value.is_none() {
                                                let client_x = event.client_coordinates().x;
                                                let client_y = event.client_coordinates().y;
                                                popup_position.set((client_x as i32, client_y as i32));
                                                popup_cell.set(Some((row, col)));
                                                popup_visible.set(true);
                                            } else {
                                                popup_visible.set(false);
                                            }
                                        }
                                    },

                                    {cell_value.map(|num| {
                                        if has_conflict {
                                            rsx! {
                                                span { style: "position: relative;",
                                                    "{num}"
                                                    span {
                                                        style: "position: absolute; top: -8px; right: -8px; \
                                                               background-color: #f44336; color: white; \
                                                               border-radius: 50%; width: 16px; height: 16px; \
                                                               font-size: 10px; display: flex; \
                                                               align-items: center; justify-content: center; \
                                                               font-weight: bold;",
                                                        "!"
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! { "{num}" }
                                        }
                                    }).unwrap_or_else(|| rsx! { "" })}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Number picker popup
        if *popup_visible.read() {
            div {
                style: format!(
                    "position: fixed; left: {}px; top: {}px; z-index: 1000; \
                     background: white; border: 2px solid #333; border-radius: 8px; \
                     box-shadow: 0 4px 12px rgba(0,0,0,0.3); padding: 8px; \
                     display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px;",
                    popup_position.read().0, popup_position.read().1
                ),
                
                for num in 1..=9 {
                    button {
                        style: "width: 40px; height: 40px; font-size: 16px; font-weight: bold; \
                               border: 1px solid #2196F3; background-color: white; color: #2196F3; \
                               border-radius: 4px; cursor: pointer; transition: all 0.2s; \
                               hover:background-color: #e3f2fd;",
                        onclick: {
                            let mut game = game.clone();
                            let mut popup_visible = popup_visible.clone();
                            let popup_cell = popup_cell.clone();
                            move |_| {
                                if let Some((row, col)) = *popup_cell.read() {
                                    game.write().select_cell(row, col);
                                    game.write().input_number(num);
                                }
                                popup_visible.set(false);
                            }
                        },
                        "{num}"
                    }
                }
            }
        }
        
        // Click outside to close popup
        if *popup_visible.read() {
            div {
                style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 999;",
                onclick: {
                    let mut popup_visible = popup_visible.clone();
                    move |_| {
                        popup_visible.set(false);
                    }
                },
            }
        }
    }
}



#[component]
pub fn DifficultySelector(game: Signal<SudokuGame>) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: center; align-items: center; gap: 10px; margin-bottom: 20px; \
                   background-color: white; padding: 15px; border-radius: 10px; \
                   box-shadow: 0 2px 4px rgba(0,0,0,0.1);",

            span {
                style: "font-weight: bold; color: #333; margin-right: 10px;",
                "Difficulty:"
            }

            button {
                style: "padding: 8px 16px; font-size: 14px; background-color: #8BC34A; \
                       color: white; border: none; border-radius: 5px; cursor: pointer; \
                       transition: background-color 0.3s;",
                onclick: {
                    let mut game = game.clone();
                    move |_| {
                        game.write().reset_with_difficulty(Difficulty::VeryEasy);
                    }
                },
                "Very Easy"
            }

            button {
                style: "padding: 8px 16px; font-size: 14px; background-color: #4CAF50; \
                       color: white; border: none; border-radius: 5px; cursor: pointer; \
                       transition: background-color 0.3s;",
                onclick: {
                    let mut game = game.clone();
                    move |_| {
                        game.write().reset_with_difficulty(Difficulty::Easy);
                    }
                },
                "Easy"
            }

            button {
                style: "padding: 8px 16px; font-size: 14px; background-color: #FF9800; \
                       color: white; border: none; border-radius: 5px; cursor: pointer; \
                       transition: background-color 0.3s;",
                onclick: {
                    let mut game = game.clone();
                    move |_| {
                        game.write().reset_with_difficulty(Difficulty::Medium);
                    }
                },
                "Medium"
            }

            button {
                style: "padding: 8px 16px; font-size: 14px; background-color: #f44336; \
                       color: white; border: none; border-radius: 5px; cursor: pointer; \
                       transition: background-color 0.3s;",
                onclick: {
                    let mut game = game.clone();
                    move |_| {
                        game.write().reset_with_difficulty(Difficulty::Hard);
                    }
                },
                "Hard"
            }
        }
    }
}

#[component]
pub fn GameControls(game: Signal<SudokuGame>) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: center; gap: 15px; margin-bottom: 20px;",

            button {
                style: "padding: 10px 20px; font-size: 16px; background-color: #FF9800; \
                       color: white; border: none; border-radius: 5px; cursor: pointer; \
                       transition: background-color 0.3s;",
                onclick: {
                    let mut game = game.clone();
                    move |_| {
                        game.write().solve_one_cell();
                    }
                },
                "💡 Hint"
            }
        }
    }
}

#[component]
pub fn Instructions() -> Element {
    rsx! {
        div {
            style: "margin-top: 30px; max-width: 600px; margin-left: auto; margin-right: auto; \
                   text-align: left; background-color: white; padding: 20px; border-radius: 10px; \
                   box-shadow: 0 2px 4px rgba(0,0,0,0.1);",

            h3 { "How to Play:" }
            ul {
                li { "Click on an empty cell to select it (highlighted in blue)" }
                li { "Use keyboard numbers (1-9) or click on empty cells to open number picker" }
                li { "Use arrow keys to navigate between cells" }
                li { "Press Delete, Backspace, or 0 to clear the selected cell" }
                li { "Each row, column, and 3×3 box must contain all numbers 1-9" }
                li { "Dark gray cells are given numbers and cannot be changed" }
                li { "Light blue cells show your input numbers" }
                li { "Click '💡 Hint' to get help with one cell" }
                li { "Click '🎮 New Game' to start a new random puzzle" }
            }
        }
    }
}

#[component]
pub fn WinMessage() -> Element {
    let player_name = use_resource(|| async {
        std::process::Command::new("whoami")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "Player".to_string())
    });
    
    rsx! {
        div {
            style: "background: linear-gradient(45deg, #FF6B6B, #4ECDC4, #45B7D1, #96CEB4, #FFEAA7); color: white; padding: 30px; border-radius: 15px; margin-bottom: 20px; font-size: 24px; font-weight: bold; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3); position: relative; overflow: hidden;",
            
            div {
                style: "font-size: 28px; margin-bottom: 10px;",
                "🎉 CONGRATULATIONS! 🎉"
            }
            
            div {
                style: "font-size: 20px; margin-bottom: 10px;",
                match player_name.read().as_ref() {
                    Some(name) => format!("Well done, {}!", name),
                    None => "Well done!".to_string()
                }
            }
            
            div {
                style: "font-size: 18px; margin-bottom: 15px;",
                "You solved the puzzle like a true Sudoku master!"
            }
            
            div {
                style: "font-size: 32px;",
                "🏆 🌟 ✨ 🎊 🎈"
            }
            
            div {
                style: "font-size: 14px; margin-top: 10px; opacity: 0.9;",
                "Amazing work! Ready for another challenge?"
            }
        }
    }
}
