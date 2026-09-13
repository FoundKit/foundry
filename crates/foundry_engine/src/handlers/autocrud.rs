use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::{ApiResponse, PaginatedData};
use foundry_storage::{ModelRecordEntity, ModelStore, RecordQuery, RecordStore};
use serde_json::Value;

/// GET /api/v1/s/:system_slug/:model_slug (List records with pagination and filters)
pub async fn list_records_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Query(query): Query<RecordQuery>,
) -> AppResult<Json<ApiResponse<PaginatedData<ModelRecordEntity>>>> {
    // Verify model exists
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;

    let records = RecordStore::list(&state.db, &system_slug, &model_slug, query).await?;
    Ok(Json(ApiResponse::success(records)))
}

/// GET /api/v1/s/:system_slug/:model_slug/:id (Get single record)
pub async fn get_record_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    let record = RecordStore::get_by_id(&state.db, &system_slug, &model_slug, id).await?;
    Ok(Json(ApiResponse::success(record)))
}

/// POST /api/v1/s/:system_slug/:model_slug (Create record with schema validation & hooks)
pub async fn create_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    let fields = ModelStore::list_fields(&state.db, model.id).await?;

    // In-memory schema validation
    RecordStore::validate_record(&fields, &payload)?;

    // Execute before_create mutation hooks
    state
        .hooks
        .execute_before_create(&ctx, &model_slug, &mut payload)
        .await?;

    let record = RecordStore::create(&state.db, &system_slug, &model_slug, payload.clone()).await?;

    // Execute after_create mutation hooks
    state
        .hooks
        .execute_after_create(&ctx, &model_slug, record.id, &payload)
        .await?;

    Ok(Json(ApiResponse::success(record)))
}

/// PUT /api/v1/s/:system_slug/:model_slug/:id (Update record with schema validation & hooks)
pub async fn update_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    let fields = ModelStore::list_fields(&state.db, model.id).await?;

    // In-memory schema validation
    RecordStore::validate_record(&fields, &payload)?;

    // Execute before_update mutation hooks
    state
        .hooks
        .execute_before_update(&ctx, &model_slug, id, &mut payload)
        .await?;

    let record =
        RecordStore::update(&state.db, &system_slug, &model_slug, id, payload.clone()).await?;

    // Execute after_update mutation hooks
    state
        .hooks
        .execute_after_update(&ctx, &model_slug, id, &payload)
        .await?;

    Ok(Json(ApiResponse::success(record)))
}

/// DELETE /api/v1/s/:system_slug/:model_slug/:id (Delete record with hooks)
pub async fn delete_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;

    // Execute before_delete mutation hooks
    state
        .hooks
        .execute_before_delete(&ctx, &model_slug, id)
        .await?;

    RecordStore::delete(&state.db, &system_slug, &model_slug, id).await?;

    // Execute after_delete mutation hooks
    state
        .hooks
        .execute_after_delete(&ctx, &model_slug, id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}
