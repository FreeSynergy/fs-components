//! The rows read so far, a few pages of them.
//!
//! Pattern: Decorator-like cache over the [`RowSource`] — the elements in view ask for one row at a
//! time, the source is asked for a page. Old pages are dropped, so scrolling through a very long
//! list keeps only a few in memory. The lock never leaves this file and no call to the source
//! happens under it.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};

use super::model::{Row, RowSource};

/// Rows asked of the source at a time.
pub(super) const PAGE: usize = 64;
/// Pages kept.
const KEPT: usize = 8;

type Page = (usize, Result<Vec<Row>, String>);

/// What the elements in view read from.
pub(super) struct RowCache {
    source: Arc<dyn RowSource>,
    pages: Mutex<VecDeque<Page>>,
}

impl RowCache {
    pub(super) fn new(source: Arc<dyn RowSource>) -> Self {
        Self {
            source,
            pages: Mutex::new(VecDeque::new()),
        }
    }

    /// Row `index`: `Ok(None)` if the source has fewer rows than it said, `Err` with its reason.
    pub(super) fn row(&self, index: usize) -> Result<Option<Row>, String> {
        let page = index / PAGE;
        if let Some(found) = self.look_up(page, index) {
            return found;
        }
        let read = self.source.rows(page * PAGE, PAGE);
        let mut pages = self.pages.lock().unwrap_or_else(PoisonError::into_inner);
        if pages.len() == KEPT {
            pages.pop_front();
        }
        pages.push_back((page, read));
        drop(pages);
        self.look_up(page, index).unwrap_or(Ok(None))
    }

    fn look_up(&self, page: usize, index: usize) -> Option<Result<Option<Row>, String>> {
        let pages = self.pages.lock().unwrap_or_else(PoisonError::into_inner);
        let (_, rows) = pages.iter().find(|(number, _)| *number == page)?;
        Some(match rows {
            Ok(rows) => Ok(rows.get(index - page * PAGE).cloned()),
            Err(why) => Err(why.clone()),
        })
    }
}
