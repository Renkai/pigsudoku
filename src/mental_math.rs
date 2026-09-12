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

/// Styles for the small per-question cooking animation in the progress grid.
const PIG_PROGRESS_CSS: &str = r#"
.pig-cell {
    position: relative;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
    line-height: 1;
}
.pig-cell.current {
    background: #fff4d6;
    border-radius: 8px;
    box-shadow: 0 0 0 2px #ffc107;
}
.pig-cooking {
    display: inline-block;
    position: relative;
    z-index: 1;
    animation: pig-wobble .65s ease-in-out infinite;
}
.pig-done {
    display: inline-block;
    animation: pig-pop .4s ease;
}
.mini-fire {
    position: absolute;
    bottom: -4px;
    left: 50%;
    font-size: 13px;
    letter-spacing: -4px;
    z-index: 0;
    transform: translateX(-50%);
    transform-origin: bottom center;
    animation: pig-flicker .3s ease-in-out infinite alternate;
}
@keyframes pig-wobble {
    0%, 100% { transform: rotate(-6deg); }
    50% { transform: rotate(6deg); }
}
@keyframes pig-pop {
    0% { transform: scale(.3) rotate(-18deg); }
    65% { transform: scale(1.3) rotate(8deg); }
    100% { transform: scale(1); }
}
@keyframes pig-flicker {
    0% { transform: translateX(-50%) scaleY(.85) scaleX(1.05); opacity: .85; }
    100% { transform: translateX(-50%) scaleY(1.15) scaleX(.95); opacity: 1; }
}
"#;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Exercise {
    MultiplicationTable,
    TwoDigitAddSub,
    TwoDigitChainAddSub,
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

impl Operator {
    fn sign(self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "×",
        }
    }
}

/// A single question. `blank` marks which position is hidden.
/// When `op2`/`third` are set the question is a left-to-right chain.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Question {
    left: u8,
    right: u8,
    /// Third operand, used by chained add/sub questions.
    third: Option<u8>,
    op: Operator,
    /// Second operator, used by chained add/sub questions.
    op2: Option<Operator>,
    blank: Blank,
}

impl Question {
    /// The value the player has to type in.
    fn answer(&self) -> u32 {
        match self.blank {
            Blank::Left => self.left as u32,
            Blank::Right => self.right as u32,
            Blank::Result => self.evaluate(),
        }
    }

    /// Left-to-right evaluation of the (possibly chained) expression.
    fn evaluate(&self) -> u32 {
        let mut value = Self::apply(self.left as i32, self.op, self.right as i32);
        if let (Some(op2), Some(third)) = (self.op2, self.third) {
            value = Self::apply(value, op2, third as i32);
        }
        value as u32
    }

    fn apply(value: i32, op: Operator, operand: i32) -> i32 {
        match op {
            Operator::Add => value + operand,
            Operator::Sub => value - operand,
            Operator::Mul => value * operand,
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
            self.evaluate().to_string()
        };
        match (self.op2, self.third) {
            (Some(op2), Some(third)) => {
                format!(
                    "{left} {} {right} {} {third} = {result}",
                    self.op.sign(),
                    op2.sign()
                )
            }
            _ => format!("{left} {} {right} = {result}", self.op.sign()),
        }
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

fn random_add_or_sub(rng: &mut SimpleRng) -> Operator {
    if rng.gen_range(0, 1) == 0 {
        Operator::Add
    } else {
        Operator::Sub
    }
}

fn random_question(exercise: Exercise) -> Question {
    let mut rng = SimpleRng::new();
    match exercise {
        Exercise::MultiplicationTable => Question {
            left: rng.gen_range(1, 9),
            right: rng.gen_range(1, 9),
            third: None,
            op: Operator::Mul,
            op2: None,
            blank: Blank::Result,
        },
        Exercise::TwoDigitAddSub => {
            // Both operands and the result must stay two-digit (10..=99).
            let op = random_add_or_sub(&mut rng);
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
                third: None,
                op,
                op2: None,
                blank,
            }
        }
        Exercise::TwoDigitChainAddSub => {
            // left op1 right op2 third = ?, all operands and the result are
            // two-digit, and intermediate values stay within 0..=100 (never
            // negative, never above 100).
            for _ in 0..1000 {
                let left = rng.gen_range(10, 99);
                let right = rng.gen_range(10, 99);
                let third = rng.gen_range(10, 99);
                let op = random_add_or_sub(&mut rng);
                let op2 = random_add_or_sub(&mut rng);

                let first = match op {
                    Operator::Add => left as i32 + right as i32,
                    _ => left as i32 - right as i32,
                };
                let result = match op2 {
                    Operator::Add => first + third as i32,
                    _ => first - third as i32,
                };

                if (0..=100).contains(&first) && (10..=99).contains(&result) {
                    return Question {
                        left,
                        right,
                        third: Some(third),
                        op,
                        op2: Some(op2),
                        blank: Blank::Result,
                    };
                }
            }
            // Practically unreachable: guaranteed-valid fallback.
            Question {
                left: 10,
                right: 10,
                third: Some(10),
                op: Operator::Add,
                op2: Some(Operator::Add),
                blank: Blank::Result,
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

            // Cooking styles for the pig -> pork chop progress display.
            style { dangerous_inner_html: PIG_PROGRESS_CSS }

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
                button {
                    style: if exercise() == Some(Exercise::TwoDigitChainAddSub) {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #FF9800; color: white; border: none; border-radius: 5px; cursor: pointer;"
                    } else {
                        "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; background-color: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 5px; cursor: pointer;"
                    },
                    onclick: move |_| start_exercise(Exercise::TwoDigitChainAddSub),
                    {t!("two-digit-chain-add-sub")}
                }
            }

            // Middle column: question and answer
            div {
                style: "min-width: 400px; background: white; border-radius: 10px; padding: 30px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                if exercise().is_some() {
                    {
                        let question_font_size = if question().op2.is_some() { "40px" } else { "48px" };
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; color: #666; font-size: 16px; margin-bottom: 20px;",
                                span { {t!("question-counter")} " {progress} / {TOTAL_QUESTIONS}" }
                                span { {t!("elapsed-time")} " {elapsed} " {t!("seconds-suffix")} }
                            }

                            // One icon per question: waiting pig (🐷), the raw cut
                            // currently on the fire (🥩), and the finished dish (🍖).
                            div {
                                style: "display: grid; grid-template-columns: repeat(10, 1fr); gap: 4px; \
                                       margin-bottom: 10px; user-select: none;",
                                for i in 0..TOTAL_QUESTIONS {
                                    div {
                                        key: "{i}",
                                        class: if i == progress() && !finished() { "pig-cell current" } else { "pig-cell" },
                                        if i < progress() {
                                            span { class: "pig-done", "🍖" }
                                        } else if i == progress() && !finished() {
                                            span { class: "mini-fire", "🔥🔥" }
                                            span { class: "pig-cooking", "🥩" }
                                        } else {
                                            span { "🐷" }
                                        }
                                    }
                                }
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
                                    style: "font-size: {question_font_size}; font-weight: bold; color: #333; padding: 30px 0; text-align: center;",
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

    #[test]
    fn chain_add_sub_keeps_all_terms_two_digit_and_steps_in_range() {
        for _ in 0..2000 {
            let q = random_question(Exercise::TwoDigitChainAddSub);
            let third = q.third.expect("chain question must have a third operand");
            let op2 = q.op2.expect("chain question must have a second operator");

            assert_eq!(q.blank, Blank::Result, "chain blank must be the result");
            assert!((10..=99).contains(&q.left), "left not two-digit: {q:?}");
            assert!((10..=99).contains(&q.right), "right not two-digit: {q:?}");
            assert!((10..=99).contains(&third), "third not two-digit: {q:?}");

            let first = match q.op {
                Operator::Add => q.left as i32 + q.right as i32,
                Operator::Sub => q.left as i32 - q.right as i32,
                Operator::Mul => unreachable!("chain add/sub must not multiply"),
            };
            assert!(
                (0..=100).contains(&first),
                "intermediate out of 0..=100: {q:?}"
            );

            let result = match op2 {
                Operator::Add => first + third as i32,
                Operator::Sub => first - third as i32,
                Operator::Mul => unreachable!("chain add/sub must not multiply"),
            };
            assert!((10..=99).contains(&result), "result not two-digit: {q:?}");
            assert_eq!(q.answer(), result as u32);
        }
    }
}
