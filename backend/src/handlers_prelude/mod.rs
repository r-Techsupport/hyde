//! All Axum handlers are exported from this module

mod doc;
pub use doc::*;
mod tree;
pub use tree::*;
mod oauth;
pub use oauth::*;

use color_eyre::Report;
use log::error;
use reqwest::StatusCode;

/// Quick and dirty way to convert an eyre error to a (StatusCode, message) response, meant for use with `map_err`, so that errors can be propagated out of
/// axum handlers with `?`.
pub fn eyre_to_axum_err(e: Report) -> (StatusCode, String) {
    error!("An error was encountered in an axum handler: {e:?}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("An error was encountered, check server logs for more info: {e}"),
    )
}
