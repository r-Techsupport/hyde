use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOAuthQuery {
    pub code: String,
    pub state: String,
}

pub async fn get_oauth2_handler(
    State(state): State<AppState>,
    Query(query): Query<GetOAuthQuery>,
) -> Redirect {
    // Now you can trade it for an access token.
    let token_result = state
        .oauth
        .oauth_token_handler(query.code, query.state)
        .await;
    return Redirect::temporary("http://localhost:5173/editor");
}

// pub async fn get_oauth2_url(State(state): State<AppState>) -> String {
// }
