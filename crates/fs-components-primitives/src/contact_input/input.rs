//! The two inputs and the questions they ask.
//!
//! Pattern: Strategy (a function per kind) — `email_*` and `phone_*` bind the rule and the Blinc
//! input type to one shared builder, `field`.

use std::sync::Arc;

use blinc_cn::components::input::{InputBuilder, InputSize, input};
use blinc_layout::widgets::text_input::{InputType, SharedTextInputData};
use blinc_theme::{ColorToken, ThemeState};
use fs_interfaces::values::{ContactValue, EmailAddress, PhoneNumber, ValueProblem};

/// Called with the whole text after every change.
pub type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// What is wrong with `typed` as an e-mail address; `None` if it is fine or empty (empty means no value).
pub fn email_problem(typed: &str) -> Option<ValueProblem> {
    problem_of::<EmailAddress>(typed)
}

/// What is wrong with `typed` as a phone number; `None` if it is fine or empty.
pub fn phone_problem(typed: &str) -> Option<ValueProblem> {
    problem_of::<PhoneNumber>(typed)
}

fn problem_of<V: ContactValue>(typed: &str) -> Option<ValueProblem> {
    if typed.trim().is_empty() {
        return None;
    }
    V::check(typed).err()
}

/// An e-mail field over `data`, showing `placeholder` while empty (like `name@example.org`).
/// `problem` is the sentence to show under it now: the program asks [`email_problem`], words the
/// answer, and decides when a value is judged (it judges only what the person changed).
pub fn email_input(
    data: &SharedTextInputData,
    label: &str,
    placeholder: &str,
    problem: Option<String>,
    on_change: OnChange,
) -> InputBuilder {
    field(
        data,
        label,
        InputType::Email,
        placeholder,
        problem,
        on_change,
    )
}

/// A phone field over `data`, showing `placeholder` while empty (like `+49 30 1234567`); see
/// [`email_input`].
pub fn phone_input(
    data: &SharedTextInputData,
    label: &str,
    placeholder: &str,
    problem: Option<String>,
    on_change: OnChange,
) -> InputBuilder {
    field(data, label, InputType::Tel, placeholder, problem, on_change)
}

/// Text, placeholder and ground come from the theme of this build, as a pair (the layer's input
/// default set no text colour in 0.6.0).
fn field(
    data: &SharedTextInputData,
    label: &str,
    kind: InputType,
    placeholder: &str,
    problem: Option<String>,
    on_change: OnChange,
) -> InputBuilder {
    let theme = ThemeState::get();
    let mut built = input(data)
        .size(InputSize::Small)
        .label(label.to_string())
        .input_type(kind)
        .placeholder(placeholder.to_string())
        .w_full()
        .text_color(theme.color(ColorToken::TextPrimary))
        .placeholder_color(theme.color(ColorToken::TextTertiary))
        .idle_bg_color(theme.color(ColorToken::InputBg))
        .on_change(move |value| on_change(value));
    if let Some(message) = problem {
        built = built.error(message);
    }
    built
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_no_problem_and_the_rule_is_the_value_objects() {
        assert_eq!(email_problem(""), None);
        assert_eq!(email_problem("  "), None);
        assert_eq!(email_problem("ada"), Some(ValueProblem::MissingAt));
        assert_eq!(email_problem("ada@example.org"), None);
        assert_eq!(phone_problem("030 123"), Some(ValueProblem::NoCountryCode));
        assert_eq!(phone_problem("+49 30 1234567"), None);
    }
}
