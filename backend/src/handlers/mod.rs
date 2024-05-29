//! All Axum handlers are exported from here

mod get_doc;
pub use get_doc::*;
mod get_tree;
pub use get_tree::*;
mod oauth;
pub use oauth::*;
