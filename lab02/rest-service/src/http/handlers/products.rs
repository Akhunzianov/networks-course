use axum::{
    Json,
    extract::{Path, State},
};
use tracing::info;

use crate::{
    error::AppResult,
    http::dto::products::{CreateProductRequest, ProductResponse, UpdateProductRequest},
    services::product::{CreateProductInput, UpdateProductInput},
    state::AppState,
};

pub async fn create_product(
    State(state): State<AppState>,
    Json(body): Json<CreateProductRequest>,
) -> AppResult<Json<ProductResponse>> {
    info!(
        name = %body.name,
        description_len = body.description.len(),
        "create_product request"
    );
    let product = state
        .product_service
        .create_product(CreateProductInput {
            name: body.name,
            description: body.description,
        })
        .await?;

    Ok(Json(ProductResponse::from(product)))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<Json<ProductResponse>> {
    info!(product_id = id, "get_product request");
    let product = state.product_service.get_product(id).await?;
    Ok(Json(ProductResponse::from(product)))
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(body): Json<UpdateProductRequest>,
) -> AppResult<Json<ProductResponse>> {
    info!(
        product_id = id,
        name_present = body.name.is_some(),
        description_present = body.description.is_some(),
        "update_product request"
    );
    let product = state
        .product_service
        .update_product(
            id,
            UpdateProductInput {
                name: body.name,
                description: body.description,
                icon: None,
            },
        )
        .await?;

    Ok(Json(ProductResponse::from(product)))
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<Json<ProductResponse>> {
    info!(product_id = id, "delete_product request");
    let product = state.product_service.delete_product(id).await?;
    Ok(Json(ProductResponse::from(product)))
}

pub async fn list_products(State(state): State<AppState>) -> AppResult<Json<Vec<ProductResponse>>> {
    info!("list_products request");
    let products = state.product_service.list_products().await?;
    Ok(Json(
        products.into_iter().map(ProductResponse::from).collect(),
    ))
}
