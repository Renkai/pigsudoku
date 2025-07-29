//! Frontend module containing UI components and styling

use crate::backend::{Difficulty, SudokuGame};
use dioxus::prelude::*;
use dioxus_i18n::t;

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
                                if is_highlighted {
                                    // Selected and highlighted: darker yellow
                                    if is_initial {
                                        cell_style.push_str(" background-color: #ffc107; color: #333; font-weight: 900;");
                                    } else {
                                        cell_style.push_str(" background-color: #ffc107; color: #1976D2; font-weight: 600;");
                                    }
                                } else if is_initial {
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
                                // Given numbers: same background as filled cells, bold black text
                                cell_style.push_str(" background-color: #f8f9fa; color: #000; font-weight: 900;");
                            } else if cell_value.is_some() {
                                // User input numbers: same background as preset cells, blue text
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
                                    }).unwrap_or_else(|| {
                                        // Show notes if cell is empty
                                         let notes = game_state.get_notes(row, col);
                                         if !notes.is_empty() {
                                             let mut notes_vec: Vec<u8> = notes.iter().cloned().collect();
                                             notes_vec.sort();
                                            let notes_display = notes_vec.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
                                            rsx! {
                                                div {
                                                    style: "font-size: 10px; color: #666; line-height: 1; \
                                                           display: flex; flex-wrap: wrap; justify-content: center; \
                                                           align-items: center; width: 100%; height: 100%; \
                                                           padding: 2px;",
                                                    "{notes_display}"
                                                }
                                            }
                                        } else {
                                            rsx! { "" }
                                        }
                                    })}
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
pub fn UndoRedoControls(game: Signal<SudokuGame>) -> Element {
    let game_state = game.read();
    let can_undo = game_state.can_undo();
    let can_redo = game_state.can_redo();

    rsx! {
            div {
                style: "display: flex; justify-content: center; gap: 15px; margin-top: 15px;",

                button {
                    style: format!(
                        "padding: 10px 20px; font-size: 16px; border: none; border-radius: 5px; cursor: {}; transition: all 0.3s; {}",
                        if can_undo { "pointer" } else { "not-allowed" },
                        if can_undo {
                            "background-color: #2196F3; color: white;"
                        } else {
                            "background-color: #ccc; color: #666;"
                        }
                    ),
                    disabled: !can_undo,
                    onclick: {
                        let mut game = game.clone();
                        move |_| {
                            game.write().undo();
                        }
                    },
    {t!("undo")}
                }

                button {
                    style: format!(
                        "padding: 10px 20px; font-size: 16px; border: none; border-radius: 5px; cursor: {}; transition: all 0.3s; {}",
                        if can_redo { "pointer" } else { "not-allowed" },
                        if can_redo {
                            "background-color: #4CAF50; color: white;"
                        } else {
                            "background-color: #ccc; color: #666;"
                        }
                    ),
                    disabled: !can_redo,
                    onclick: {
                        let mut game = game.clone();
                        move |_| {
                            game.write().redo();
                        }
                    },
    {t!("redo")}
                }
            }
        }
}

#[component]
pub fn NumberPanel(game: Signal<SudokuGame>) -> Element {
    let mut is_note_mode = use_signal(|| false);
    
    rsx! {
        div {
            style: "background-color: #f8f9fa; border: 2px solid #dee2e6; border-radius: 8px; padding: 15px; margin-top: 20px;",
            
            // Toggle button for note/fill mode
            div {
                style: "margin-bottom: 15px; text-align: center;",
                button {
                    style: format!(
                        "padding: 8px 16px; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; transition: all 0.2s; {}",
                        if is_note_mode() {
                            "background-color: #ffc107; color: #000;"
                        } else {
                            "background-color: #007bff; color: white;"
                        }
                    ),
                    onclick: move |_| {
                        is_note_mode.set(!is_note_mode());
                    },
                    if is_note_mode() {
                        {t!("note-mode")}
                    } else {
                        {t!("fill-mode")}
                    }
                }
            }
            
            // Number grid (3x3)
            div {
                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px;",
                
                for num in 1..=9 {
                    button {
                        style: "width: 50px; height: 50px; border: 2px solid #6c757d; border-radius: 6px; \
                               background-color: white; font-size: 18px; font-weight: bold; cursor: pointer; \
                               transition: all 0.2s; display: flex; align-items: center; justify-content: center;",
                        onmouseenter: move |_| {},
                        onmouseleave: move |_| {},
                        onclick: move |_| {
                            let mut game_state = game.write();
                            if let Some((row, col)) = game_state.selected_cell {
                                if is_note_mode() {
                                    game_state.toggle_note(row, col, num);
                                } else {
                                    game_state.input_number(num);
                                }
                            }
                        },
                        "{num}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn MoveLog(game: Signal<SudokuGame>) -> Element {
    let game_state = game.read();
    let move_log = game_state.get_move_log();

    rsx! {
            div {
                style: "background-color: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); max-height: 500px; overflow-y: auto;",

                h3 {
                    style: "margin-top: 0; margin-bottom: 15px; color: #333; font-size: 18px; border-bottom: 2px solid #2196F3; padding-bottom: 8px;",
    {t!("move-history")}
                }

                if move_log.is_empty() {
                    div {
                        style: "text-align: center; color: #666; font-style: italic; padding: 20px;",
    {t!("no-moves")}
                    }
                } else {
                    div {
                        style: "font-family: 'Courier New', monospace; font-size: 14px; line-height: 1.6;",

                        for (index, log_entry) in move_log.iter().enumerate() {
                            {
                                let entry_style = if log_entry.starts_with("► ") {
                                    "background-color: #e3f2fd; padding: 8px; margin: 2px 0; border-radius: 4px; border-left: 4px solid #2196F3; font-weight: bold;"
                                } else if log_entry.starts_with("✓ ") {
                                    "background-color: #f1f8e9; padding: 8px; margin: 2px 0; border-radius: 4px; border-left: 4px solid #4CAF50; color: #2e7d32;"
                                } else {
                                    "background-color: #f5f5f5; padding: 8px; margin: 2px 0; border-radius: 4px; border-left: 4px solid #ccc; color: #666;"
                                };

                                rsx! {
                                    div {
                                        key: "{index}",
                                        style: "{entry_style}",
                                        "{log_entry}"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style: "margin-top: 15px; padding-top: 15px; border-top: 1px solid #eee; font-size: 12px; color: #666;",

                        div { {t!("legend")} }
                        div { {t!("current-position")} }
                        div { {t!("completed-moves")} }
                        div { {t!("future-moves")} }
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
                    {t!("difficulty")}
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
    {t!("very-easy")}
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
    {t!("easy")}
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
    {t!("medium")}
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
    {t!("hard")}
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
    {t!("hint")}
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

            h3 { {t!("instructions-title")} }
            ul {
                li { {t!("instruction-1")} }
                li { {t!("instruction-2")} }
                li { {t!("instruction-3")} }
                li { {t!("instruction-4")} }
                li { {t!("instruction-5")} }
                li { {t!("instruction-6")} }
                li { {t!("instruction-7")} }
                li { {t!("instruction-8")} }
                li { {t!("instruction-9")} }
                li { {t!("instruction-10")} }
                li { {t!("instruction-11")} }
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
    {t!("congratulations")}
                }

                div {
                    style: "font-size: 20px; margin-bottom: 10px;",
                    match player_name.read().as_ref() {
                        Some(name) => format!("{} {}!", t!("well-done"), name),
                        None => t!("well-done-anonymous")
                    }
                }

                div {
                    style: "font-size: 18px; margin-bottom: 15px;",
    {t!("sudoku-master")}
                }

                div {
                    style: "font-size: 32px;",
                    "🏆 🌟 ✨ 🎊 🎈"
                }

                div {
                    style: "font-size: 14px; margin-top: 10px; opacity: 0.9;",
    {t!("amazing-work")}
                }
            }
        }
}
