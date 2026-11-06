//! The avatar group as elements.
//!
//! Pattern: Adapter — our [`AvatarGroup`] in, `blinc_cn`'s avatar group out.

use blinc_cn::components::avatar::{AvatarBuilder, AvatarSize, avatar, avatar_group};

use super::model::{AvatarGroup, Person, Presence, Size};
use crate::Element;

impl AvatarGroup {
    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let shown = self.shown_count();
        let mut group = avatar_group().size(size(self.size)).max(shown);
        for (i, person) in self.people.iter().enumerate() {
            // The people who do not fit are only counted by the group: an empty circle stands in.
            group = group.child(if i < shown { one(person) } else { avatar() });
        }
        group.class("fs-avatars")
    }
}

fn one(person: &Person) -> AvatarBuilder {
    let mut a = avatar()
        .fallback(super::model::initials(&person.name))
        .alt(person.name.clone());
    // Blinc shows a grey disc for a picture it cannot load and never the initials, so a local file
    // that is not there is not handed over at all.
    if let Some(image) = person
        .image
        .as_ref()
        .filter(|i| crate::picture::loadable(i))
    {
        a = a.src(image.clone());
    }
    match person.presence {
        Some(Presence::Online) => a.online(),
        Some(Presence::Away) => a.away(),
        Some(Presence::Busy) => a.busy(),
        Some(Presence::Offline) => a.offline(),
        None => a,
    }
}

fn size(size: Size) -> AvatarSize {
    match size {
        Size::ExtraSmall => AvatarSize::ExtraSmall,
        Size::Small => AvatarSize::Small,
        Size::Large => AvatarSize::Large,
        Size::ExtraLarge => AvatarSize::ExtraLarge,
        _ => AvatarSize::Medium,
    }
}
