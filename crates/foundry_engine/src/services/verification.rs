use foundry_core::error::{AppError, AppResult};
use foundry_storage::RedisPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

struct CodeEntry {
    code: String,
    expires_at: Instant,
    sent_at: Instant,
}

pub struct VerificationService {
    redis: Option<RedisPool>,
    memory_store: Arc<RwLock<HashMap<String, CodeEntry>>>,
}

impl VerificationService {
    pub fn new(redis: Option<RedisPool>) -> Self {
        Self {
            redis,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate 6-digit random code
    pub fn generate_code() -> String {
        format!("{:06}", fastrand::u32(100_000..=999_999))
    }

    /// Check cooldown and store verification code (TTL: 5 minutes = 300s, cooldown: 60s)
    pub async fn save_code(&self, scope: &str, identifier: &str, code: &str) -> AppResult<()> {
        let key = format!("{}:{}", scope, identifier);
        let now = Instant::now();

        if let Some(ref redis) = self.redis {
            let cooldown_key = format!("auth:cooldown:{}", key);
            let code_key = format!("auth:code:{}", key);

            let mut conn = redis.clone();
            let exists: bool = redis::cmd("EXISTS")
                .arg(&cooldown_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(false);

            if exists {
                return Err(AppError::Validation(
                    "验证码发送过于频繁，请等待60秒后再试 (Verification code requested too frequently, please wait 60s)".into()
                ));
            }

            let _: () = redis::cmd("SETEX")
                .arg(&code_key)
                .arg(300)
                .arg(code)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("Redis save code error: {}", e)))?;

            let _: () = redis::cmd("SETEX")
                .arg(&cooldown_key)
                .arg(60)
                .arg("1")
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("Redis save cooldown error: {}", e)))?;

            return Ok(());
        }

        // In-memory fallback
        let mut store = self.memory_store.write().await;
        if matches!(store.get(&key), Some(entry) if now < entry.sent_at + Duration::from_secs(60)) {
            return Err(AppError::Validation(
                "验证码发送过于频繁，请等待60秒后再试 (Verification code requested too frequently, please wait 60s)".into()
            ));
        }

        store.insert(
            key,
            CodeEntry {
                code: code.to_string(),
                expires_at: now + Duration::from_secs(300),
                sent_at: now,
            },
        );

        Ok(())
    }

    /// Verify code and delete on success
    pub async fn verify_code(
        &self,
        scope: &str,
        identifier: &str,
        input_code: &str,
    ) -> AppResult<()> {
        let key = format!("{}:{}", scope, identifier);

        if let Some(ref redis) = self.redis {
            let code_key = format!("auth:code:{}", key);
            let mut conn = redis.clone();

            let stored: Option<String> = redis::cmd("GET")
                .arg(&code_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(None);

            match stored {
                Some(saved_code) if saved_code == input_code.trim() => {
                    let _: () = redis::cmd("DEL")
                        .arg(&code_key)
                        .query_async(&mut conn)
                        .await
                        .unwrap_or(());
                    return Ok(());
                }
                _ => {
                    return Err(AppError::Validation(
                        "验证码错误或已过期 (Invalid or expired verification code)".into(),
                    ));
                }
            }
        }

        // In-memory fallback
        let mut store = self.memory_store.write().await;
        let now = Instant::now();
        if let Some(entry) = store.get(&key) {
            if now > entry.expires_at {
                store.remove(&key);
                return Err(AppError::Validation(
                    "验证码已过期，请重新获取 (Verification code has expired)".into(),
                ));
            }
            if entry.code == input_code.trim() {
                store.remove(&key);
                return Ok(());
            }
        }

        Err(AppError::Validation(
            "验证码错误或已失效 (Invalid or expired verification code)".into(),
        ))
    }
}
