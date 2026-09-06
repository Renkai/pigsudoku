//! Mental arithmetic training game (placeholder)

use dioxus::prelude::*;
use dioxus_i18n::t;

#[component]
pub fn MentalMath() -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 20px; padding: 40px;",
            h2 {
                style: "color: #333; margin: 0;",
                {t!("mental-math-title")}
            }
            p {
                style: "color: #666; font-size: 18px;",
                {t!("mental-math-coming-soon")}
            }
        }
    }
}
