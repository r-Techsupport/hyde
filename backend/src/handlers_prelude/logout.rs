use crate::AppState;
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;

/// Tell the browser on the other end to overwrite the access token, effectively logging the user out
pub async fn get_logout_handler() -> HeaderMap {
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        "Set-Cookie",
        "access-token=logged-out; Secure; HttpOnly; Path=/; "
            .parse()
            .expect("Statically defined logout cookie isn't valid"),
    );
    response_headers
}

pub async fn create_logout_route() -> Router<AppState> {
    Router::new().route("/logout", get(get_logout_handler))
}
