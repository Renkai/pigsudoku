use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use dioxus_i18n::t;
use dioxus_i18n::unic_langid::langid;

mod game_logic;
mod frontend;

use game_logic::SudokuGame;
use frontend::{
    DifficultySelector, GameControls, Instructions, MoveLog, NumberPanel, SudokuGrid, UndoRedoControls,
    WinMessage,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let current_locale = use_signal(|| langid!("en-US"));

    rsx! {
        if current_locale() == langid!("en-US") {
            AppWithLocale {
                key: "en-US",
                locale: langid!("en-US"),
                current_locale: current_locale
            }
        } else {
            AppWithLocale {
                key: "zh-CN",
                locale: langid!("zh-CN"),
                current_locale: current_locale
            }
        }
    }
}

#[component]
fn AppWithLocale(
    locale: dioxus_i18n::unic_langid::LanguageIdentifier,
    mut current_locale: Signal<dioxus_i18n::unic_langid::LanguageIdentifier>,
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

            div {
                style: "display: flex; justify-content: center; align-items: center; gap: 20px; margin-bottom: 20px;",
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
