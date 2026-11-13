//! The composer's words as `MessageFormat` 2 files: paired, English unchanged from before the migration, German there, and
//! the chain of the language works through the `Localized` interface.

use fs_components_mf2::check_pairs;
use fs_components_widgets::message_composer::{Labels, TEXTS};
use fs_components_words::{Locale, Localized};

fn l(tag: &str) -> Labels {
    Labels::for_locale(&Locale::parse(tag).unwrap())
}

#[test]
fn every_id_is_in_german_and_english_with_the_same_variables() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
}

#[test]
fn the_english_default_is_what_it_was_before_the_files() {
    let d = Labels::default();
    assert_eq!(d.placeholder, "Write a message");
    assert_eq!((d.send.as_str(), d.sending.as_str(), d.attach.as_str(), d.remove.as_str()), ("Send", "Sending", "Attach", "Remove"));
    assert_eq!(d.failed, "Not sent: {}. Your text is still here.");
    assert_eq!(d.restored, "Draft restored.");
    assert_eq!(d.draft_failed, "Draft could not be saved: {}");
    assert_eq!(d, l("en"));
}

#[test]
fn german_is_german_in_every_field_and_a_region_or_unknown_language_follows_the_chain() {
    let (de, en) = (l("de"), l("en"));
    assert_eq!(de.send, "Senden");
    assert_eq!(de.failed, "Nicht gesendet: {}. Ihr Text ist noch da.");
    for (g, e) in [
        (&de.placeholder, &en.placeholder), (&de.send, &en.send), (&de.sending, &en.sending), (&de.attach, &en.attach),
        (&de.remove, &en.remove), (&de.failed, &en.failed), (&de.restored, &en.restored), (&de.draft_failed, &en.draft_failed),
    ] {
        assert_ne!(g, e, "{g} is not translated");
    }
    assert_eq!(l("de-AT"), de, "de-AT uses de");
    assert_eq!(l("fr"), en, "no French: English");
}
