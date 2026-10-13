//! Mental arithmetic training game.

use dioxus::prelude::*;
use dioxus_i18n::t;
use dioxus_sdk::time::use_interval;
use std::time::Duration;

use crate::keypad_training::{self, DrillLevel, KeypadTraining, Press, Round};
use crate::rng::SimpleRng;
use crate::sound::{play_complete, play_correct, play_wrong};

/// How many questions a round of the expression drills asks.
const EXPRESSION_QUESTIONS: usize = 30;

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

/// Styles for the heartbeat that replaces the clock in the last questions.
const TIMER_CSS: &str = r#"
.time-pulse {
    display: inline-block;
    animation: time-beat 1s ease-in-out infinite;
}
@keyframes time-beat {
    0% { transform: scale(1); opacity: .85; }
    12% { transform: scale(1.35); opacity: 1; }
    35% { transform: scale(1); opacity: .85; }
    100% { transform: scale(1); opacity: .85; }
}
"#;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Exercise {
    MultiplicationTable,
    DivisionTable,
    TwoDigitAddSub,
    TwoDigitChainAddSub,
    MulMixedAddSub,
    MulDivMixedAddSub,
    /// Two equations over two fruits, solved by cancelling one unknown.
    TwoUnknowns,
}

impl Exercise {
    /// How many questions one round asks. Two unknowns costs far more work per
    /// question than an expression does — two equations to solve by elimination
    /// before the answer — so its round is a short one.
    fn questions(self) -> usize {
        match self {
            Exercise::TwoUnknowns => 10,
            _ => EXPRESSION_QUESTIONS,
        }
    }

    /// How many questions at the end run with the clock hidden: the last third
    /// of the round, so the finish is run on feel instead of on the stopwatch.
    fn hidden_clock_questions(self) -> usize {
        self.questions() / 3
    }

    /// The question the clock disappears at.
    fn clock_hides_at(self) -> usize {
        self.questions() - self.hidden_clock_questions()
    }
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
    Div,
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
            Operator::Div => "÷",
        }
    }
}

/// A single question. `blank` marks which position is hidden.
/// When `op2`/`third` are set the expression is evaluated left to right
/// (for `a × b ± c` and `a ÷ b ± c` this gives multiplication and division the
/// correct precedence).
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
            Operator::Div => value / operand,
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

/// What the board shows: a single expression, or two equations with two
/// unknowns. Both know the answer to type in and how to draw themselves.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Problem {
    Expression(Question),
    TwoUnknowns(FruitSystem),
}

impl Problem {
    /// The value the player has to type in.
    fn answer(&self) -> u32 {
        match self {
            Problem::Expression(question) => question.answer(),
            Problem::TwoUnknowns(system) => system.answer(),
        }
    }

    /// Text for the board; a two-unknown question uses one line per equation,
    /// which the board keeps as line breaks.
    fn display(&self) -> String {
        match self {
            Problem::Expression(question) => question.display(),
            Problem::TwoUnknowns(system) => system.display(),
        }
    }

    /// Type size for the board: the longer questions need smaller type. A
    /// two-unknown board is sized so that its longest equation — five fruit
    /// symbols — still fits on one line inside the narrowest board, since a
    /// wrapped equation would read like an extra equation.
    fn font_size(&self) -> &'static str {
        match self {
            Problem::Expression(question) if question.op2.is_none() => "48px",
            Problem::Expression(_) => "40px",
            Problem::TwoUnknowns(_) => "32px",
        }
    }
}

// ---------------------------------------------------------------------------
// Two unknowns
// ---------------------------------------------------------------------------

/// Fruits a question is built from, grouped by how alike they look. The two
/// fruits of a question come from different groups, so they are never hard to
/// tell apart on the board. The warm colours share a group: a red apple next to
/// a peach or an orange reads as poorly as 🍐 next to 🥝.
const LOOKALIKE_FRUITS: [&[&str]; 4] = [
    &["🍎", "🍓", "🍑", "🍊"], // warm: red, peach and orange
    &["🍌"],                  // yellow
    &["🍐", "🥝"],            // green
    &["🍇"],                  // purple
];

/// A fruit is worth at least two, so the product asked at the end is never a
/// trivial `1 × n` and the quotient never the dividend itself.
const MIN_FRUIT_VALUE: u8 = 2;
/// Largest fruit value, which keeps the product and the quotient inside the
/// 9×9 table.
const MAX_FRUIT_VALUE: u8 = 9;
/// Most fruit symbols on one line: the first fruit once plus the repeats of the
/// second, which keeps the equations short enough for the board.
const MAX_FRUITS_PER_EQUATION: u8 = 5;

/// One equation of a two-unknown question: each fruit counted `counts[j]` times,
/// where a negative count means that fruit is subtracted.
#[derive(Clone, Copy, PartialEq, Debug)]
struct FruitEquation {
    counts: [i8; 2],
    /// Right-hand side; kept two-digit.
    total: i32,
}

impl FruitEquation {
    /// The equation as text, positive term first so no line starts with a minus.
    fn display(&self, fruits: [&'static str; 2]) -> String {
        let order = if self.counts[0] > 0 { [0, 1] } else { [1, 0] };
        let sign = if self.counts[order[1]] < 0 { " − " } else { " + " };
        format!(
            "{}{sign}{} = {}",
            Self::term(fruits[order[0]], self.counts[order[0]]),
            Self::term(fruits[order[1]], self.counts[order[1]]),
            self.total
        )
    }

    /// A term: the fruit repeated once per count, joined by the term's own sign.
    fn term(fruit: &str, count: i8) -> String {
        let joiner = if count < 0 { " − " } else { " + " };
        vec![fruit; count.unsigned_abs() as usize].join(joiner)
    }
}

/// A question with two unknowns: two equations over the same two fruits, which
/// cancel one unknown when added or subtracted, and then the product or quotient
/// of the two fruit values is asked for.
///
/// The values are whole numbers, so the two equations always solve back to the
/// values the question was built from.
#[derive(Clone, Copy, PartialEq, Debug)]
struct FruitSystem {
    fruits: [&'static str; 2],
    values: [u8; 2],
    equations: [FruitEquation; 2],
    /// Asked at the end: `fruits[0] × fruits[1]` or `fruits[0] ÷ fruits[1]`.
    op: Operator,
}

impl FruitSystem {
    fn answer(&self) -> u32 {
        match self.op {
            Operator::Div => (self.values[0] / self.values[1]) as u32,
            _ => self.values[0] as u32 * self.values[1] as u32,
        }
    }

    /// The two equations, then the question about the two fruits.
    fn display(&self) -> String {
        format!(
            "{}\n{}\n{} {} {} = ?",
            self.equations[0].display(self.fruits),
            self.equations[1].display(self.fruits),
            self.fruits[0],
            self.op.sign(),
            self.fruits[1]
        )
    }
}

/// Two fruits that do not look alike, so a question never shows the same fruit
/// twice and never two fruits that are hard to tell apart.
fn pick_fruits(rng: &mut SimpleRng) -> [&'static str; 2] {
    let first = rng.gen_range(0, LOOKALIKE_FRUITS.len() as u8 - 1);
    let mut second = rng.gen_range(0, LOOKALIKE_FRUITS.len() as u8 - 1);
    if second == first {
        second = (second + 1) % LOOKALIKE_FRUITS.len() as u8;
    }
    [
        pick_fruit(rng, LOOKALIKE_FRUITS[first as usize]),
        pick_fruit(rng, LOOKALIKE_FRUITS[second as usize]),
    ]
}

/// One fruit out of a group, so the fruits that look alike both turn up across
/// questions.
fn pick_fruit(rng: &mut SimpleRng, group: &[&'static str]) -> &'static str {
    group[rng.gen_range(0, group.len() as u8 - 1) as usize]
}

fn random_sign(rng: &mut SimpleRng) -> i8 {
    if rng.gen_range(0, 1) == 0 {
        1
    } else {
        -1
    }
}

/// One equation whose two-digit right-hand side follows from `values`. The
/// second fruit is counted out and added, the first one is added or subtracted:
/// with two-digit totals those are the only two shapes an equation can take, so
/// a minus on the board always sits in front of the single first fruit
/// (`🍑 + 🍑 + 🍑 − 🍎 = 10`).
fn random_equation(rng: &mut SimpleRng, values: [u8; 2]) -> FruitEquation {
    let counts = [
        random_sign(rng),
        rng.gen_range(1, MAX_FRUITS_PER_EQUATION - 1) as i8,
    ];
    FruitEquation {
        counts,
        total: counts[0] as i32 * values[0] as i32 + counts[1] as i32 * values[1] as i32,
    }
}

/// A pair of fruit values and two equations that solve to them. The values are a
/// fact of the 9×9 table, both sides of each equation stay two-digit, and the
/// division that cancelling one unknown leaves is a table fact as well.
fn random_fruit_system(rng: &mut SimpleRng) -> FruitSystem {
    for _ in 0..1000 {
        let (op, values) = match random_mul_or_div(rng) {
            Operator::Div => {
                // Built from its factors, so the quotient is exact.
                let second = rng.gen_range(MIN_FRUIT_VALUE, 4);
                let quotient = rng.gen_range(MIN_FRUIT_VALUE, MAX_FRUIT_VALUE / second);
                (Operator::Div, [second * quotient, second])
            }
            _ => (
                Operator::Mul,
                [
                    rng.gen_range(MIN_FRUIT_VALUE, MAX_FRUIT_VALUE),
                    rng.gen_range(MIN_FRUIT_VALUE, MAX_FRUIT_VALUE),
                ],
            ),
        };

        let equations = [random_equation(rng, values), random_equation(rng, values)];
        // Both right-hand sides stay two-digit, like the add/sub practices.
        if equations.iter().any(|e| !(10..=99).contains(&e.total)) {
            continue;
        }
        // The equations have to be independent; the same counts behind the same
        // signs would be one line twice, with no single solution.
        let determinant = equations[0].counts[0] as i32 * equations[1].counts[1] as i32
            - equations[1].counts[0] as i32 * equations[0].counts[1] as i32;
        if determinant == 0 {
            continue;
        }
        // Cancelling the first fruit divides by the difference of the two counts
        // when its sign is the same in both equations and by their sum when the
        // signs differ; that divisor has to stay a table fact.
        let (first, second) = (
            equations[0].counts[1].unsigned_abs(),
            equations[1].counts[1].unsigned_abs(),
        );
        let divisor = if equations[0].counts[0] == equations[1].counts[0] {
            first.abs_diff(second)
        } else {
            first + second
        };
        // The symbol cap already keeps the divisor at eight or below; the check
        // is what keeps this a table fact if that cap ever grows.
        if divisor > MAX_FRUIT_VALUE {
            continue;
        }

        return FruitSystem {
            fruits: pick_fruits(rng),
            values,
            equations,
            op,
        };
    }

    // Practically unreachable: two equations over 2 and 3 that satisfy every rule
    // above. The apple and the banana look nothing alike.
    FruitSystem {
        fruits: ["🍎", "🍌"],
        values: [2, 3],
        equations: [
            FruitEquation {
                counts: [1, 5],
                total: 17,
            },
            FruitEquation {
                counts: [1, 3],
                total: 11,
            },
        ],
        op: Operator::Mul,
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

fn random_mul_or_div(rng: &mut SimpleRng) -> Operator {
    if rng.gen_range(0, 1) == 0 {
        Operator::Mul
    } else {
        Operator::Div
    }
}

fn random_problem(exercise: Exercise) -> Problem {
    let mut rng = SimpleRng::new();
    let question = match exercise {
        Exercise::MultiplicationTable => Question {
            left: rng.gen_range(1, 9),
            right: rng.gen_range(1, 9),
            third: None,
            op: Operator::Mul,
            op2: None,
            blank: Blank::Result,
        },
        Exercise::DivisionTable => {
            // The reverse of the multiplication table: dividend ÷ divisor =
            // quotient, all three within the table, so every answer is exact.
            // Divisor 1 would only ever ask for the dividend itself.
            let divisor = rng.gen_range(2, 9);
            let quotient = rng.gen_range(1, 9);
            let blank = match rng.gen_range(0, 2) {
                0 => Blank::Left,
                1 => Blank::Right,
                _ => Blank::Result,
            };
            Question {
                left: divisor * quotient,
                right: divisor,
                third: None,
                op: Operator::Div,
                op2: None,
                blank,
            }
        }
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
                    return Problem::Expression(Question {
                        left,
                        right,
                        third: Some(third),
                        op,
                        op2: Some(op2),
                        blank: Blank::Result,
                    });
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
        Exercise::MulDivMixedAddSub => {
            // a × b ± c or a ÷ b ± c (table fact first): the result stays within
            // two digits (0..=99) and the blank can be any of the four numbers.
            // Division questions are built from their factors so they are exact.
            let (op, left, right, fact) = match random_mul_or_div(&mut rng) {
                Operator::Div => {
                    let divisor = rng.gen_range(2, 9);
                    let quotient = rng.gen_range(1, 9);
                    (Operator::Div, divisor * quotient, divisor, quotient as i32)
                }
                _ => {
                    let left = rng.gen_range(1, 9);
                    let right = rng.gen_range(1, 9);
                    (Operator::Mul, left, right, left as i32 * right as i32)
                }
            };
            let op2 = random_add_or_sub(&mut rng);
            let third = match op2 {
                // fact + c <= 99
                Operator::Add => rng.gen_range(1, (99 - fact) as u8),
                // fact - c >= 0
                _ => rng.gen_range(1, fact as u8),
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
                op,
                op2: Some(op2),
                blank,
            }
        }
        Exercise::TwoUnknowns => return Problem::TwoUnknowns(random_fruit_system(&mut rng)),
    };
    Problem::Expression(question)
}

// ---------------------------------------------------------------------------
// Pace display
// ---------------------------------------------------------------------------

/// Whether the clock is hidden at this point in a round of `exercise`.
fn time_is_hidden(progress: usize, exercise: Exercise) -> bool {
    progress >= exercise.clock_hides_at()
}

/// Mood shown instead of the seconds, stepping up as the round runs on.
fn pace_tier(elapsed: u64) -> &'static str {
    match elapsed {
        0..=19 => "pace-steady",
        20..=39 => "pace-hurry",
        40..=59 => "pace-sprint",
        _ => "pace-final",
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
    let mut question = use_signal(|| random_problem(Exercise::MultiplicationTable));
    let mut progress = use_signal(|| 0usize);
    let mut input = use_signal(String::new);
    // None = no feedback yet, Some(true) = correct, Some(false) = wrong
    let mut feedback = use_signal(|| None::<bool>);
    let mut elapsed = use_signal(|| 0u64);
    let mut finished = use_signal(|| false);
    let mut history = use_signal(Vec::<u64>::new);
    // Sound effects are opt-in: the drills are often practised somewhere quiet,
    // so a fresh session starts silent and the toggle turns them on.
    let mut sound_enabled = use_signal(|| false);
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
                question.set(random_problem(ex));
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
            if done >= ex.questions() {
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
                question.set(random_problem(ex));
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

            // Heartbeat shown while the clock is hidden.
            style { dangerous_inner_html: TIMER_CSS }

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
                    label: t!("division-table"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::DivisionTable)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::DivisionTable)),
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
                PracticeButton {
                    label: t!("mul-div-mixed-add-sub"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::MulDivMixedAddSub)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::MulDivMixedAddSub)),
                }
                PracticeButton {
                    label: t!("two-unknowns"),
                    selected: practice() == Some(Practice::Arithmetic(Exercise::TwoUnknowns)),
                    onpick: move |_| start_practice(Practice::Arithmetic(Exercise::TwoUnknowns)),
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
                style: "min-width: 460px; background: white; border-radius: 10px; padding: 30px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                if let Some(Practice::Keypad(_)) = practice() {
                    KeypadTraining { round: keypad_round, feedback: keypad_feedback }
                } else if let Some(Practice::Arithmetic(exercise)) = practice() {
                    {
                        let question_font_size = question().font_size();
                        let total_questions = exercise.questions();
                        let hidden_questions = exercise.hidden_clock_questions();
                        let clock_hidden = time_is_hidden(progress(), exercise);
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; color: #666; font-size: 16px; margin-bottom: 20px;",
                                span { {t!("question-counter")} " {progress} / {total_questions}" }
                                if clock_hidden && !finished() {
                                    // The exact time is hidden for the last questions:
                                    // a heartbeat keeps the rhythm, a word keeps the
                                    // pressure, and the number comes back at the end.
                                    span {
                                        style: "display: inline-flex; align-items: center; gap: 6px; font-weight: bold;",
                                        span { class: "time-pulse", "💓" }
                                        span { {t!(pace_tier(elapsed()))} }
                                    }
                                } else {
                                    span { {t!("elapsed-time")} " {elapsed} " {t!("seconds-suffix")} }
                                }
                            }

                            if clock_hidden && !finished() {
                                div {
                                    style: "background: #fff3e0; border: 1px solid #ffcc80; color: #e65100; \
                                            border-radius: 8px; padding: 8px 12px; margin-bottom: 12px; \
                                            font-size: 16px; font-weight: bold;",
                                    {t!("time-hidden-banner", count: {hidden_questions})}
                                }
                            }

                            // One icon per question: waiting pig (🐷), the raw cut
                            // currently on the fire (🥩), and the finished dish (🍖).
                            div {
                                style: "display: grid; grid-template-columns: repeat(10, 1fr); gap: 4px; \
                                       margin-bottom: 10px; user-select: none;",
                                for i in 0..total_questions {
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
                                        {t!("time-revealed")} " {elapsed} " {t!("seconds-suffix")}
                                    }
                                    p {
                                        style: "color: #999; font-size: 14px;",
                                        {t!("esc-hint")}
                                    }
                                }
                            } else {
                                div {
                                    style: "font-size: {question_font_size}; font-weight: bold; color: #333; padding: 30px 0; text-align: center; white-space: pre-line;",
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

    /// The expression generator behind [`random_problem`], for the tests that
    /// only look at single-expression questions.
    fn expression_of(exercise: Exercise) -> Question {
        match random_problem(exercise) {
            Problem::Expression(question) => question,
            other => panic!("{exercise:?} is not a single expression: {other:?}"),
        }
    }

    #[test]
    fn two_digit_add_sub_terms_are_all_two_digit() {
        for _ in 0..2000 {
            let q = expression_of(Exercise::TwoDigitAddSub);
            assert!((10..=99).contains(&q.left), "left not two-digit: {q:?}");
            assert!((10..=99).contains(&q.right), "right not two-digit: {q:?}");

            let result = match q.op {
                Operator::Add => q.left as u32 + q.right as u32,
                Operator::Sub => q.left as u32 - q.right as u32,
                Operator::Mul | Operator::Div => {
                    unreachable!("two-digit add/sub must not multiply or divide")
                }
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
            match expression_of(Exercise::TwoDigitAddSub).blank {
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
            let q = expression_of(Exercise::TwoDigitChainAddSub);
            let third = q.third.expect("chain question must have a third operand");
            let op2 = q.op2.expect("chain question must have a second operator");

            assert_eq!(q.blank, Blank::Result, "chain blank must be the result");
            assert!((10..=99).contains(&q.left), "left not two-digit: {q:?}");
            assert!((10..=99).contains(&q.right), "right not two-digit: {q:?}");
            assert!((10..=99).contains(&third), "third not two-digit: {q:?}");

            let first = match q.op {
                Operator::Add => q.left as i32 + q.right as i32,
                Operator::Sub => q.left as i32 - q.right as i32,
                Operator::Mul | Operator::Div => unreachable!("chain add/sub must not multiply or divide"),
            };
            assert!(
                (0..=100).contains(&first),
                "intermediate out of 0..=100: {q:?}"
            );

            let result = match op2 {
                Operator::Add => first + third as i32,
                Operator::Sub => first - third as i32,
                Operator::Mul | Operator::Div => unreachable!("chain add/sub must not multiply or divide"),
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
            let q = expression_of(Exercise::MulMixedAddSub);
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
                Operator::Mul | Operator::Div => unreachable!("second operator must be + or -"),
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

    #[test]
    fn the_clock_hides_for_the_last_third_of_the_round() {
        // Whatever a round's length, the clock goes away for its last third.
        for exercise in [
            Exercise::MultiplicationTable,
            Exercise::DivisionTable,
            Exercise::TwoDigitAddSub,
            Exercise::TwoDigitChainAddSub,
            Exercise::MulMixedAddSub,
            Exercise::MulDivMixedAddSub,
            Exercise::TwoUnknowns,
        ] {
            let questions = exercise.questions();
            let hidden = exercise.hidden_clock_questions();
            assert!(hidden > 0, "no question runs with the clock hidden");
            assert!(
                !time_is_hidden(questions - hidden - 1, exercise),
                "the clock hides too early"
            );
            assert!(time_is_hidden(questions - hidden, exercise));
            assert!(time_is_hidden(questions, exercise));
        }

        // The expression drills ask thirty questions and hide the clock for the
        // last ten of them...
        assert_eq!(Exercise::MultiplicationTable.questions(), EXPRESSION_QUESTIONS);
        assert_eq!(Exercise::MultiplicationTable.hidden_clock_questions(), 10);
        // ...and two unknowns asks ten, with its last three hidden.
        assert_eq!(Exercise::TwoUnknowns.questions(), 10);
        assert_eq!(Exercise::TwoUnknowns.hidden_clock_questions(), 3);
    }

    #[test]
    fn the_pace_word_steps_up_with_the_time() {
        assert_eq!(pace_tier(0), "pace-steady");
        assert_eq!(pace_tier(19), "pace-steady");
        assert_eq!(pace_tier(20), "pace-hurry");
        assert_eq!(pace_tier(39), "pace-hurry");
        assert_eq!(pace_tier(40), "pace-sprint");
        assert_eq!(pace_tier(59), "pace-sprint");
        assert_eq!(pace_tier(60), "pace-final");
        assert_eq!(pace_tier(9_999), "pace-final");
    }

    #[test]
    fn division_table_questions_are_exact_table_facts() {
        let mut seen_left = false;
        let mut seen_right = false;
        let mut seen_result = false;

        for _ in 0..2000 {
            let q = expression_of(Exercise::DivisionTable);
            assert_eq!(q.op, Operator::Div, "division table must divide: {q:?}");
            assert!(
                q.third.is_none() && q.op2.is_none(),
                "single-step question: {q:?}"
            );
            assert!((2..=9).contains(&q.right), "divisor out of 2..=9: {q:?}");
            assert!(q.left <= 81, "dividend beyond the 9x9 table: {q:?}");

            let quotient = q.left / q.right;
            assert_eq!(q.left % q.right, 0, "division is not exact: {q:?}");
            assert!((1..=9).contains(&quotient), "quotient out of 1..=9: {q:?}");
            assert_eq!(
                q.display().matches('?').count(),
                1,
                "exactly one blank expected: {}",
                q.display()
            );

            let expected = match q.blank {
                Blank::Left => q.left as u32,
                Blank::Right => q.right as u32,
                Blank::Third => unreachable!("the division table has no third operand"),
                Blank::Result => quotient as u32,
            };
            assert_eq!(q.answer(), expected);

            match q.blank {
                Blank::Left => seen_left = true,
                Blank::Right => seen_right = true,
                Blank::Result => seen_result = true,
                Blank::Third => {}
            }
        }

        assert!(
            seen_left && seen_right && seen_result,
            "blank should cover the dividend, the divisor and the result"
        );
    }

    #[test]
    fn mul_div_mixed_add_sub_keeps_table_facts_and_two_digit_results() {
        let mut seen_mul = false;
        let mut seen_div = false;
        let mut seen_blank = [false; 4];

        for _ in 0..3000 {
            let q = expression_of(Exercise::MulDivMixedAddSub);
            let third = q.third.expect("mixed question must have a third operand");
            let op2 = q.op2.expect("mixed question must have a second operator");

            // The first step is always a table fact: a × b, or a ÷ b built from
            // its factors so that it divides exactly.
            let fact = match q.op {
                Operator::Mul => {
                    assert!((1..=9).contains(&q.left), "factor out of 1..=9: {q:?}");
                    assert!((1..=9).contains(&q.right), "factor out of 1..=9: {q:?}");
                    q.left as i32 * q.right as i32
                }
                Operator::Div => {
                    assert!((2..=9).contains(&q.right), "divisor out of 2..=9: {q:?}");
                    assert_eq!(q.left % q.right, 0, "division is not exact: {q:?}");
                    let quotient = q.left / q.right;
                    assert!((1..=9).contains(&quotient), "quotient out of 1..=9: {q:?}");
                    quotient as i32
                }
                _ => unreachable!("the first operator is × or ÷"),
            };
            let result = match op2 {
                Operator::Add => fact + third as i32,
                Operator::Sub => fact - third as i32,
                _ => unreachable!("the second operator is + or -"),
            };
            assert!((0..=99).contains(&result), "result out of 0..=99: {q:?}");
            assert_eq!(
                q.display().matches('?').count(),
                1,
                "exactly one blank expected: {}",
                q.display()
            );

            let expected = match q.blank {
                Blank::Left => q.left as u32,
                Blank::Right => q.right as u32,
                Blank::Third => third as u32,
                Blank::Result => result as u32,
            };
            assert_eq!(q.answer(), expected);

            match q.op {
                Operator::Mul => seen_mul = true,
                Operator::Div => seen_div = true,
                _ => {}
            }
            match q.blank {
                Blank::Left => seen_blank[0] = true,
                Blank::Right => seen_blank[1] = true,
                Blank::Third => seen_blank[2] = true,
                Blank::Result => seen_blank[3] = true,
            }
        }

        assert!(seen_mul && seen_div, "both × and ÷ questions should show up");
        assert!(
            seen_blank.iter().all(|seen| *seen),
            "blank should cover all four positions"
        );
    }

    /// The group of fruits that look alike a fruit belongs to.
    fn lookalike_group_of(fruit: &str) -> usize {
        LOOKALIKE_FRUITS
            .iter()
            .position(|group| group.contains(&fruit))
            .unwrap_or_else(|| panic!("{fruit} is in no fruit group"))
    }

    #[test]
    fn every_fruit_belongs_to_exactly_one_lookalike_group() {
        assert!(
            LOOKALIKE_FRUITS.len() >= 2,
            "two fruits need two groups to come from"
        );
        for (index, group) in LOOKALIKE_FRUITS.iter().enumerate() {
            assert!(!group.is_empty(), "fruit group {index} is empty");
            for fruit in group.iter() {
                assert_eq!(
                    LOOKALIKE_FRUITS
                        .iter()
                        .filter(|other| other.contains(fruit))
                        .count(),
                    1,
                    "{fruit} is in more than one fruit group"
                );
            }
        }
    }

    #[test]
    fn two_unknown_questions_are_two_digit_and_solve_by_elimination() {
        for _ in 0..2000 {
            let system = match random_problem(Exercise::TwoUnknowns) {
                Problem::TwoUnknowns(system) => system,
                other => panic!("expected two unknowns, got {other:?}"),
            };

            assert_ne!(
                system.fruits[0], system.fruits[1],
                "the same fruit twice: {system:?}"
            );
            assert_ne!(
                lookalike_group_of(system.fruits[0]),
                lookalike_group_of(system.fruits[1]),
                "the two fruits look alike: {system:?}"
            );
            for value in system.values {
                assert!(
                    (MIN_FRUIT_VALUE..=MAX_FRUIT_VALUE).contains(&value),
                    "fruit value outside the table: {system:?}"
                );
            }
            for equation in system.equations {
                assert!(
                    (10..=99).contains(&equation.total),
                    "total is not two-digit: {system:?}"
                );
                let symbols: u32 = equation.counts.iter().map(|c| c.unsigned_abs() as u32).sum();
                assert!(
                    (1..=MAX_FRUITS_PER_EQUATION as u32).contains(&symbols),
                    "wrong number of fruit symbols on one line: {system:?}"
                );
            }

            // Adding or subtracting the two equations cancels one unknown, and
            // either way they solve back to the values they were built from.
            let (one, two) = (system.equations[0], system.equations[1]);
            let determinant = one.counts[0] as i32 * two.counts[1] as i32
                - two.counts[0] as i32 * one.counts[1] as i32;
            assert_ne!(
                determinant, 0,
                "the two equations are not independent: {system:?}"
            );
            let solved_first =
                (one.total * two.counts[1] as i32 - two.total * one.counts[1] as i32) / determinant;
            let solved_second =
                (one.counts[0] as i32 * two.total - two.counts[0] as i32 * one.total) / determinant;
            assert_eq!(
                solved_first, system.values[0] as i32,
                "first fruit solves wrong: {system:?}"
            );
            assert_eq!(
                solved_second, system.values[1] as i32,
                "second fruit solves wrong: {system:?}"
            );

            // What is asked at the end is a fact of the 9×9 table.
            match system.op {
                Operator::Div => {
                    assert_eq!(
                        system.answer() * system.values[1] as u32,
                        system.values[0] as u32,
                        "division is not exact: {system:?}"
                    );
                    assert!(
                        system.answer() >= MIN_FRUIT_VALUE as u32,
                        "trivial division: {system:?}"
                    );
                }
                _ => assert_eq!(
                    system.answer(),
                    system.values[0] as u32 * system.values[1] as u32
                ),
            }
            assert!(system.answer() <= 81, "answer outside the table: {system:?}");
        }
    }

    #[test]
    fn two_unknown_questions_both_add_and_subtract_a_fruit() {
        let (mut added, mut subtracted) = (false, false);
        for _ in 0..2000 {
            if let Problem::TwoUnknowns(system) = random_problem(Exercise::TwoUnknowns) {
                if system.equations.iter().all(|e| e.counts[0] > 0) {
                    added = true;
                }
                if system.equations.iter().any(|e| e.counts[0] < 0) {
                    subtracted = true;
                }
            }
        }
        assert!(added, "no question adds the first fruit in both equations");
        assert!(subtracted, "no question subtracts the first fruit");
    }

    #[test]
    fn a_two_unknown_board_shows_both_equations_and_the_final_question() {
        // The apple and peach example, with the peaches trimmed to the five-symbol
        // cap: 4 apples, 3 peaches, so 4 × 3 = 12.
        let system = FruitSystem {
            fruits: ["🍎", "🍑"],
            values: [4, 3],
            equations: [
                FruitEquation {
                    counts: [1, 4],
                    total: 16,
                },
                FruitEquation {
                    counts: [1, 2],
                    total: 10,
                },
            ],
            op: Operator::Mul,
        };
        assert_eq!(
            system.display(),
            "🍎 + 🍑 + 🍑 + 🍑 + 🍑 = 16\n🍎 + 🍑 + 🍑 = 10\n🍎 × 🍑 = ?"
        );
        assert_eq!(system.answer(), 12);

        // A subtracted fruit keeps its own sign and the positive term leads.
        let subtracted = FruitSystem {
            fruits: ["🍎", "🍑"],
            values: [2, 5],
            equations: [
                FruitEquation {
                    counts: [-1, 4],
                    total: 18,
                },
                FruitEquation {
                    counts: [1, 3],
                    total: 17,
                },
            ],
            op: Operator::Mul,
        };
        assert_eq!(
            subtracted.display(),
            "🍑 + 🍑 + 🍑 + 🍑 − 🍎 = 18\n🍎 + 🍑 + 🍑 + 🍑 = 17\n🍎 × 🍑 = ?"
        );
        assert_eq!(subtracted.answer(), 10);

        let divided = FruitSystem {
            fruits: ["🍊", "🍐"],
            values: [6, 2],
            equations: [
                FruitEquation {
                    counts: [1, 4],
                    total: 14,
                },
                FruitEquation {
                    counts: [1, 3],
                    total: 12,
                },
            ],
            op: Operator::Div,
        };
        assert_eq!(divided.display().lines().last(), Some("🍊 ÷ 🍐 = ?"));
        assert_eq!(divided.answer(), 3);
    }
}
