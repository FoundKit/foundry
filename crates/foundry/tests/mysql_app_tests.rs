#[cfg(feature = "mysql")]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use foundry::prelude::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    struct TestSubsystem;

    impl SubsystemModule for TestSubsystem {
        fn slug(&self) -> &'static str {
            "demo"
        }

        fn display_name(&self) -> &'static str {
            "Demo Subsystem"
        }

        fn register_routes(&self, router: Router) -> Router {
            router.route("/ping", get(|| async { "pong" }))
        }
    }

    #[tokio::test]
    async fn test_foundry_app_builder_with_mysql() {
        let mysql_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/foundry_app_test".to_string());

        // Prepare test database if root connection is available
        if let Ok(root_pool) = sqlx::MySqlPool::connect("mysql://root:root@127.0.0.1:3306/").await {
            let _ = sqlx::query("DROP DATABASE IF EXISTS foundry_app_test;")
                .execute(&root_pool)
                .await;
            let _ = sqlx::query("CREATE DATABASE foundry_app_test DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;")
                .execute(&root_pool)
                .await;
        }

        let app = match FoundryApp::builder()
            .database_url(&mysql_url)
            .auto_migrate(true)
            .init_tracing(false)
            .register_subsystem(TestSubsystem)
            .build()
            .await
        {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Skipping test - unable to connect to MySQL: {}", e);
                return;
            }
        };

        // 1. Verify health endpoint
        let router = app.router();
        let health_req = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let health_res = router.clone().oneshot(health_req).await.unwrap();
        assert_eq!(health_res.status(), StatusCode::OK);

        // 2. Verify registered subsystem route
        let sub_req = Request::builder()
            .uri("/api/v1/s/demo/ext/ping")
            .body(Body::empty())
            .unwrap();
        let sub_res = router.clone().oneshot(sub_req).await.unwrap();
        assert_eq!(sub_res.status(), StatusCode::OK);
        let body = sub_res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"pong");

        // 3. Verify Database facade methods work against MySQL
        let db = app.db();
        let sys = db
            .systems()
            .create("demo_tenant", "Demo Tenant", None)
            .await
            .expect("Failed to create system via facade in MySQL");
        assert_eq!(sys.slug, "demo_tenant");

        let fetched_sys = db
            .systems()
            .get_by_slug("demo_tenant")
            .await
            .expect("Failed to fetch system via facade in MySQL");
        assert_eq!(fetched_sys.name, "Demo Tenant");
    }
}
