---
title: External Standalone Subsystems
description: Extend Foundry without recompiling the main binary via External Standalone Subsystems, subsystem.json manifests, and runtime hot-discovery.
---

> **Overview**: For modular enterprise platforms and plugin ecosystems, Foundry supports **External Standalone Subsystems**, allowing dynamic discovery and mounting of subsystem directories via `subsystem.json` manifests without recompiling the main Rust binary.

---

## 1. Directory Structure

```text
my-app/
├── Cargo.toml
├── src/main.rs
└── external_systems/
    └── analytics/
        ├── subsystem.json              # Subsystem manifest specification
        └── custom_pages/               # Sandboxed admin views
            └── dashboard.html
```

---

## 2. Manifest Specification (`subsystem.json`)

```json
{
  "slug": "analytics",
  "display_name": "Analytics Dashboard",
  "description": "Real-time traffic and conversion insights",
  "version": "1.0.0",
  "custom_pages": [
    {
      "key": "dashboard",
      "title": "Traffic Monitor",
      "icon": "BarChart3",
      "page_type": "iframe",
      "entry": "/admin/ext/analytics/dashboard.html",
      "required_role": null
    }
  ]
}
```

---

## 3. Dynamic Discovery in `main.rs`

```rust
use foundry::prelude::*;
use foundry_engine::external::load_external_subsystems;
use std::path::PathBuf;

#[foundry::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FoundryConfig::from_env();
    let external_dirs = vec![PathBuf::from("./external_systems")];

    let modules = load_external_subsystems(&external_dirs);
    let mut builder = FoundryApp::builder().config(config);

    for m in modules {
        builder = builder.register_boxed_subsystem(m);
    }

    let app = builder.build().await?;
    app.run().await?;
    Ok(())
}
```
