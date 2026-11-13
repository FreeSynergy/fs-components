//! `Args` and `Value`: the named values a sentence is filled with.
//!
//! Pattern: Value object — small, immutable once built, compared by value. A value carries its
//! kind (text, count, number, date, time), never a finished string for numbers or dates, so each
//! language can format them itself.

/// One value with its kind.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Free text, inserted as it is (never evaluated again).
    Text(String),
    /// A whole number of things; selects the plural form.
    Count(u64),
    /// A number to show (not a count).
    Number(f64),
    /// A day, as seconds since 1970-01-01 UTC (any second of that day).
    Date(i64),
    /// A time of day, as seconds since 1970-01-01 UTC.
    Time(i64),
}

/// Named values for one sentence. Setting a name twice keeps the last value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Args(Vec<(String, Value)>);

impl Args {
    /// No values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value under a name.
    #[must_use]
    pub fn set(mut self, name: &str, value: Value) -> Self {
        match self.0.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = value,
            None => self.0.push((name.to_owned(), value)),
        }
        self
    }

    /// Set a [`Value::Text`].
    #[must_use]
    pub fn text(self, name: &str, text: impl Into<String>) -> Self {
        self.set(name, Value::Text(text.into()))
    }

    /// Set a [`Value::Count`].
    #[must_use]
    pub fn count(self, name: &str, count: u64) -> Self {
        self.set(name, Value::Count(count))
    }

    /// Set a [`Value::Number`].
    #[must_use]
    pub fn number(self, name: &str, number: f64) -> Self {
        self.set(name, Value::Number(number))
    }

    /// Set a [`Value::Date`] (Unix seconds).
    #[must_use]
    pub fn date(self, name: &str, unix: i64) -> Self {
        self.set(name, Value::Date(unix))
    }

    /// Set a [`Value::Time`] (Unix seconds).
    #[must_use]
    pub fn time(self, name: &str, unix: i64) -> Self {
        self.set(name, Value::Time(unix))
    }

    /// All values with their names, in the order they were first set.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(n, v)| (n.as_str(), v))
    }

    /// The value under a name, if any.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}
