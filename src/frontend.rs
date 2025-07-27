//! Frontend module containing UI components and styling

use crate::backend::{SudokuGame, Difficulty};
use dioxus::prelude::*;

#[component]
pub fn SudokuGrid(game: Signal<SudokuGame>) -> Element {
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

                            let mut cell_style = String::from(
                                "width: 50px; height: 50px; border: 1px solid #ccc; \
                                 display: flex; align-items: center; justify-content: center; \
                                 font-size: 18px; font-weight: bold; cursor: pointer;"
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
                            if is_selected {
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
                                        move |_| {
                                            game.write().select_cell(row, col);
                                        }
                                    },

                                    {cell_value.map(|num| rsx! { "{num}" }).unwrap_or_else(|| rsx! { "" })}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NumberInput(game: Signal<SudokuGame>) -> Element {
    rsx! {
        div {
            style: "margin-bottom: 20px;",

            h3 { "Select a number:" }

            div {
                style: "display: flex; justify-content: center; gap: 10px; flex-wrap: wrap;",

                for num in 1..=9 {
                    button {
                        style: "width: 40px; height: 40px; font-size: 18px; font-weight: bold; \
                               border: 2px solid #2196F3; background-color: white; color: #2196F3; \
                               border-radius: 5px; cursor: pointer;",
                        onclick: {
                            let mut game = game.clone();
                            move |_| {
                                game.write().input_number(num);
                            }
                        },
                        "{num}"
                    }
                }

                button {
                    style: "width: 80px; height: 40px; font-size: 14px; font-weight: bold; \
                           border: 2px solid #f44336; background-color: white; color: #f44336; \
                           border-radius: 5px; cursor: pointer; margin-left: 10px;",
                    onclick: {
                        let mut game = game.clone();
                        move |_| {
                            game.write().clear_selected_cell();
                        }
                    },
                    "Clear"
                }
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
                li { "Use keyboard numbers (1-9) or click number buttons to fill the selected cell" }
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
    rsx! {
        div {
            style: "background-color: #4CAF50; color: white; padding: 10px; border-radius: 5px; margin-bottom: 20px; font-size: 18px;",
            "🎉 Congratulations! You solved the puzzle! 🎉"
        }
    }
}
