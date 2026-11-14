//! The change operations of the model and what they do to a document.
//!
//! Pattern: Command — an operation is data; [`apply`] does it, [`diff`] finds the operations between two documents.
//! A split of a block is a replace of the block and an insert after it; a join is a replace and a remove.

use crate::Doc;

/// One change of a document. The index is the place in the document as it is when the operation runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// A new block at `index` (the blocks from there on move down).
    InsertBlock {
        /// Where it goes (0..=len).
        index: usize,
        /// The Markdown source of the block.
        text: String,
    },
    /// The block at `index` goes.
    RemoveBlock {
        /// Which block.
        index: usize,
    },
    /// The block at `index` gets this text.
    ReplaceBlock {
        /// Which block.
        index: usize,
        /// The new Markdown source of the block.
        text: String,
    },
}

/// An operation that does not fit the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpError(pub String);

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for OpError {}

/// Do `ops` to `doc`, in order.
///
/// # Errors
/// [`OpError`] if an index is outside the document; the document stays as it was before that operation.
pub fn apply(doc: &mut Doc, ops: &[Op]) -> Result<(), OpError> {
    for op in ops {
        match op {
            Op::InsertBlock { index, text } if *index <= doc.blocks.len() => {
                doc.blocks.insert(*index, text.clone());
            }
            Op::RemoveBlock { index } if *index < doc.blocks.len() => {
                doc.blocks.remove(*index);
            }
            Op::ReplaceBlock { index, text } if *index < doc.blocks.len() => {
                doc.blocks[*index].clone_from(text);
            }
            _ => {
                return Err(OpError(format!(
                    "{op:?} does not fit a document of {} blocks",
                    doc.blocks.len()
                )));
            }
        }
    }
    Ok(())
}

/// The operations that make `before` into `after`: nothing for equal documents; the blocks that differ in the middle are
/// replaced where both have one, inserted or removed where only one has.
#[must_use]
pub fn diff(before: &Doc, after: &Doc) -> Vec<Op> {
    let (a, b) = (&before.blocks, &after.blocks);
    let head = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let tail = a[head..]
        .iter()
        .rev()
        .zip(b[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (old, new) = (&a[head..a.len() - tail], &b[head..b.len() - tail]);
    let mut ops = Vec::new();
    for (k, text) in new.iter().enumerate().take(old.len()) {
        ops.push(Op::ReplaceBlock {
            index: head + k,
            text: text.clone(),
        });
    }
    for (k, text) in new.iter().enumerate().skip(old.len()) {
        ops.push(Op::InsertBlock {
            index: head + k,
            text: text.clone(),
        });
    }
    for _ in new.len()..old.len() {
        ops.push(Op::RemoveBlock {
            index: head + new.len(),
        });
    }
    ops
}
