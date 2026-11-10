//! Test support: the description of a contact (a type in the shape of a person).
#![allow(dead_code, clippy::pedantic)]

use fs_types::{BasicType, CommandDescription, CommandKind, FieldDescription, TypeDescription};

pub fn contact() -> TypeDescription {
    TypeDescription::new("Contact", "fs-test", "RFC 9553 Card", "contact")
        .field(
            FieldDescription::new("name", BasicType::Text)
                .required()
                .label("contact.name"),
        )
        .field(
            FieldDescription::new("emails", BasicType::Email)
                .multiple()
                .label("contact.emails"),
        )
        .field(FieldDescription::new("phone", BasicType::Phone).label("contact.phone"))
        .field(FieldDescription::new("note", BasicType::MultilineText).label("contact.note"))
        .command(
            CommandDescription::new("create", CommandKind::Create, "command.create")
                .input(
                    FieldDescription::new("name", BasicType::Text)
                        .required()
                        .label("contact.name"),
                )
                .input(
                    FieldDescription::new("emails", BasicType::Email)
                        .multiple()
                        .label("contact.emails"),
                ),
        )
        // change and delete have no input: the id travels in the call, change is an Edit of the fields
        .command(CommandDescription::new(
            "change",
            CommandKind::Change,
            "command.change",
        ))
        .command(CommandDescription::standard_find("command.find", "Contact"))
        .command(CommandDescription::new(
            "delete",
            CommandKind::Delete,
            "command.delete",
        ))
        .command(
            CommandDescription::new("domains", CommandKind::Suggest, "command.domains").result(
                FieldDescription::new("domains", BasicType::Text)
                    .multiple()
                    .label("command.domains"),
            ),
        )
}
