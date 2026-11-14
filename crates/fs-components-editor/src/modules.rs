//! The extension points of the building blocks: declared, not built (ADR 0045, modules like hot plugins).
//!
//! Pattern: Registry — each building block will take modules of its own kind; the traits are empty for now and the registry
//! only keeps what is put into it.

/// A module of the `SourceEditor`: a language (highlighting, language server) or a preview.
pub trait SourceModule: Send + Sync {}

/// A module of the `BlockEditor`: a block type and its entry in the `/` menu.
pub trait BlockModule: Send + Sync {}

/// A module of the `RichTextEditor`: a mark or a button of its bar.
pub trait RichModule: Send + Sync {}

/// The modules of one kind that a building block has been given.
pub struct Registry<M: ?Sized> {
    modules: Vec<Box<M>>,
}

impl<M: ?Sized> Default for Registry<M> {
    fn default() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
}

impl<M: ?Sized> Registry<M> {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a module.
    pub fn register(&mut self, module: Box<M>) {
        self.modules.push(module);
    }

    /// How many modules there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Are there none?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}
