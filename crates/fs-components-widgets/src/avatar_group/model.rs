//! The people of a group and how many of them fit.
//!
//! Pattern: none — plain data, no Blinc type; the view reads it.

/// How present a person is (only shown when the caller knows it).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    /// Available.
    Online,
    /// Away for now.
    Away,
    /// Do not disturb.
    Busy,
    /// Not there.
    Offline,
}

/// One person: an identity the caller understands, a name, perhaps a picture.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Person {
    /// Identity for the caller (it finds the person again in `shown()` and `hidden()`); never shown.
    pub key: String,
    /// The name; its initials are shown when there is no picture (or it cannot be loaded).
    pub name: String,
    /// Path or address of a picture.
    pub image: Option<String>,
    /// Presence, if known.
    pub presence: Option<Presence>,
}

impl Person {
    /// A person with a key and a name.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ..Self::default()
        }
    }

    /// With a picture.
    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// With a presence.
    #[must_use]
    pub fn with_presence(mut self, presence: Presence) -> Self {
        self.presence = Some(presence);
        self
    }
}

/// The size of the circles.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 24 px.
    ExtraSmall,
    /// 32 px.
    Small,
    /// 40 px.
    #[default]
    Medium,
    /// 48 px.
    Large,
    /// 64 px.
    ExtraLarge,
}

/// Most circles ever drawn, whatever `max` says: the rest is counted in the "+N" circle.
pub const MAX_SHOWN: usize = 50;

/// Several people.
#[derive(Clone, Debug, Default)]
pub struct AvatarGroup {
    pub(super) people: Vec<Person>,
    pub(super) max: usize,
    pub(super) size: Size,
}

impl AvatarGroup {
    /// A group of `people`; five circles are shown unless [`AvatarGroup::max`] says otherwise.
    pub fn new(people: Vec<Person>) -> Self {
        Self {
            people,
            max: 5,
            size: Size::default(),
        }
    }

    /// Show at most `max` circles (at most [`MAX_SHOWN`]); the others become one "+N" circle.
    #[must_use]
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    /// The size of the circles.
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// How many circles are drawn with a person in them.
    pub fn shown_count(&self) -> usize {
        self.people.len().min(self.max.min(MAX_SHOWN))
    }

    /// The people drawn as circles.
    pub fn shown(&self) -> &[Person] {
        &self.people[..self.shown_count()]
    }

    /// The people counted in the "+N" circle (for a tooltip or a list the caller opens).
    pub fn hidden(&self) -> &[Person] {
        &self.people[self.shown_count()..]
    }
}

/// The letters shown for `name`: the first letter of the first and of the last word that has a
/// letter or digit, in capitals; one letter for one word; `?` when there is none. Invisible
/// characters (zero width, direction marks), symbols and emoji are skipped, because they draw as
/// nothing or as half a sign; a letter keeps the marks that belong to it (accents).
pub fn initials(name: &str) -> String {
    let mut words = name.split_whitespace().filter_map(first_letter);
    let first = words.next();
    let last = words.next_back();
    match (first, last) {
        (Some(a), Some(b)) => format!("{a}{b}"),
        (Some(a), None) => a,
        _ => "?".to_string(),
    }
}

/// The first letter or digit of `word` with the combining marks after it, in capitals.
fn first_letter(word: &str) -> Option<String> {
    let mut chars = word.chars().skip_while(|c| !c.is_alphanumeric());
    let head = chars.next()?;
    let mut out: String = head.to_uppercase().collect();
    out.extend(chars.take_while(|c| matches!(c, '\u{300}'..='\u{36F}')));
    Some(out)
}
