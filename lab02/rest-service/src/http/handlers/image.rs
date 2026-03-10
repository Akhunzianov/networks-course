use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderValue, header},
    response::IntoResponse,
};
use tracing::{info, warn};

use crate::{
    error::{AppError, AppResult},
    http::dto::image::IconResponse,
    services::product::AttachIconInput,
    state::AppState,
};

pub async fn attach_icon(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    mut multipart: Multipart,
) -> AppResult<Json<IconResponse>> {
    info!(product_id = id, "attach_icon request");
    let mut bytes: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("invalid multipart body".into()))?
    {
        if field.name() == Some("icon") {
            content_type = field.content_type().map(|value| value.to_string());
            let data = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest("failed to read uploaded file".into()))?;

            bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = bytes.ok_or_else(|| {
        warn!(product_id = id, "attach_icon missing 'icon' field");
        AppError::BadRequest("multipart field 'icon' is required".into())
    })?;
    let content_type = content_type.ok_or_else(|| {
        warn!(product_id = id, "attach_icon missing content type");
        AppError::BadRequest("icon content type is required".into())
    })?;

    let stored_image = state
        .product_service
        .attach_icon(
            id,
            AttachIconInput {
                bytes,
                content_type,
            },
        )
        .await?;

    Ok(Json(IconResponse::from(stored_image)))
}

pub async fn get_attached_icon(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<impl IntoResponse> {
    info!(product_id = id, "get_attached_icon request");
    let stored_image = state.product_service.get_attached_icon(id).await?;
    let content_type = HeaderValue::from_str(stored_image.content_type())
        .map_err(|_| AppError::BadRequest("invalid stored content type".into()))?;

    Ok((
        [(header::CONTENT_TYPE, content_type)],
        Body::from(stored_image.into_bytes()),
    ))
}
