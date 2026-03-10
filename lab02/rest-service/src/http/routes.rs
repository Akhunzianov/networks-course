use axum::{
    Router,
    routing::{get, post},
};

use crate::http::handlers::health::health;
use crate::http::handlers::image::{attach_icon, get_attached_icon};
use crate::http::handlers::products::{
    create_product, delete_product, get_product, list_products, update_product,
};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/product", post(create_product))
        .route(
            "/product/{product_id}",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/products", get(list_products))
        .route(
            "/product/{id}/image",
            get(get_attached_icon).post(attach_icon),
        )
        .with_state(state)
}
