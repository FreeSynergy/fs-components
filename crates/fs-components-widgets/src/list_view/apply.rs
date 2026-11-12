//! Showing the columns as the person's layout says.
//!
//! Pattern: Adapter — the columns the caller gave, narrowed and ordered by a `ColumnLayout`, plus the map from each
//! shown column back to the cell of a row it shows.

use std::sync::Arc;

use super::head::Layout;
use super::model::ListView;

impl ListView {
    /// Show the columns as the person's layout says: the shown ones, in its order, with the widths it gave. What the
    /// head needs to change the layout is given back. Without a layout every cell shows its own column.
    pub(super) fn apply_layout(&mut self) -> Option<Arc<Layout>> {
        let Some((layout, on_change)) = self.layout.take() else {
            self.cell_map = (0..self.columns.len()).collect();
            return None;
        };
        let titles: Vec<(String, String)> = self
            .columns
            .iter()
            .map(|c| (c.key(), c.title.clone()))
            .collect();
        let (mut columns, mut map) = (Vec::new(), Vec::new());
        for key in layout.shown() {
            if let Some(at) = self.columns.iter().position(|c| c.key() == key) {
                let mut column = self.columns[at].clone();
                if let Some(px) = layout.width_of(key) {
                    column.width = super::column::ColumnWidth::Pixels(px);
                }
                columns.push(column);
                map.push(at);
            }
        }
        if columns.is_empty() {
            // a layout that knows none of the columns: show them as they are
            self.cell_map = (0..self.columns.len()).collect();
            return None;
        }
        self.columns = columns;
        self.cell_map = map;
        Some(Arc::new(Layout {
            current: layout,
            on_change,
            titles,
        }))
    }
}
