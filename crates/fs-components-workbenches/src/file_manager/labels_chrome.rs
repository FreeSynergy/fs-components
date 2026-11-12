//! The words of the frame of the file window: tabs, the command bar, the sidebar.
//!
//! Pattern: none — plain data; every text comes from the caller (ADR 0053), the defaults are neutral English.

/// Words for the tabs, the command bar and the sidebar of the file manager.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChromeLabels {
    /// Default: `Coming soon`; added to the tooltip of what is shown but not built yet (ADR 0066).
    pub coming_soon: String,
    /// Default: `New`.
    pub new_menu: String,
    /// Default: `Cut`.
    pub cut: String,
    /// Default: `Copy`.
    pub copy: String,
    /// Default: `Paste`.
    pub paste: String,
    /// Default: `Share`.
    pub share: String,
    /// Default: `Sort by the next order`.
    pub sort: String,
    /// Default: `Details`.
    pub details: String,
    /// Default: `New tab`.
    pub new_tab: String,
    /// Default: `Close tab`.
    pub close_tab: String,
    /// Default: `Home`.
    pub group_home: String,
    /// Default: `Pinned`.
    pub group_pinned: String,
    /// Default: `Storage`.
    pub group_storage: String,
    /// Default: `Trash`.
    pub group_trash: String,
    /// Default: `Network`.
    pub group_network: String,
    /// Default: `Nothing connected yet`.
    pub network_empty: String,
    /// Default: `Settings`.
    pub settings: String,
    /// Default: `Search`.
    pub search_tip: String,
    /// Default: `Places`.
    pub places: String,
    /// Default: `Folders`.
    pub folders_title: String,
    /// Default: `Back`.
    pub nav_back: String,
    /// Default: `Forward`.
    pub nav_forward: String,
    /// Default: `Up one folder`.
    pub nav_up: String,
    /// Default: `Reload`.
    pub nav_reload: String,
    /// Default: `The rescan found changes`; what a screen reader says of the dot on the rescan button.
    pub changes_found: String,
    /// Default: `{} transfers are open`; what is said of the number on the transfers button.
    pub transfers_open: String,
    /// Default: `Running...`; shown while a long call (rescan, upload, download) is on its way.
    pub running: String,
    /// Default: `Still running; wait until it is done.`; said to a second start.
    pub busy: String,
}

impl Default for ChromeLabels {
    fn default() -> Self {
        let s = str::to_string;
        Self {
            coming_soon: s("Coming soon"),
            new_menu: s("New"),
            cut: s("Cut"),
            copy: s("Copy"),
            paste: s("Paste"),
            share: s("Share"),
            sort: s("Sort by the next order"),
            details: s("Details"),
            new_tab: s("New tab"),
            close_tab: s("Close tab"),
            group_home: s("Home"),
            group_pinned: s("Pinned"),
            group_storage: s("Storage"),
            group_trash: s("Trash"),
            group_network: s("Network"),
            network_empty: s("Nothing connected yet"),
            settings: s("Settings"),
            search_tip: s("Search"),
            places: s("Places"),
            folders_title: s("Folders"),
            nav_back: s("Back"),
            nav_forward: s("Forward"),
            nav_up: s("Up one folder"),
            nav_reload: s("Reload"),
            changes_found: s("The rescan found changes"),
            transfers_open: s("{} transfers are open"),
            running: s("Running..."),
            busy: s("Still running; wait until it is done."),
        }
    }
}
