use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::{ApiResponse, PaginatedData};
use foundry_storage::{ModelRecordEntity, RecordQuery, coerce_record_data, validate_record};
use serde_json::Value;

/// Query parameters for retrieving a single record with optional field projection
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct SingleRecordQuery {
    pub fields: Option<String>,
    pub exclude_fields: Option<String>,
}

/// GET /api/s/:system_slug/:model_slug (List records with pagination and filters)
pub async fn list_records_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Query(query): Query<RecordQuery>,
) -> AppResult<Json<ApiResponse<PaginatedData<ModelRecordEntity>>>> {
    let model = state
        .db
        .models()
        .get_model(&system_slug, &model_slug)
        .await?;
    let fields = state.db.models().list_fields(model.id).await?;

    let query_fields = query.fields.clone();
    let query_exclude_fields = query.exclude_fields.clone();

    let mut records = state
        .db
        .records()
        .list(&system_slug, &model_slug, query)
        .await?;

    for rec in &mut records.items {
        coerce_record_data(&fields, &mut rec.data);
    }

    // Apply custom field projection / filtering
    if let Some(ref fields_str) = query_fields {
        let requested_fields: std::collections::HashSet<&str> = fields_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !requested_fields.is_empty() {
            for rec in &mut records.items {
                if let Some(obj) = rec.data.as_object_mut() {
                    obj.retain(|k, _| requested_fields.contains(k.as_str()));
                }
            }
        }
    } else if let Some(ref exclude_str) = query_exclude_fields {
        let excluded_fields: std::collections::HashSet<&str> = exclude_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for rec in &mut records.items {
            if let Some(obj) = rec.data.as_object_mut() {
                obj.retain(|k, _| !excluded_fields.contains(k.as_str()));
            }
        }
    }

    Ok(Json(ApiResponse::success(records)))
}

/// GET /api/s/:system_slug/:model_slug/:id (Get single record with optional field projection)
pub async fn get_record_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
    Query(query): Query<SingleRecordQuery>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = state
        .db
        .models()
        .get_model(&system_slug, &model_slug)
        .await?;
    let fields = state.db.models().list_fields(model.id).await?;
    let mut record = state
        .db
        .records()
        .get_by_id(&system_slug, &model_slug, id)
        .await?;

    coerce_record_data(&fields, &mut record.data);

    // Apply custom field projection / filtering
    if let Some(ref fields_str) = query.fields {
        let requested_fields: std::collections::HashSet<&str> = fields_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !requested_fields.is_empty()
            && let Some(obj) = record.data.as_object_mut()
        {
            obj.retain(|k, _| requested_fields.contains(k.as_str()));
        }
    } else if let Some(ref exclude_str) = query.exclude_fields {
        let excluded_fields: std::collections::HashSet<&str> = exclude_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(obj) = record.data.as_object_mut() {
            obj.retain(|k, _| !excluded_fields.contains(k.as_str()));
        }
    }

    Ok(Json(ApiResponse::success(record)))
}

/// POST /api/s/:system_slug/:model_slug (Create record with schema validation & hooks)
pub async fn create_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = state
        .db
        .models()
        .get_model(&system_slug, &model_slug)
        .await?;
    let fields = state.db.models().list_fields(model.id).await?;

    // Gracefully coerce payload to match target schema when admin/user saves
    coerce_record_data(&fields, &mut payload);

    // In-memory schema validation
    validate_record(&fields, &payload)?;

    // Retain only fields declared in the model schema
    let active_field_names: std::collections::HashSet<&str> =
        fields.iter().map(|f| f.name.as_str()).collect();
    if let Some(obj) = payload.as_object_mut() {
        obj.retain(|k, _| active_field_names.contains(k.as_str()));
    }

    // Execute before_create mutation hooks
    state
        .hooks
        .execute_before_create(&ctx, &model_slug, &mut payload)
        .await?;

    let record = state
        .db
        .records()
        .create(&system_slug, &model_slug, payload.clone())
        .await?;

    // Execute after_create mutation hooks
    state
        .hooks
        .execute_after_create(&ctx, &model_slug, record.id, &payload)
        .await?;

    Ok(Json(ApiResponse::success(record)))
}

/// PUT /api/s/:system_slug/:model_slug/:id (Update record with schema validation & hooks)
pub async fn update_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = state
        .db
        .models()
        .get_model(&system_slug, &model_slug)
        .await?;
    let fields = state.db.models().list_fields(model.id).await?;

    // Gracefully coerce payload to match target schema when admin/user saves
    coerce_record_data(&fields, &mut payload);

    // In-memory schema validation
    validate_record(&fields, &payload)?;

    // Retain only fields declared in the model schema, purging obsolete/removed fields
    let active_field_names: std::collections::HashSet<&str> =
        fields.iter().map(|f| f.name.as_str()).collect();
    if let Some(obj) = payload.as_object_mut() {
        obj.retain(|k, _| active_field_names.contains(k.as_str()));
    }

    // Execute before_update mutation hooks
    state
        .hooks
        .execute_before_update(&ctx, &model_slug, id, &mut payload)
        .await?;

    let record = state
        .db
        .records()
        .update(&system_slug, &model_slug, id, payload.clone())
        .await?;

    // Execute after_update mutation hooks
    state
        .hooks
        .execute_after_update(&ctx, &model_slug, id, &payload)
        .await?;

    Ok(Json(ApiResponse::success(record)))
}

/// DELETE /api/s/:system_slug/:model_slug/:id (Delete record with hooks)
pub async fn delete_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    let _model = state
        .db
        .models()
        .get_model(&system_slug, &model_slug)
        .await?;

    // Execute before_delete mutation hooks
    state
        .hooks
        .execute_before_delete(&ctx, &model_slug, id)
        .await?;

    state
        .db
        .records()
        .delete(&system_slug, &model_slug, id)
        .await?;

    // Execute after_delete mutation hooks
    state
        .hooks
        .execute_after_delete(&ctx, &model_slug, id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}
