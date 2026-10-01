//! Mental arithmetic training game.

use dioxus::prelude::*;
use dioxus_i18n::t;
use dioxus_sdk::time::use_interval;
use std::time::Duration;

use crate::keypad_training::{self, DrillLevel, KeypadTraining, Press, Round};
use crate::rng::SimpleRng;
use crate::sound::{play_complete, play_correct, play_wrong};

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
    MulMixedAddSub,
}

/// One entry of the practice list: an arithmetic exercise, or a numeric-keypad
/// finger drill from [`crate::keypad_training`].
#[derive(Clone, Copy, PartialEq, Debug)]
enum Practice {
    Arithmetic(Exercise),
    Keypad(DrillLevel),
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
    Third,
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
/// When `op2`/`third` are set the expression is evaluated left to right
/// (for `a × b ± c` this gives multiplication the correct precedence).
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
            Blank::Third => self.third.unwrap_or(0) as u32,
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
        let third = self.third.map(|t| {
            if self.blank == Blank::Third {
                "?".to_string()
            } else {
                t.to_string()
            }
        });
        match (self.op2, third) {
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

// ---------------------------------------------------------------------------
// Question generation
// ---------------------------------------------------------------------------

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
        Exercise::MulMixedAddSub => {
            // a × b ± c (multiplication first): the result stays within two
            // digits (0..=99) and the blank can be any of the four numbers.
            let left = rng.gen_range(1, 9);
            let right = rng.gen_range(1, 9);
            let product = left as i32 * right as i32;
            let op2 = random_add_or_sub(&mut rng);
            let third = match op2 {
                // product + c <= 99
                Operator::Add => rng.gen_range(1, (99 - product) as u8),
                // product - c >= 0
                _ => rng.gen_range(1, product as u8),
            };
            let blank = match rng.gen_range(0, 3) {
                0 => Blank::Left,
                1 => Blank::Right,
                2 => Blank::Third,
                _ => Blank::Result,
            };
            Question {
                left,
                right,
                third: Some(third),
                op: Operator::Mul,
                op2: Some(op2),
                blank,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Practice list
// ---------------------------------------------------------------------------

/// A practice button of the left column; highlighted while its drill is running.
#[component]
fn PracticeButton(label: String, selected: bool, onpick: EventHandler<()>) -> Element {
    rsx! {
        button {
            style: if selected {
                "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; text-align: left; background-color: #FF9800; color: white; border: none; border-radius: 5px; cursor: pointer;"
            } else {
                "display: block; width: 100%; padding: 12px; margin: 5px 0; font-size: 16px; text-align: left; background-color: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 5px; cursor: pointer;"
            },
            onclick: move |_| onpick.call(()),
            "{label}"
        }
    }
}

#[component]
pub fn MentalMath() -> Element {
    let mut practice = use_signal(|| None::<Practice>);
    let mut question = use_signal(|| random_question(Exercise::MultiplicationTable));
    let mut progress = use_signal(|| 0usize);
    let mut input = use_signal(String::new);
    // None = no feedback yet, Some(true) = correct, Some(false) = wrong
    let mut feedback = use_signal(|| None::<bool>);
    let mut elapsed = use_signal(|| 0u64);
    let mut finished = use_signal(|| false);
    let mut history = use_signal(Vec::<u64>::new);
    let mut sound_enabled = use_signal(|| true);
    // Keypad drill: the round lives here too, so starting a drill (like starting
    // an arithmetic exercise) begins from a clean slate.
    let mut keypad_round = use_signal(|| Round::new(DrillLevel::Progressive));
    let mut keypad_feedback = use_signal(|| None::<Press>);

    use_interval(Duration::from_secs(1), move |()| {
        match practice() {
            Some(Practice::Arithmetic(_)) if !finished() => elapsed += 1,
            Some(Practice::Keypad(_)) => keypad_round.write().tick(),
            _ => {}
        }
    });

    // Focus what the running practice reads its input from: the answer input for
    // an arithmetic exercise, the screen itself for a keypad drill (its key
    // handler sits on the screen, and clicking a button does not move focus on
    // every platform).
    use_effect(move || {
        let focus_id = match practice() {
            Some(Practice::Arithmetic(_)) if !finished() => Some("mental-math-answer"),
            Some(Practice::Keypad(_)) => Some("mental-math-screen"),
            _ => None,
        };
        if let Some(id) = focus_id {
            spawn(async move {
                let _ = dioxus::document::eval(&format!(
                    r#"document.getElementById("{id}")?.focus();"#
                ))
                .await;
            });
        }
    });

    let mut start_practice = move |p: Practice| {
        practice.set(Some(p));
        match p {
            Practice::Arithmetic(ex) => {
                question.set(random_question(ex));
                progress.set(0);
                input.set(String::new());
                feedback.set(None);
                elapsed.set(0);
                finished.set(false);
            }
            Practice::Keypad(level) => {
                keypad_round.set(Round::new(level));
                keypad_feedback.set(None);
            }
        }
    };

    let mut submit_answer = move || {
        let Some(Practice::Arithmetic(ex)) = practice() else {
            return;
        };
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
                if sound_enabled() {
                    play_complete();
                }
            } else {
                if sound_enabled() {
                    play_correct();
                }
                question.set(random_question(ex));
            }
        } else {
            feedback.set(Some(false));
            if sound_enabled() {
                play_wrong();
            }
        }
        input.set(String::new());
    };

    rsx! {
        div {
            id: "mental-math-screen",
            tabindex: "0",
            style: "display: flex; justify-content: center; gap: 40px; align-items: flex-start; max-width: 1200px; margin: 0 auto; outline: none;",
            onkeydown: move |event: Event<KeyboardData>| {
                use dioxus::prelude::Key;
                if event.key() == Key::Escape {
                    match practice() {
                        Some(Practice::Arithmetic(ex)) => start_practice(Practice::Arithmetic(ex)),
                        Some(Practice::Keypad(_)) => {
                            keypad_round.write().restart();
                            keypad_feedback.set(None);
                        }
                        None => {}
                    }
                }

                // The write guard has to be dropped before the round is read
                // again below, so collect the outcome first.
                let press = keypad_training::handle_key(&mut keypad_round.write(), &event);
                if let Some(press) = press {
                    keypad_feedback.set(Some(press));
                        match press {
                            Press::Correct { question_done: true } => {
                                if sound_enabled() {
                                    play_correct();
                                }
                            }
                            Press::RoundFinished => {
                                let secs = keypad_round.read().elapsed();
                                history.write().push(secs);
                                if sound_enabled() {
                                    play_complete();
                                }
                            }
                            Press::Wrong | Press::WrongRow => {
                                if sound_enabled() {
                                    play_wrong();
                                }
                            }
                            Press::Correct { question_done: false } => {}
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
                PracticeButton {
                    label: t!("multiplication-table"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::MultiplicationTable)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::MultiplicationTable)),
                }
                PracticeButton {
                    label: t!("two-digit-add-sub"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::TwoDigitAddSub)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::TwoDigitAddSub)),
                }
                PracticeButton {
                    label: t!("two-digit-chain-add-sub"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::TwoDigitChainAddSub)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::TwoDigitChainAddSub)),
                }
                PracticeButton {
                    label: t!("mul-mixed-add-sub"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::MulMixedAddSub)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::MulMixedAddSub)),
                }

                // Numeric-keypad finger drills, alongside the arithmetic ones.
                hr { style: "margin: 14px 0 8px; border: none; border-top: 1px solid #eee;" }
                p {
                    style: "margin: 0; color: #333; font-size: 15px; font-weight: bold; text-align: left;",
                    {t!("keypad-practice")}
                }
                for level in DrillLevel::all() {
                    PracticeButton {
                        key: "{level.i18n_key()}",
                        label: t!(level.i18n_key()),
                        selected: practice() == Some(Practice::Keypad(level)),
                        onpick: move |_| start_practice(Practice::Keypad(level)),
                    }
                }

                button {
                    style: format!(
                        "display: block; width: 100%; padding: 10px; margin: 16px 0 5px; font-size: 15px; \
                         border-radius: 5px; cursor: pointer; transition: all 0.2s; {}",
                        if sound_enabled() {
                            "background-color: #e8f5e9; color: #2e7d32; border: 1px solid #a5d6a7;"
                        } else {
                            "background-color: #f5f5f5; color: #999; border: 1px solid #ddd;"
                        }
                    ),
                    onclick: move |_| sound_enabled.set(!sound_enabled()),
                    if sound_enabled() {
                        {t!("sound-on")}
                    } else {
                        {t!("sound-off")}
                    }
                }
            }

            // Middle column: question and answer
            div {
                style: "min-width: 400px; background: white; border-radius: 10px; padding: 30px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                if let Some(Practice::Keypad(_)) = practice() {
                    KeypadTraining { round: keypad_round, feedback: keypad_feedback }
                } else if practice().is_some() {
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

// ---------------------------------------------------------------------------
// Sound effects live in `crate::sound`, shared with the keypad drill.
// ---------------------------------------------------------------------------

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
                Blank::Third => unreachable!("two-digit add/sub never blanks the third operand"),
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
                Blank::Third => {}
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

    #[test]
    fn mul_mixed_add_sub_keeps_result_within_two_digits() {
        let mut seen_left = false;
        let mut seen_right = false;
        let mut seen_third = false;
        let mut seen_result = false;
        for _ in 0..2000 {
            let q = random_question(Exercise::MulMixedAddSub);
            let third = q.third.expect("mul mixed question must have a third operand");
            let op2 = q.op2.expect("mul mixed question must have a second operator");

            assert_eq!(q.op, Operator::Mul, "first operator must be ×");
            assert!((1..=9).contains(&q.left), "factor out of 1..=9: {q:?}");
            assert!((1..=9).contains(&q.right), "factor out of 1..=9: {q:?}");
            assert!((1..=99).contains(&third), "third operand out of 1..=99: {q:?}");
            assert_eq!(
                q.display().matches('?').count(),
                1,
                "exactly one blank expected: {}",
                q.display()
            );

            let product = q.left as i32 * q.right as i32;
            let result = match op2 {
                Operator::Add => product + third as i32,
                Operator::Sub => product - third as i32,
                Operator::Mul => unreachable!("second operator must be + or -"),
            };
            assert!((0..=99).contains(&result), "result out of 0..=99: {q:?}");

            let expected = match q.blank {
                Blank::Left => q.left as u32,
                Blank::Right => q.right as u32,
                Blank::Third => third as u32,
                Blank::Result => result as u32,
            };
            assert_eq!(q.answer(), expected);

            match q.blank {
                Blank::Left => seen_left = true,
                Blank::Right => seen_right = true,
                Blank::Third => seen_third = true,
                Blank::Result => seen_result = true,
            }
        }
        assert!(
            seen_left && seen_right && seen_third && seen_result,
            "blank should cover all four positions"
        );
    }

}
