use async_trait::async_trait;
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use serde_json::Value;
use std::sync::Arc;

#[async_trait]
pub trait MutationHook: Send + Sync + 'static {
    async fn before_create(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _data: &mut Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn after_create(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn before_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &mut Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn after_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn before_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn after_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct HookPipeline {
    hooks: Vec<Arc<dyn MutationHook>>,
}

impl HookPipeline {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register<H: MutationHook>(&mut self, hook: H) {
        self.hooks.push(Arc::new(hook));
    }

    pub async fn execute_before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.before_create(ctx, model_slug, data).await?;
        }
        Ok(())
    }

    pub async fn execute_after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.after_create(ctx, model_slug, record_id, data).await?;
        }
        Ok(())
    }

    pub async fn execute_before_update(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &mut Value,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.before_update(ctx, model_slug, record_id, data).await?;
        }
        Ok(())
    }

    pub async fn execute_after_update(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.after_update(ctx, model_slug, record_id, data).await?;
        }
        Ok(())
    }

    pub async fn execute_before_delete(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.before_delete(ctx, model_slug, record_id).await?;
        }
        Ok(())
    }

    pub async fn execute_after_delete(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
    ) -> AppResult<()> {
        for hook in &self.hooks {
            hook.after_delete(ctx, model_slug, record_id).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestAuditHook {
        created_count: AtomicUsize,
    }

    #[async_trait]
    impl MutationHook for TestAuditHook {
        async fn before_create(
            &self,
            _ctx: &SystemContext,
            _model_slug: &str,
            data: &mut Value,
        ) -> AppResult<()> {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("injected_field".to_string(), serde_json::json!("injected"));
            }
            Ok(())
        }

        async fn after_create(
            &self,
            _ctx: &SystemContext,
            _model_slug: &str,
            _record_id: i64,
            _data: &Value,
        ) -> AppResult<()> {
            self.created_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_hook_pipeline_execution() {
        let mut pipeline = HookPipeline::new();
        let hook = TestAuditHook {
            created_count: AtomicUsize::new(0),
        };
        pipeline.register(hook);

        let ctx = SystemContext::new("test_slug");
        let mut payload = serde_json::json!({ "name": "original" });

        pipeline
            .execute_before_create(&ctx, "products", &mut payload)
            .await
            .unwrap();

        assert_eq!(payload["injected_field"], "injected");

        pipeline
            .execute_after_create(&ctx, "products", 101, &payload)
            .await
            .unwrap();
    }
}
