//! Which screen the window shows.
//!
//! Pattern: State — the states of [`crate::Session`]; the transitions are its methods.

/// Which screen the window shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    /// The list with its search and the commands for the list.
    List,
    /// One record and the commands for it.
    Detail(String),
    /// The form of a command; `record` is the record it works on.
    Command {
        /// The command.
        name: String,
        /// The record it works on, if any.
        record: Option<String>,
    },
    /// "Really?" before a command that deletes.
    Confirm {
        /// The command.
        name: String,
        /// The record it works on.
        record: String,
    },
    /// What a command answered.
    Result(String),
}

impl Screen {
    /// A key that is the same for the same screen: the scope of its messages and its help address.
    pub fn key(&self) -> String {
        match self {
            Screen::List => "list".to_owned(),
            Screen::Detail(id) => format!("detail:{id}"),
            Screen::Command { name, .. } => format!("command:{name}"),
            Screen::Confirm { name, .. } => format!("confirm:{name}"),
            Screen::Result(name) => format!("result:{name}"),
        }
    }
}
