//! One value of a field as the Admin frontend shows it: the compact component of its type, or the canonical text.
//!
//! Pattern: Strategy by basic type — the component of `fs-components-fields` where there is one ([`kind_of`]), else
//! the canonical text in the style's body font (what `types.capnp` promises for a kind a reader does not know).

use fs_components_fields::{Look, Say, compact, display};
use fs_components_widgets::Element;
use fs_types::{BasicType, FieldDescription};

use super::kinds::kind_of;

/// A value to read. A person's name and a postal address have parts worth showing apart (`display`); everything else
/// is one line.
pub fn show(
    look: &Look,
    say: &dyn Say,
    field: &FieldDescription,
    value: &str,
) -> Box<dyn Element> {
    let (s, pal) = (look.style, look.palette());
    match kind_of(field.basic) {
        Some(kind) => {
            let parts = kind.field().split(value);
            if matches!(field.basic, BasicType::Name | BasicType::Address) {
                Box::new(display(kind, &parts, look, say))
            } else {
                Box::new(compact(kind, &parts, look))
            }
        }
        None => {
            let text = if field.basic == BasicType::Choice {
                say.say(&format!("{}.{value}", field.label))
            } else {
                value.to_owned()
            };
            Box::new(
                blinc_layout::div::div()
                    .flex_row()
                    .items_center()
                    .min_h(s.text.body_line)
                    .child(look.text(text, s.text.body, pal.text)),
            )
        }
    }
}
