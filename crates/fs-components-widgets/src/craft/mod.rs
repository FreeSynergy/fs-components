//! The building blocks of the Craft look (dark, square corners, 1 px lines, tiles that share their lines, small capitals in a mono
//! face): labels, square buttons, joined tabs, badges, numbered boxes, the progress bar, and the frames that hold them. A program
//! chooses the style (`fs_components_render::use_style(Some(StyleId::Craft))`) and puts its content into these.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

mod frame;
mod parts;

pub use frame::{
    BAR_HEIGHT, FOOTER_HEIGHT, TILE_PADDING, bar, bar_cell, footer, grid, link, section_title,
    tile, tile_columns, tile_head,
};
pub use parts::{
    Act, BadgeTone, Look, accent, accent_text, badge, blinc_colour, button, cell_button, fact,
    heading, label, label_dim, number_box, progress, tab, tabs, with_tip,
};
