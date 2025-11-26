pub mod context;
pub mod react_query;
pub mod swr;

pub use context::HookContext;

/// Represents a generated hook file.
#[derive(Debug, Clone)]
pub struct HookFile {
    pub filename: String,
    pub content: String,
}

