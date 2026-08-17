pub mod custom_controller;

use axum::{Router, routing::post};

pub fn build_routes() -> Router {
    Router::new().route("/participate", post(custom_controller::handle_participate))
}
