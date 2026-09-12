//! Mental arithmetic training game

use dioxus::prelude::*;
use dioxus_i18n::t;
use dioxus_sdk::time::use_interval;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

const TOTAL_QUESTIONS: usize = 30;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Exercise {
    MultiplicationTable,
    TwoDigitAddSub,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operator {
    Add,
    Sub,
    Mul,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Blank {
    Left,
    Right,
    Result,
}

/// A single question. `blank` marks which of the three positions is hidden.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Question {
    left: u8,
    right: u8,
    op: Operator,
    blank: Blank,
}

impl Question {
    /// The value the player has to type in.
    fn answer(&self) -> u32 {
        match self.blank {
            Blank::Left => self.left as u32,
            Blank::Right => self.right as u32,
            Blank::Result => match self.op {
                Operator::Add => self.left as u32 + self.right as u32,
                Operator::Sub => self.left as u32 - self.right as u32,
                Operator::Mul => self.left as u32 * self.right as u32,
            },
        }
    }

    /// Text shown on the board, with `?` for the blank position.
    fn display(&self) -> String {
        let left = if self.blank == Blank::Left {
            "?".to_string()
        } else {
            self.left.to_string()
        };
        let right = if self.blank == Blank::Right {
            "?".to_string()
        } else {
            self.right.to_string()
        };
        let result = if self.blank == Blank::Result {
            "?".to_string()
        } else {
            match self.op {
                Operator::Add => (self.left as u32 + self.right as u32).to_string(),
                Operator::Sub => (self.left as u32 - self.right as u32).to_string(),
                Operator::Mul => (self.left as u32 * self.right as u32).to_string(),
            }
        };
        let sign = match self.op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "×",
        };
        format!("{left} {sign} {right} = {result}")
    }
}

// Simple PRNG so we don't need the rand crate (works on wasm without extra setup)
struct SimpleRng(u64);

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn gen_range(&mut self, lo: u8, hi: u8) -> u8 {
        lo + (self.next() % (hi - lo + 1) as u64) as u8
    }
}

fn random_question(exercise: Exercise) -> Question {
    let mut rng = SimpleRng::new();
    match exercise {
        Exercise::MultiplicationTable => Question {
            left: rng.gen_range(1, 9),
            right: rng.gen_range(1, 9),
            op: Operator::Mul,
            blank: Blank::Result,
        },
        Exercise::TwoDigitAddSub => {
            // Both operands and the result must stay two-digit (10..=99).
            let op = if rng.gen_range(0, 1) == 0 {
                Operator::Add
            } else {
                Operator::Sub
            };
            let (left, right) = match op {
                // left + right <= 99, both >= 10
                Operator::Add => {
                    let left = rng.gen_range(10, 89);
                    let right = rng.gen_range(10, 99 - left);
                    (left, right)
                }
                // left - right >= 10, both >= 10
                _ => {
                    let left = rng.gen_range(20, 99);
                    let right = rng.gen_range(10, left - 10);
                    (left, right)
                }
            };
            let blank = match rng.gen_range(0, 2) {
                0 => Blank::Left,
                1 => Blank::Right,
                _ => Blank::Result,
            };
            Question {
                left,
                right,
                op,
                blank,
            }
        }
    }
}

#[component]
pub fn MentalMath() -> Element {
    let mut exercise = use_signal(|| None::<Exercise>);
    let mut question = use_signal(|| random_question(Exercise::MultiplicationTable));
    let mut progress = use_signal(|| 0usize);
    let mut input = use_signal(String::new);
    // None = no feedback yet, Some(true) = correct, Some(false) = wrong
    let mut feedback = use_signal(|| None::<bool>);
    let mut elapsed = use_signal(|| 0u64);
    let mut finished = use_signal(|| false);
    let mut history = use_signal(Vec::<u64>::new);

    use_interval(Duration::from_secs(1), move |()| {
        if exercise.read().is_some() && !finished() {
            elapsed += 1;
        }
    });

    // Focus the answer input after an exercise is (re)started
    use_effect(move || {
        let should_focus = exercise.read().is_some() && !*finished.read();
        if should_focus {
            spawn(async move {
                let _ = dioxus::document::eval(
                    r#"document.getElementById("mental-math-answer")?.focus();"#,
                )
                .await;
            });
        }
    });

    let mut start_exercise = move |ex: Exercise| {
        exercise.set(Some(ex));
        question.set(random_question(ex));
        progress.set(0);
        input.set(String::new());
        feedback.set(None);
        elapsed.set(0);
        finished.set(false);
    };

    let mut submit_answer = move || {
        let Some(ex) = exercise() else { return };
        if finished() {
            return;
        }
        let Ok(answer) = input().trim().parse::<u32>() else {
            return;
        };
        let expected = question().answer();

        if answer == expected {
            feedback.set(Some(true));
            let done = progress() + 1;
            progress.set(done);
            if done >= TOTAL_QUESTIONS {
                finished.set(true);
                let secs = elapsed();
                history.write().push(secs);
            } else {
                question.set(random_question(ex));
            }
        } else {
            feedback.set(Some(false));
        }
        input.set(String::new());
    };

    rsx! {
        div {
            tabindex: "0",
            style: "display: flex; justify-content: center; gap: 40px; align-items: flex-start; max-width: 1200px; margin: 0 auto; outline: none;",
            onkeydown: move |event: Event<KeyboardData>| {
                use dioxus::prelude::Key;
                if event.key() == Key::Escape {
                    if let Some(ex) = exercise() {
                        start_exercise(ex);
                    }
                }
            },

            // Left column: exercise selection
            div {
                style: "min-width: 220px; text-align: left; background: white; border-radius: 10px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                h3 {
                    style: "margin-top: 0; color: #333;",
                    {t!("practice-content")}
                }
                button {
                    style: if exercise() == Some(Exercise::MultiplicationTable) {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #FF9800; color: white; border: none; border-radius: 5px; cursor: pointer;"
                    } else {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 5px; cursor: pointer;"
                    },
                    onclick: move |_| start_exercise(Exercise::MultiplicationTable),
                    {t!("multiplication-table")}
                }
                button {
                    style: if exercise() == Some(Exercise::TwoDigitAddSub) {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #FF9800; color: white; border: none; border-radius: 5px; cursor: pointer;"
                    } else {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 5px; cursor: pointer;"
                    },
                    onclick: move |_| start_exercise(Exercise::TwoDigitAddSub),
                    {t!("two-digit-add-sub")}
                }
            }

            // Middle column: question and answer
            div {
                style: "min-width: 400px; background: white; border-radius: 10px; padding: 30px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                if exercise().is_some() {
                    {
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; color: #666; font-size: 16px; margin-bottom: 20px;",
                                span { {t!("question-counter")} " {progress} / {TOTAL_QUESTIONS}" }
                                span { {t!("elapsed-time")} " {elapsed} " {t!("seconds-suffix")} }
                            }

                            if finished() {
                                div {
                                    style: "padding: 30px; text-align: center;",
                                    h2 {
                                        style: "color: #4CAF50; margin: 0 0 10px 0;",
                                        {t!("finished-msg")}
                                    }
                                    p {
                                        style: "font-size: 20px; color: #333;",
                                        {t!("elapsed-time")} " {elapsed} " {t!("seconds-suffix")}
                                    }
                                    p {
                                        style: "color: #999; font-size: 14px;",
                                        {t!("esc-hint")}
                                    }
                                }
                            } else {
                                div {
                                    style: "font-size: 48px; font-weight: bold; color: #333; padding: 30px 0; text-align: center;",
                                    "{question().display()}"
                                }

                                input {
                                    r#type: "text",
                                    id: "mental-math-answer",
                                    autofocus: true,
                                    value: "{input}",
                                    placeholder: "",
                                    style: "width: 100%; box-sizing: border-box; padding: 15px; font-size: 28px; text-align: center; border: 2px solid #ddd; border-radius: 8px; outline: none;",
                                    oninput: move |event| {
                                        input.set(event.value());
                                        feedback.set(None);
                                    },
                                    onkeydown: move |event: Event<KeyboardData>| {
                                        use dioxus::prelude::Key;
                                        if event.key() == Key::Enter {
                                            submit_answer();
                                        }
                                    },
                                }

                                p {
                                    style: "color: #999; font-size: 14px; margin: 10px 0;",
                                    {t!("press-enter")} " · " {t!("esc-hint")}
                                }

                                if let Some(correct) = feedback() {
                                    p {
                                        style: if correct {
                                            "color: #4CAF50; font-size: 20px; font-weight: bold; margin: 10px 0;"
                                        } else {
                                            "color: #f44336; font-size: 20px; font-weight: bold; margin: 10px 0;"
                                        },
                                        if correct {
                                            {t!("correct-feedback")}
                                        } else {
                                            {t!("wrong-feedback")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p {
                        style: "color: #999; font-size: 18px; padding: 40px 0;",
                        {t!("select-exercise-hint")}
                    }
                }
            }

            // Right column: time history
            div {
                style: "min-width: 220px; text-align: left; background: white; border-radius: 10px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                h3 {
                    style: "margin-top: 0; color: #333;",
                    {t!("time-history")}
                }
                if history.read().is_empty() {
                    p {
                        style: "color: #999;",
                        {t!("no-records")}
                    }
                } else {
                    ul {
                        style: "list-style: none; padding: 0; margin: 0;",
                        for (index, secs) in history.read().iter().enumerate().rev() {
                            li {
                                key: "{index}",
                                style: "padding: 8px 0; border-bottom: 1px solid #eee; color: #333; font-size: 16px;",
                                "#{index + 1} — {secs} " {t!("seconds-suffix")}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_digit_add_sub_terms_are_all_two_digit() {
        for _ in 0..2000 {
            let q = random_question(Exercise::TwoDigitAddSub);
            assert!((10..=99).contains(&q.left), "left not two-digit: {q:?}");
            assert!((10..=99).contains(&q.right), "right not two-digit: {q:?}");

            let result = match q.op {
                Operator::Add => q.left as u32 + q.right as u32,
                Operator::Sub => q.left as u32 - q.right as u32,
                Operator::Mul => unreachable!("two-digit add/sub must not multiply"),
            };
            assert!((10..=99).contains(&result), "result not two-digit: {q:?}");

            let expected = match q.blank {
                Blank::Left => q.left as u32,
                Blank::Right => q.right as u32,
                Blank::Result => result,
            };
            assert_eq!(q.answer(), expected);
        }
    }

    #[test]
    fn blank_position_covers_all_three_positions() {
        let mut seen_left = false;
        let mut seen_right = false;
        let mut seen_result = false;
        for _ in 0..1000 {
            match random_question(Exercise::TwoDigitAddSub).blank {
                Blank::Left => seen_left = true,
                Blank::Right => seen_right = true,
                Blank::Result => seen_result = true,
            }
        }
        assert!(
            seen_left && seen_right && seen_result,
            "blank position should be random across left/right/result"
        );
    }
}
