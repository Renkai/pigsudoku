use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use dioxus_i18n::t;
use dioxus_i18n::unic_langid::langid;

mod game_logic;
mod frontend;
mod mental_math;

use game_logic::SudokuGame;
use frontend::{
    DifficultySelector, GameControls, Instructions, MoveLog, NumberPanel, SudokuGrid, UndoRedoControls,
    WinMessage,
};
use mental_math::MentalMath;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Menu,
    Sudoku,
    MentalMath,
}

// On web, allow bypassing the menu via URL, e.g. ?game=math or ?game=sudoku
fn initial_screen() -> Screen {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if search.contains("game=math") {
                    return Screen::MentalMath;
                }
                if search.contains("game=sudoku") {
                    return Screen::Sudoku;
                }
            }
        }
    }
    Screen::Menu
}

#[component]
fn App() -> Element {
    // Default UI language: Chinese (use the toggle button to switch to English)
    let current_locale = use_signal(|| langid!("zh-CN"));
    let screen = use_signal(initial_screen);

    rsx! {
        if current_locale() == langid!("en-US") {
            AppWithLocale {
                key: "{current_locale}",
                locale: langid!("en-US"),
                current_locale: current_locale,
                screen: screen,
            }
        } else {
            AppWithLocale {
                key: "{current_locale}",
                locale: langid!("zh-CN"),
                current_locale: current_locale,
                screen: screen,
            }
        }
    }
}

#[component]
fn AppWithLocale(
    locale: dioxus_i18n::unic_langid::LanguageIdentifier,
    mut current_locale: Signal<dioxus_i18n::unic_langid::LanguageIdentifier>,
    mut screen: Signal<Screen>,
) -> Element {
    let _i18 = use_init_i18n(move || {
        I18nConfig::new(locale)
            .with_locale(Locale::new_static(
                langid!("en-US"),
                include_str!("../locales/en-US.ftl"),
            ))
            .with_locale(Locale::new_static(
                langid!("zh-CN"),
                include_str!("../locales/zh-CN.ftl"),
            ))
    });

    rsx! {
        div {
            style: "text-align: center; padding: 20px; font-family: Arial, sans-serif; background-color: #f0f0f0; min-height: 100vh;",

            div {
                style: "display: flex; justify-content: center; align-items: center; gap: 20px; margin-bottom: 20px;",
                if screen() != Screen::Menu {
                    button {
                        style: "padding: 8px 16px; font-size: 14px; background-color: #9e9e9e; color: white; border: none; border-radius: 5px; cursor: pointer;",
                        onclick: move |_| screen.set(Screen::Menu),
                        {t!("back-to-menu")}
                    }
                }
                h1 {
                    style: "color: #333; margin: 0;",
                    {t!("game-title")}
                }
                button {
                    style: "padding: 8px 16px; font-size: 14px; background-color: #2196F3; color: white; border: none; border-radius: 5px; cursor: pointer; transition: background-color 0.3s;",
                    onclick: move |_| {
                         let new_locale = if current_locale() == langid!("en-US") {
                             langid!("zh-CN")
                         } else {
                             langid!("en-US")
                         };
                         current_locale.set(new_locale);
                     },
                     if current_locale() == langid!("en-US") { "Switch to Chinese" } else { "Switch to English" }
                }
            }

            match screen() {
                Screen::Menu => rsx! {
                    StartScreen { screen: screen }
                },
                Screen::Sudoku => rsx! {
                    SudokuScreen {}
                },
                Screen::MentalMath => rsx! {
                    MentalMath {}
                },
            }
        }
    }
}

#[component]
fn StartScreen(mut screen: Signal<Screen>) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 20px; padding: 40px;",
            h2 {
                style: "color: #333; margin: 0;",
                {t!("choose-game")}
            }
            div {
                style: "display: flex; gap: 20px;",
                button {
                    style: "padding: 20px 40px; font-size: 24px; background-color: #4CAF50; color: white; border: none; border-radius: 10px; cursor: pointer;",
                    onclick: move |_| screen.set(Screen::Sudoku),
                    {t!("sudoku-game")}
                }
                button {
                    style: "padding: 20px 40px; font-size: 24px; background-color: #FF9800; color: white; border: none; border-radius: 10px; cursor: pointer;",
                    onclick: move |_| screen.set(Screen::MentalMath),
                    {t!("mental-math-title")}
                }
            }
        }
    }
}

#[component]
fn SudokuScreen() -> Element {
    let game = use_signal(|| SudokuGame::new());
    let is_complete = game.read().is_complete();

    rsx! {
        div {
            tabindex: "0",
            onkeydown: {
                let mut game = game.clone();
                move |event: Event<KeyboardData>| {
                    use dioxus::prelude::Key;
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
                    NumberPanel { game: game }
                    MoveLog { game: game }
                }
            }

            Instructions {}
        }
    }
}
