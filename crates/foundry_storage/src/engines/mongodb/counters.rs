use foundry_core::error::{AppError, AppResult};
use mongodb::Database;
use mongodb::bson::doc;
use mongodb::options::ReturnDocument;

pub async fn next_id(db: &Database, counter_name: &str) -> AppResult<i64> {
    let coll = db.collection::<mongodb::bson::Document>("_counters");
    let filter = doc! { "_id": counter_name };
    let update = doc! { "$inc": { "seq": 1i64 } };

    let result = coll
        .find_one_and_update(filter, update)
        .upsert(true)
        .return_document(ReturnDocument::After)
        .await
        .map_err(|e| {
            AppError::Database(format!(
                "Failed to increment counter {}: {}",
                counter_name, e
            ))
        })?;

    if let Some(doc) = result
        && let Ok(seq) = doc.get_i64("seq")
    {
        return Ok(seq);
    }
    Err(AppError::Database(format!(
        "Failed to retrieve sequence number for {}",
        counter_name
    )))
}
