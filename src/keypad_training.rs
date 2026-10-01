//! Numeric keypad (ten-key) touch-typing drill.
//!
//! The player picks a drill on the practice list, then types the key sequence
//! shown on the panel — digits followed by Enter — three times in a row. Every
//! key carries the finger that should press it, so the drill teaches the numpad
//! finger assignment as much as the key sequence itself.
//!
//! The practice screen owns the [`Round`] (so picking a drill can start a fresh
//! one, exactly like picking an arithmetic exercise) and forwards key events to
//! [`handle_key`]; this module holds the state machine, the key classification
//! and the panel that draws both.

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::rng::SimpleRng;

/// Questions in one round. Nine questions split the progressive drill into
/// three one-digit, three two-digit and three three-digit questions.
const QUESTIONS_PER_ROUND: usize = 9;

/// How often the key sequence of one question is typed in a row.
const REPEATS_PER_QUESTION: usize = 3;

/// One entry of the keypad group in the practice list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrillLevel {
    /// One digit + Enter.
    OneDigit,
    /// Two digits + Enter.
    TwoDigits,
    /// Three digits + Enter.
    ThreeDigits,
    /// Walks 1 → 2 → 3 digits across one round.
    Progressive,
}

impl DrillLevel {
    /// The keypad entries, in the order the practice list shows them.
    pub fn all() -> [DrillLevel; 4] {
        [
            DrillLevel::OneDigit,
            DrillLevel::TwoDigits,
            DrillLevel::ThreeDigits,
            DrillLevel::Progressive,
        ]
    }

    /// Translation key of the entry's label.
    pub fn i18n_key(self) -> &'static str {
        match self {
            DrillLevel::OneDigit => "keypad-one-digit",
            DrillLevel::TwoDigits => "keypad-two-digits",
            DrillLevel::ThreeDigits => "keypad-three-digits",
            DrillLevel::Progressive => "keypad-progressive",
        }
    }

    /// Digits asked for in question `index` (0-based) of a round.
    fn digit_count(self, index: usize) -> usize {
        match self {
            DrillLevel::OneDigit => 1,
            DrillLevel::TwoDigits => 2,
            DrillLevel::ThreeDigits => 3,
            // A third of the round per digit count; the tests pin the exact
            // sequence this produces for a full round.
            DrillLevel::Progressive => 1 + index / (QUESTIONS_PER_ROUND / 3),
        }
    }
}

/// One key of a practice sequence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PadKey {
    Digit(u8),
    Enter,
}

impl PadKey {
    /// Digit keys are always 0..=9: they come from a 0..=9 random range or from
    /// parsing a single ASCII digit, never from arbitrary input.
    fn digit(value: u8) -> Self {
        debug_assert!(value <= 9, "not a digit key: {value}");
        PadKey::Digit(value)
    }

    /// What the player sees on a key: the digit, or the Enter arrow.
    fn glyph(self) -> String {
        match self {
            PadKey::Digit(value) => value.to_string(),
            PadKey::Enter => "⏎".to_string(),
        }
    }

    /// The key as it is named in the press hint, e.g. “press 7 …”.
    fn name(self) -> String {
        match self {
            PadKey::Digit(value) => value.to_string(),
            PadKey::Enter => t!("keypad-enter-name"),
        }
    }

    /// Touch typing on the numpad: one finger per column, with the thumb resting
    /// on 0 and the pinky reaching over to Enter.
    fn finger(self) -> Finger {
        match self {
            PadKey::Digit(0) => Finger::Thumb,
            PadKey::Digit(1 | 4 | 7) => Finger::Index,
            PadKey::Digit(2 | 5 | 8) => Finger::Middle,
            PadKey::Digit(3 | 6 | 9) => Finger::Ring,
            // Unreachable for the 0..=9 keys the drill builds; keep the drill
            // running instead of panicking inside the UI.
            PadKey::Digit(_) => Finger::Index,
            PadKey::Enter => Finger::Pinky,
        }
    }
}

/// The finger that presses a key on the numeric keypad.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl Finger {
    /// Translation key of the finger's name.
    fn i18n_key(self) -> &'static str {
        match self {
            Finger::Thumb => "finger-thumb",
            Finger::Index => "finger-index",
            Finger::Middle => "finger-middle",
            Finger::Ring => "finger-ring",
            Finger::Pinky => "finger-pinky",
        }
    }

    fn name(self) -> String {
        t!(self.i18n_key())
    }

    /// Colour of this finger's keys; the same colour is used on the on-screen
    /// numpad, on the key chips and in the legend.
    fn color(self) -> &'static str {
        match self {
            Finger::Thumb => "#8e24aa",
            Finger::Index => "#1e88e5",
            Finger::Middle => "#388e3c",
            Finger::Ring => "#f57c00",
            Finger::Pinky => "#d81b60",
        }
    }

    /// Lighter background for the same colour.
    fn background(self) -> &'static str {
        match self {
            Finger::Thumb => "#f6e9fa",
            Finger::Index => "#e7f1fd",
            Finger::Middle => "#e9f6ea",
            Finger::Ring => "#fff3e6",
            Finger::Pinky => "#fde8ef",
        }
    }
}

/// The finger assignment as a legend: which finger covers which keys.
const FINGER_LEGEND: [(Finger, &str); 5] = [
    (Finger::Thumb, "0"),
    (Finger::Index, "1 · 4 · 7"),
    (Finger::Middle, "2 · 5 · 8"),
    (Finger::Ring, "3 · 6 · 9"),
    (Finger::Pinky, "⏎"),
];

/// Progress inside the current question: the key sequence, how many complete
/// repetitions are done, and how far the repetition in progress got.
#[derive(Clone, PartialEq, Debug)]
struct Drill {
    target: Vec<PadKey>,
    repeats_done: usize,
    typed: usize,
}

impl Drill {
    fn new(target: Vec<PadKey>) -> Self {
        Self {
            target,
            repeats_done: 0,
            typed: 0,
        }
    }

    fn target(&self) -> &[PadKey] {
        &self.target
    }

    fn repeats_done(&self) -> usize {
        self.repeats_done
    }

    fn typed(&self) -> usize {
        self.typed
    }

    fn next_key(&self) -> Option<PadKey> {
        self.target.get(self.typed).copied()
    }

    /// Drops the partial repetition; the completed ones are kept.
    fn restart_repetition(&mut self) {
        self.typed = 0;
    }

    /// Advances after a matching key. Returns `true` once the sequence has been
    /// typed for the last required time.
    fn advance(&mut self) -> bool {
        self.typed += 1;
        if self.typed < self.target.len() {
            return false;
        }
        self.typed = 0;
        self.repeats_done += 1;
        self.repeats_done >= REPEATS_PER_QUESTION
    }
}

/// A key press, as far as the drill is concerned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Input {
    /// A key of the numeric keypad, or the only Enter a keyboard has.
    Numpad(PadKey),
    /// A digit from the row above the letters: the drill wants the numpad.
    NumberRow,
    /// Anything the drill ignores.
    Ignored,
}

/// What one press did, for the caller's sound and feedback.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Press {
    /// The key matched; `question_done` marks the third repetition, which
    /// completes the question.
    Correct { question_done: bool },
    /// Wrong numpad key: the repetition restarted.
    Wrong,
    /// A number-row digit: the repetition restarted with a hint to use the
    /// numpad.
    WrongRow,
    /// The third repetition of the last question: the round is over.
    RoundFinished,
}

impl Press {
    /// Message shown to the player, if any.
    fn i18n_key(self) -> &'static str {
        match self {
            Press::Correct { .. } | Press::RoundFinished => "keypad-correct",
            Press::Wrong => "keypad-wrong",
            Press::WrongRow => "keypad-wrong-row",
        }
    }

    fn is_correct(self) -> bool {
        matches!(self, Press::Correct { .. } | Press::RoundFinished)
    }
}

/// State of one drill round.
#[derive(Clone, PartialEq, Debug)]
pub struct Round {
    level: DrillLevel,
    question_index: usize,
    drill: Drill,
    presses: usize,
    mistakes: usize,
    elapsed: u64,
    finished: bool,
}

impl Round {
    /// A fresh round of `level`, starting with the first question.
    pub fn new(level: DrillLevel) -> Self {
        Self {
            level,
            question_index: 0,
            drill: Drill::new(random_question(level.digit_count(0))),
            presses: 0,
            mistakes: 0,
            elapsed: 0,
            finished: false,
        }
    }

    pub fn level(&self) -> DrillLevel {
        self.level
    }

    /// 0-based index of the question being typed.
    pub fn question_index(&self) -> usize {
        self.question_index
    }

    /// The key sequence of the current question, for the panel.
    fn target(&self) -> &[PadKey] {
        self.drill.target()
    }

    /// How far the repetition in progress got, as an index into [`Round::target`].
    pub fn typed(&self) -> usize {
        self.typed_prefix().len()
    }

    /// Complete repetitions of the current question.
    pub fn repeats_done(&self) -> usize {
        self.drill.repeats_done()
    }

    /// The key to press next, `None` once the round is over.
    fn next_key(&self) -> Option<PadKey> {
        if self.finished {
            return None;
        }
        self.drill.next_key()
    }

    pub fn mistakes(&self) -> usize {
        self.mistakes
    }

    pub fn elapsed(&self) -> u64 {
        self.elapsed
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Percentage of key presses that were right, for the end-of-round lines.
    pub fn accuracy(&self) -> usize {
        if self.presses == 0 {
            return 100;
        }
        (self.presses - self.mistakes) * 100 / self.presses
    }

    /// Adds one second while the round is running.
    pub fn tick(&mut self) {
        if !self.finished {
            self.elapsed += 1;
        }
    }

    /// Starts the same drill over (Esc).
    pub fn restart(&mut self) {
        let level = self.level();
        *self = Self::new(level);
    }

    /// Keys already typed in the repetition in progress.
    fn typed_prefix(&self) -> &[PadKey] {
        &self.drill.target()[..self.drill.typed()]
    }

    /// Handles one press; `None` once the round is over.
    fn press(&mut self, input: Input) -> Option<Press> {
        if self.finished {
            return None;
        }
        self.presses += 1;

        let matches_next = match input {
            Input::Numpad(key) => self.drill.next_key() == Some(key),
            Input::NumberRow | Input::Ignored => false,
        };
        if !matches_next {
            // The repetition starts over, so the whole sequence has to be typed
            // correctly three times instead of being hunted key by key.
            self.mistakes += 1;
            self.drill.restart_repetition();
            return Some(match input {
                Input::NumberRow => Press::WrongRow,
                _ => Press::Wrong,
            });
        }

        if !self.drill.advance() {
            return Some(Press::Correct {
                question_done: false,
            });
        }
        self.question_index += 1;
        if self.question_index >= QUESTIONS_PER_ROUND {
            self.finished = true;
            return Some(Press::RoundFinished);
        }
        self.drill = Drill::new(random_question(self.level.digit_count(self.question_index)));
        Some(Press::Correct { question_done: true })
    }
}

/// `digit_count` random digit keys followed by Enter.
fn random_question(digit_count: usize) -> Vec<PadKey> {
    let mut rng = SimpleRng::new();
    let mut target: Vec<PadKey> = (0..digit_count)
        .map(|_| PadKey::digit(rng.gen_range(0, 9)))
        .collect();
    target.push(PadKey::Enter);
    target
}

/// What a key event means for the drill.
fn classify(code: Code, key: &Key) -> Input {
    let numpad_digit = match code {
        Code::Numpad0 => Some(0),
        Code::Numpad1 => Some(1),
        Code::Numpad2 => Some(2),
        Code::Numpad3 => Some(3),
        Code::Numpad4 => Some(4),
        Code::Numpad5 => Some(5),
        Code::Numpad6 => Some(6),
        Code::Numpad7 => Some(7),
        Code::Numpad8 => Some(8),
        Code::Numpad9 => Some(9),
        _ => None,
    };
    if let Some(value) = numpad_digit {
        return Input::Numpad(PadKey::digit(value));
    }

    match code {
        // A keyboard without a numpad only has the main Enter, which is pressed
        // with the same pinky, so both count as the Enter key.
        Code::NumpadEnter | Code::Enter => Input::Numpad(PadKey::Enter),
        Code::Digit0 | Code::Digit1 | Code::Digit2 | Code::Digit3 | Code::Digit4 | Code::Digit5
        | Code::Digit6 | Code::Digit7 | Code::Digit8 | Code::Digit9 => Input::NumberRow,
        // Some platforms report no physical key; fall back to the character so
        // the drill still works there.
        Code::Unidentified => match key {
            Key::Character(ch) if ch.len() == 1 => match ch.as_bytes()[0] {
                digit @ b'0'..=b'9' => Input::Numpad(PadKey::digit(digit - b'0')),
                _ => Input::Ignored,
            },
            Key::Enter => Input::Numpad(PadKey::Enter),
            _ => Input::Ignored,
        },
        _ => Input::Ignored,
    }
}

/// Turns one key event into a drill press, ignoring what the drill does not
/// care about.
pub fn handle_key(round: &mut Round, event: &Event<KeyboardData>) -> Option<Press> {
    if event.is_auto_repeating() {
        // Holding a key down must not type the sequence by itself.
        return None;
    }
    match classify(event.code(), &event.key()) {
        Input::Ignored => None,
        input => round.press(input),
    }
}

/// The drill panel: the keys to type, the finger hints and the on-screen numpad.
/// All state lives in the practice screen, which also handles the key events.
#[component]
pub fn KeypadTraining(round: Signal<Round>, feedback: Signal<Option<Press>>) -> Element {
    let state = round();
    let feedback = feedback();

    rsx! {
        div {
            style: "width: 100%;",

            div {
                style: "display: flex; justify-content: space-between; gap: 10px; color: #666; font-size: 15px; margin-bottom: 12px;",
                span { {t!("question-counter")} " {state.question_index() + 1} / {QUESTIONS_PER_ROUND}" }
                span {
                    {t!("keypad-repeat", n: state.repeats_done() + 1, total: REPEATS_PER_QUESTION)}
                }
                span { {t!("elapsed-time")} " {state.elapsed()} " {t!("seconds-suffix")} }
                span { {t!("keypad-mistakes")} " {state.mistakes()}" }
            }

            // One tick per question of the round.
            div {
                style: "display: grid; grid-template-columns: repeat({QUESTIONS_PER_ROUND}, 1fr); gap: 4px; margin-bottom: 18px; user-select: none;",
                for i in 0..QUESTIONS_PER_ROUND {
                    div {
                        key: "{i}",
                        style: format!("{TICK_LAYOUT} {}", tick_style(i, &state)),
                        "{tick_mark(i, &state)}"
                    }
                }
            }

            if state.finished() {
                div {
                    style: "padding: 20px 0; text-align: center;",
                    h2 { style: "color: #4CAF50; margin: 0 0 10px 0;", {t!("keypad-finished")} }
                    p {
                        style: "font-size: 20px; color: #333; margin: 6px 0;",
                        {t!("elapsed-time")} " {state.elapsed()} " {t!("seconds-suffix")}
                    }
                    p {
                        style: "font-size: 18px; color: #333; margin: 6px 0;",
                        {t!("keypad-mistakes")} " {state.mistakes()} · "
                        {t!("keypad-accuracy")} " {state.accuracy()}%"
                    }
                    p { style: "color: #999; font-size: 14px;", {t!("keypad-esc-hint")} }
                }
            } else {
                // The sequence to type, with the finger for every key.
                div {
                    style: "display: flex; flex-wrap: wrap; justify-content: center; align-items: stretch; gap: 12px; margin-bottom: 8px;",
                    for (index, key) in state.target().iter().enumerate() {
                        div {
                            key: "{index}",
                            style: format!("{CHIP_LAYOUT} {}", chip_style(index, &state)),
                            div { style: "font-size: 30px; font-weight: bold; line-height: 1.15; color: #333;", "{key.glyph()}" }
                            div { style: "font-size: 12px; color: {key.finger().color()};", "{key.finger().name()}" }
                        }
                    }
                }

                p { style: "color: #999; font-size: 14px; margin: 4px 0 14px;", {t!("keypad-copy-hint")} }

                if let Some(next) = state.next_key() {
                    p {
                        style: "font-size: 21px; font-weight: bold; color: #333; margin: 0 0 14px;",
                        "👉 "
                        {t!("keypad-press-hint", finger: next.finger().name(), key: next.name())}
                    }
                }

                // The keypad itself, so the kid can see where the key sits.
                div {
                    style: "display: inline-grid; grid-template-columns: repeat(4, 62px); grid-template-rows: repeat(4, 52px); gap: 7px; margin: 0 auto 6px;",
                    for key in NUM_PAD_KEYS {
                        {numpad_cell(key, &state)}
                    }
                }

                div {
                    style: "display: flex; flex-wrap: wrap; justify-content: center; gap: 12px; border-top: 1px solid #eee; padding-top: 10px; margin-top: 6px;",
                    for (finger, keys) in FINGER_LEGEND {
                        span {
                            key: "{finger.i18n_key()}",
                            style: "font-size: 13px; color: {finger.color()};",
                            {finger.name()} " " {keys}
                        }
                    }
                }

                if let Some(press) = feedback {
                    p {
                        style: if press.is_correct() {
                            "color: #4CAF50; font-size: 18px; font-weight: bold; margin: 14px 0 0;"
                        } else {
                            "color: #f44336; font-size: 18px; font-weight: bold; margin: 14px 0 0;"
                        },
                        {t!(press.i18n_key())}
                    }
                }
            }

            p {
                style: "color: #999; font-size: 13px; margin: 14px 0 0;",
                {t!("keypad-numpad-note")}
            }
        }
    }
}

/// The keys of a real numpad, in the order they are added to the grid.
const NUM_PAD_KEYS: [PadKey; 11] = [
    PadKey::Digit(7),
    PadKey::Digit(8),
    PadKey::Digit(9),
    PadKey::Digit(4),
    PadKey::Digit(5),
    PadKey::Digit(6),
    PadKey::Digit(1),
    PadKey::Digit(2),
    PadKey::Digit(3),
    PadKey::Digit(0),
    PadKey::Enter,
];

/// Where a key sits on a numpad, as a CSS grid placement: 0 is wide, Enter is
/// the tall key on the bottom right.
fn grid_place(key: PadKey) -> &'static str {
    match key {
        PadKey::Digit(7) => "grid-area: 1 / 1;",
        PadKey::Digit(8) => "grid-area: 1 / 2;",
        PadKey::Digit(9) => "grid-area: 1 / 3;",
        PadKey::Digit(4) => "grid-area: 2 / 1;",
        PadKey::Digit(5) => "grid-area: 2 / 2;",
        PadKey::Digit(6) => "grid-area: 2 / 3;",
        PadKey::Digit(1) => "grid-area: 3 / 1;",
        PadKey::Digit(2) => "grid-area: 3 / 2;",
        PadKey::Digit(3) => "grid-area: 3 / 3;",
        PadKey::Digit(0) => "grid-area: 4 / 1 / 5 / 3;",
        PadKey::Enter => "grid-area: 3 / 4 / 5 / 5;",
        PadKey::Digit(_) => "",
    }
}

/// One key of the on-screen numpad: coloured by finger, highlighted while it is
/// the key to press next, dimmed once it has been typed in this repetition.
fn numpad_cell(key: PadKey, round: &Round) -> Element {
    let finger = key.finger();
    let is_next = round.next_key() == Some(key);
    let typed = !is_next && round.typed_prefix().contains(&key);
    let state_style = if is_next {
        "box-shadow: 0 0 0 4px #ffc107; transform: scale(1.06);"
    } else if typed {
        "opacity: 0.45;"
    } else {
        ""
    };

    rsx! {
        div {
            key: "{key.glyph()}",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; \
                    border-radius: 8px; border: 3px solid {finger.color()}; background: {finger.background()}; \
                    {grid_place(key)} {state_style}",
            div { style: "font-size: 22px; font-weight: bold; color: #333; line-height: 1.1;", "{key.glyph()}" }
            div { style: "font-size: 11px; color: {finger.color()};", "{key.finger().name()}" }
        }
    }
}

const TICK_LAYOUT: &str =
    "height: 26px; display: flex; align-items: center; justify-content: center; border-radius: 6px; font-size: 14px;";
const CHIP_LAYOUT: &str = "min-width: 60px; padding: 6px 10px; border-radius: 10px; text-align: center; \
                           background: #fafafa; border: 3px solid #ddd;";

/// Style of question tick `index`.
fn tick_style(index: usize, round: &Round) -> &'static str {
    if round.finished() || index < round.question_index() {
        "background: #e8f5e9; color: #2e7d32;"
    } else if index == round.question_index() {
        "background: #fff4d6; color: #ef6c00; box-shadow: 0 0 0 2px #ffc107;"
    } else {
        "background: #f5f5f5; color: #bbb;"
    }
}

/// Mark of question tick `index`.
fn tick_mark(index: usize, round: &Round) -> &'static str {
    if round.finished() || index < round.question_index() {
        "✓"
    } else if index == round.question_index() {
        "▶"
    } else {
        "·"
    }
}

/// Style of the key chip at `index` of the target sequence.
fn chip_style(index: usize, round: &Round) -> &'static str {
    if index < round.typed() {
        "border-color: #66bb6a; background: #e8f5e9; opacity: 0.6;"
    } else if index == round.typed() {
        "border-color: #ffb300; background: #fff8e1; box-shadow: 0 0 12px rgba(255,193,7,0.55); transform: scale(1.08);"
    } else {
        "border-color: #ddd;"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A round whose first question uses a fixed target, so a test can type it.
    fn round_with_target(target: Vec<PadKey>) -> Round {
        Round {
            level: DrillLevel::OneDigit,
            question_index: 0,
            drill: Drill::new(target),
            presses: 0,
            mistakes: 0,
            elapsed: 0,
            finished: false,
        }
    }

    fn numpad(value: u8) -> Input {
        Input::Numpad(PadKey::digit(value))
    }

    #[test]
    fn fingers_follow_the_numpad_columns() {
        assert_eq!(PadKey::digit(0).finger(), Finger::Thumb);
        for value in [1, 4, 7] {
            assert_eq!(PadKey::digit(value).finger(), Finger::Index, "digit {value}");
        }
        for value in [2, 5, 8] {
            assert_eq!(PadKey::digit(value).finger(), Finger::Middle, "digit {value}");
        }
        for value in [3, 6, 9] {
            assert_eq!(PadKey::digit(value).finger(), Finger::Ring, "digit {value}");
        }
        assert_eq!(PadKey::Enter.finger(), Finger::Pinky);
    }

    #[test]
    fn every_numpad_key_sits_on_the_grid() {
        for key in NUM_PAD_KEYS {
            assert!(!grid_place(key).is_empty(), "no grid cell for {key:?}");
        }
        assert_eq!(NUM_PAD_KEYS.len(), 11, "ten digits and Enter");
    }

    #[test]
    fn a_question_needs_three_clean_repetitions() {
        let mut round = round_with_target(vec![PadKey::digit(4), PadKey::Enter]);

        for repetition in 1..=3 {
            assert_eq!(
                round.press(numpad(4)),
                Some(Press::Correct {
                    question_done: false
                }),
                "repetition {repetition}: first key"
            );
            let last = round.press(Input::Numpad(PadKey::Enter));
            if repetition < 3 {
                assert_eq!(
                    last,
                    Some(Press::Correct {
                        question_done: false
                    }),
                    "repetition {repetition}: second key"
                );
                assert_eq!(round.repeats_done(), repetition);
            } else {
                assert_eq!(
                    last,
                    Some(Press::Correct {
                        question_done: true
                    }),
                    "the third repetition completes the question"
                );
            }
        }
        assert_eq!(round.question_index(), 1, "the next question is up");
        assert_eq!(round.repeats_done(), 0, "the new question starts fresh");
        assert_eq!(round.mistakes(), 0);
    }

    #[test]
    fn a_wrong_key_restarts_the_repetition() {
        let mut round = round_with_target(vec![PadKey::digit(4), PadKey::Enter]);
        assert_eq!(round.press(numpad(4)), Some(Press::Correct { question_done: false }));
        assert_eq!(round.typed(), 1);

        assert_eq!(round.press(numpad(9)), Some(Press::Wrong));
        assert_eq!(round.typed(), 0, "the repetition starts over");
        assert_eq!(round.repeats_done(), 0, "completed repetitions are kept as they were");
        assert_eq!(round.mistakes(), 1);

        // The same question is still up, and it can be typed cleanly now.
        assert_eq!(round.target(), [PadKey::digit(4), PadKey::Enter]);
        assert!(round.press(numpad(4)).is_some());
        assert!(round.press(Input::Numpad(PadKey::Enter)).is_some());
        assert_eq!(round.repeats_done(), 1);
    }

    #[test]
    fn number_row_digits_are_rejected_with_a_hint() {
        let mut round = round_with_target(vec![PadKey::digit(4), PadKey::Enter]);
        assert_eq!(round.press(Input::NumberRow), Some(Press::WrongRow));
        assert_eq!(round.mistakes(), 1);
        assert_eq!(round.typed(), 0);
        assert!(!Press::WrongRow.is_correct());
    }

    #[test]
    fn the_round_ends_after_the_last_question() {
        // Start from a fixed one-key question; the following questions use the
        // level's digit count, so the loop always presses whatever is next.
        let mut round = round_with_target(vec![PadKey::digit(1)]);
        let mut press_count = 0;

        while !round.finished() {
            let key = round.next_key().expect("a running round expects a key");
            let press = round
                .press(Input::Numpad(key))
                .expect("a running round accepts presses");
            press_count += 1;
            assert!(press_count < 1000, "the round has to end");
            if press == Press::RoundFinished {
                assert_eq!(round.question_index(), QUESTIONS_PER_ROUND);
            }
        }

        assert_eq!(
            round.question_index(),
            QUESTIONS_PER_ROUND,
            "every question of the round was answered"
        );
        assert_eq!(round.next_key(), None, "nothing left to press");
        assert_eq!(round.press(numpad(1)), None, "presses are ignored once finished");
        assert_eq!(round.mistakes(), 0);
        assert_eq!(round.accuracy(), 100, "no mistakes were made");

        round.tick();
        assert_eq!(round.elapsed(), 0, "a finished round stops counting");
    }

    #[test]
    fn elapsed_time_stops_when_the_round_is_over() {
        let mut round = round_with_target(vec![PadKey::digit(1)]);
        round.tick();
        round.tick();
        assert_eq!(round.elapsed(), 2);
        round.finished = true;
        round.tick();
        assert_eq!(round.elapsed(), 2, "a finished round stops counting");
    }

    #[test]
    fn restarting_keeps_the_level_and_clears_the_progress() {
        let mut round = Round::new(DrillLevel::TwoDigits);
        round.mistakes = 3;
        round.elapsed = 42;
        let level = round.level();
        assert_eq!(level, DrillLevel::TwoDigits);

        round.restart();
        assert_eq!(round.level(), level);
        assert_eq!(round.mistakes(), 0);
        assert_eq!(round.elapsed(), 0);
        assert_eq!(round.question_index(), 0);
        assert!(!round.finished());
        assert_eq!(round.target().len(), 3, "two digits plus Enter");
    }

    #[test]
    fn digit_counts_follow_the_level() {
        for index in 0..QUESTIONS_PER_ROUND {
            assert_eq!(DrillLevel::OneDigit.digit_count(index), 1);
            assert_eq!(DrillLevel::TwoDigits.digit_count(index), 2);
            assert_eq!(DrillLevel::ThreeDigits.digit_count(index), 3);
        }

        let progressive: Vec<usize> = (0..QUESTIONS_PER_ROUND)
            .map(|index| DrillLevel::Progressive.digit_count(index))
            .collect();
        assert_eq!(progressive, vec![1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn generated_questions_end_with_enter_and_use_digits() {
        for count in 1..=3 {
            for _ in 0..200 {
                let target = random_question(count);
                assert_eq!(target.len(), count + 1, "digits plus the Enter key");
                assert_eq!(target[count], PadKey::Enter, "Enter comes last");
                for key in &target[..count] {
                    match key {
                        PadKey::Digit(value) => assert!(*value <= 9, "not a digit key: {key:?}"),
                        PadKey::Enter => panic!("Enter must only be the last key"),
                    }
                    // Every key of a question also has a finger to show for it.
                    assert!(FINGER_LEGEND.iter().any(|(finger, _)| *finger == key.finger()));
                }
            }
        }
    }

    #[test]
    fn key_codes_are_classified_by_where_they_sit() {
        // The numpad is what the drill wants.
        assert_eq!(classify(Code::Numpad7, &Key::Character("7".to_string())), Input::Numpad(PadKey::Digit(7)));
        assert_eq!(classify(Code::Numpad0, &Key::Character("0".to_string())), Input::Numpad(PadKey::Digit(0)));
        assert_eq!(classify(Code::NumpadEnter, &Key::Enter), Input::Numpad(PadKey::Enter));
        // The main Enter is pressed with the same pinky, so it counts too.
        assert_eq!(classify(Code::Enter, &Key::Enter), Input::Numpad(PadKey::Enter));

        // The number row above the letters is flagged instead of accepted.
        assert_eq!(classify(Code::Digit7, &Key::Character("7".to_string())), Input::NumberRow);
        assert_eq!(classify(Code::Digit0, &Key::Character("0".to_string())), Input::NumberRow);

        // Everything else is ignored.
        assert_eq!(classify(Code::KeyA, &Key::Character("a".to_string())), Input::Ignored);
        assert_eq!(classify(Code::ArrowUp, &Key::ArrowUp), Input::Ignored);
        assert_eq!(classify(Code::ShiftLeft, &Key::Shift), Input::Ignored);

        // With no physical key reported, fall back to the character.
        assert_eq!(
            classify(Code::Unidentified, &Key::Character("5".to_string())),
            Input::Numpad(PadKey::Digit(5))
        );
        assert_eq!(classify(Code::Unidentified, &Key::Enter), Input::Numpad(PadKey::Enter));
        assert_eq!(classify(Code::Unidentified, &Key::ArrowUp), Input::Ignored);
    }

    #[test]
    fn accuracy_tracks_wrong_presses() {
        let mut round = round_with_target(vec![PadKey::digit(4), PadKey::Enter]);
        round.press(numpad(9));
        round.press(numpad(4));
        round.press(Input::NumberRow);
        assert_eq!(round.mistakes(), 2);
        assert_eq!(round.accuracy(), 33, "one of three presses was right");
    }
}
