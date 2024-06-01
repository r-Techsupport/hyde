use crate::git::INode;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use log::error;

/// This handler reads the document folder and builds a tree style object
/// representing the state of the tree. This is used in the viewer for directory navigation.
pub async fn get_tree_handler(
    State(state): State<AppState>,
) -> Result<Json<INode>, (StatusCode, &'static str)> {
    match state.git.get_doc_tree() {
        Ok(t) => Ok(Json(t)),
        Err(e) => {
            error!("An error was encountered fetching the document tree: {e:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error was encountered fetching the doc tree, \
                    check server logs for more info",
            ))
        }
    }
}
