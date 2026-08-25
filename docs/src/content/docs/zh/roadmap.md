---
title: 开发路线图与版本策略
description: Foundry 平台与框架的开发里程碑、已实现特性、版本策略与后续规划。
---

# Foundry 路线图与版本管理策略

> **项目**: `foundry`  
> **组织**: [foundkit](https://github.com/foundkit)  

---

## 📌 状态图例
- [ ] **待开始 (Pending)**
- [/] **进行中 (In Progress)**
- [x] **已完成 (Completed)**

---

## 里程碑 1: 框架解耦与门面架构 (`v0.1.0`)

- [x] **1.1 平台框架与业务应用完全解耦**
  - [x] 创建高层门面 crate `foundry`，提供 `FoundryApp::builder()` 与统一 prelude。
  - [x] 彻底隔离框架运行时与独立用户业务工程。
  - [x] 通过 Cargo dependency (`foundry = "0.1"`) 声明式引入。

- [x] **1.2 子系统标准与动态路由体系**
  - [x] `SubsystemModule` 特征用于注册业务模块、自定义路由与 Admin 运营大屏。
  - [x] 静态编译与动态外部子系统发现引擎。
  - [x] 专题隔离 RBAC 校验与 `SystemContext` 租户上下文注入。

- [x] **1.3 存储与 Admin 控制台**
  - [x] PostgreSQL Zero-DDL 动态数据模型存储引擎。
  - [x] 自动生成高可用 RESTful CRUD 接口 (`/api/v1/s/{system_slug}/{model_slug}`)。
  - [x] 内嵌 React Admin SPA 外壳与沙箱通信 Bridge (`FoundryBridge`)。

- [x] **1.4 开发者 CLI 工具 (`foundry-cli`)**
  - [x] `foundry new <project>`: 脚手架独立用户业务工程。
  - [x] `foundry system new <slug>`: 快速创建业务子系统。
  - [x] `foundry migrate`: 执行数据库初始化迁移。
  - [x] `foundry admin create` / `reset-password`: 管理员 IAM 工具。

- [x] **1.5 完整参考示例与自动化测试**
  - [x] `examples/blog_platform`: 真实场景独立业务应用范例。
  - [x] CI 端到端集成测试与 CLI Smoke 自动化验证。

---

## 里程碑 2: 运行时扩展与开发者体验 (`v0.2.0`)

- [/] **2.1 WebAssembly (Wasmtime) 沙箱插件运行时**
  - [ ] 支持动态热加载 WASM 子系统扩展。
  - [ ] 为多租户自定义逻辑提供内存安全隔离。

- [/] **2.2 OpenAPI 3.0 与 Swagger UI 深度集成**
  - [ ] 自动聚合动态数据模型与子系统接口的 OpenAPI Schema。
  - [ ] Admin 外壳内嵌实时交互式 API 文档。

- [ ] **2.3 多数据库驱动扩展 (MySQL, SQLite)**
  - [ ] 支持可插拔的 Zero-DDL 动态存储 SQL 方言适配器。

---

## 里程碑 3: 企业级高可用与集群化 (`v1.0.0`)

- [ ] **3.1 集群编排与分布式 Pub/Sub**
  - [ ] 实时 WebSocket 事件广播。
  - [ ] 基于 Redis Pub/Sub 的多节点缓存同步。

- [ ] **3.2 动态 Webhook 事件分发器**
  - [ ] 在模型写操作时触发自定义 Webhook。
  - [ ] 支持指数退避重试队列。

---

## 🏷️ 版本管理与升级指南

Foundry 严格遵循 **语义化版本规范 (SemVer)**：
- **主版本 (`1.x.x`)**: 公共 API 的重大重构与 Breaking Changes，附带自动化迁移指引。
- **次版本 (`0.x.x`, `1.x.0`)**: 新功能发布、构造器选项扩充、向后兼容的特征默认实现。
- **补丁版本 (`0.1.x`)**: 向后兼容的缺陷修复与性能提升。

在用户工程中升级：
```bash
cargo update -p foundry
```
