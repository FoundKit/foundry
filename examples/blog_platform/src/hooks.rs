use foundry::prelude::*;
use serde_json::Value;

/// Custom application lifecycle hook to intercept entity mutations
pub struct BlogMutationHook;

#[async_trait]
impl MutationHook for BlogMutationHook {
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if model_slug == "posts" {
            tracing::info!(
                "Intercepted post creation in system '{}' for author: {:?}",
                ctx.system_slug,
                data.get("author")
            );
            if let Some(obj) = data.as_object_mut() {
                obj.insert("hook_processed".to_string(), serde_json::json!(true));
            }
        }
        Ok(())
    }

    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        tracing::info!(
            "Successfully created record {} in system '{}' (model: {})",
            record_id,
            ctx.system_slug,
            model_slug
        );
        Ok(())
    }
}
