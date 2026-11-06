//! Learning the real heights.
//!
//! Pattern: Observer + Strategy (`Probe`). After a layout, the heights of the messages built are
//! read back and replace the estimates. When the reader is not at the bottom, a message above the
//! view that turns out taller or shorter moves the offset by the same amount, so what is being
//! read does not jump.

use super::handle::MessageList;

impl MessageList {
    /// Id of the element of message `key`.
    pub(super) fn row_id(&self, key: &str) -> String {
        format!("{}-msg-{key}", self.id)
    }

    /// Read the heights of the built messages (through the probe) and keep them. Returns how many
    /// changed; draws again when any did. The widget calls this itself after each layout.
    pub fn measure(&self) -> usize {
        let (changed, shift) = {
            let mut inner = self.lock();
            let offset = self.position.offset();
            let anchor = inner.heights.at(offset);
            let probe = inner.probe.clone();
            let shown = inner.shown.clone();
            let (mut changed, mut shift) = (0, 0.0);
            for (index, key) in shown {
                let Some(height) = probe(&self.row_id(&key)) else {
                    continue;
                };
                if inner.heights.is_exact(index) && (height - inner.heights.get(index)).abs() < 0.5
                {
                    continue;
                }
                let delta = inner.heights.set(index, height, true);
                changed += 1;
                if index < anchor {
                    shift += delta;
                }
            }
            (changed, if inner.follow { 0.0 } else { shift })
        };
        if shift != 0.0 {
            self.position
                .apply_offset((self.position.offset() + shift).max(0.0));
        }
        if changed > 0 {
            self.live.refresh();
        }
        changed
    }
}
