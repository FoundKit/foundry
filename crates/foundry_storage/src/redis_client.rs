use foundry_core::error::{AppError, AppResult};
use redis::aio::ConnectionManager;
use tracing::info;

pub type RedisPool = ConnectionManager;

pub async fn init_redis(redis_url: &str) -> AppResult<RedisPool> {
    let client = redis::Client::open(redis_url)
        .map_err(|e| AppError::Internal(format!("Invalid Redis URL: {}", e)))?;

    let manager = ConnectionManager::new(client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to Redis: {}", e)))?;

    info!("Redis connection established successfully.");
    Ok(manager)
}
