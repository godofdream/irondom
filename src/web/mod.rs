use crate::app::AppState;
use axum::{extract::State, response::Html, routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(index))
        .route("/index.html", get(index))
}

async fn index(State(_state): State<AppState>) -> Html<&'static str> {
    Html(include_str!("../../assets/index.html"))
}
