# Blog Platform Example (Foundry Framework)

This is a complete, standalone example application built using the **Foundry Backend Platform & Framework**.

## Architecture

```
blog_platform/
├── Cargo.toml                # Depends on `foundry` crate
├── src/
│   ├── main.rs               # Application bootstrap with FoundryApp::builder()
│   ├── lib.rs                # Library entry for subsystem & hook exports
│   ├── hooks.rs              # Custom mutation lifecycle hooks
│   └── systems/
│       ├── blog/             # Blog & Content Management Subsystem
│       │   ├── controllers/  # Axum HTTP handlers mounted at /api/v1/s/blog/ext/*
│       │   ├── logic/        # Domain business services
│       │   ├── dto/          # DTO schemas & field validators
│       │   └── custom_pages/ # Custom Admin UI Extension Studio
│       └── newsletter/       # Audience & Newsletter Subsystem
│           ├── controllers/  # Mounted at /api/v1/s/newsletter/ext/*
│           ├── logic/
│           ├── dto/
│           └── custom_pages/
└── tests/
    └── integration_test.rs   # Comprehensive end-to-end integration tests
```

## Running the Application

1. Start database:
```bash
docker compose up -d
```

2. Run the application:
```bash
cargo run
```

3. Run integration tests:
```bash
cargo test
```
