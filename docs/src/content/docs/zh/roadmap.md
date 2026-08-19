---
title: 开发路线图与任务清单
description: Foundry 平台开发里程碑、已完成功能与后续规划。
---

# Foundry 开发路线图与任务清单

> **项目**: `foundry`  
> **组织**: [foundkit](https://github.com/foundkit)  
> **仓库**: Monorepo (Rust 引擎 + 子系统工作区 + Web 管理后台 SPA + CLI 工具)

---

## 📌 状态图例
- [ ] **待办 (Pending)**
- [/] **进行中 (In Progress)**
- [x] **已完成 (Completed)**

---

## 里程碑 0: Monorepo 基座与开发环境初始化

- [x] **0.1 仓库与项目脚手架搭建**
  - [x] 初始化 Rust 后端 Crates 与 `systems/` 工作区的 Cargo Workspace (`Cargo.toml`)。
  - [x] 应用来自 `migrations/init.sql` 的基线数据库表结构（Zero-DDL 表：`systems`、`system_configs`、`models`、`model_fields`、`model_records`、`admins`、`audit_logs`）。
  - [x] 在 `apps/admin` 初始化管理后台 SPA 项目（React + Vite + TypeScript + Tailwind CSS）。
  - [x] 配置 GitHub Actions CI 与代码风格检查。
  - [x] 配置 `docker/` 容器编排（PostgreSQL 18.6+ 与 Redis 8+ 的 `docker-compose.yml`）。

- [x] **0.2 核心目录规划**
  - [x] 创建 `apps/server`（Rust Axum 二进制服务入口）。
  - [x] 创建 `apps/admin`（Vite + React + Tailwind CSS 管理后台 SPA）。
  - [x] 创建 `apps/cli`（用于子系统脚手架和迁移的 CLI 命令行工具）。
  - [x] 创建 `systems/`（子系统定制代码工作区）。
  - [x] 创建核心 Rust Crates（`crates/foundry_core`、`crates/foundry_storage`、`crates/foundry_auth`、`crates/foundry_engine`、`crates/foundry_extension`）。

---

## 里程碑 1: 核心引擎与多系统隔离 (后端 MVP)

- [x] **1.1 系统上下文与租户路由 (`crates/foundry_core` & `crates/foundry_engine`)**
  - [x] 实现子系统元数据原语：`system_id` (UUID) 与不可变的唯一 `system_slug`。
  - [x] 构建 Axum 中间件，支持通过 URL 路径 (`/api/v1/s/{system_slug}/...`)、Header (`X-Foundry-System-ID`) 提取并校验 `SystemContext`。
  - [x] 解析 `Accept-Language` 请求头为 `SystemContext.locale`，用于多语言国际化响应。
  - [x] 实现全局错误处理与 i18n JSON 响应包（`code`, `message`, `i18n_key`, `args`）。

- [x] **1.2 Zero-DDL 动态存储引擎与模型运行时 (`crates/foundry_storage`)**
  - [x] 实现系统配置引擎 (`system_configs`)：单行属性管理，可视化表单类型，自动聚合为单一 JSON 对象。
  - [x] 实现动态数据模型引擎 (`models` & `model_fields`)：Schema 元数据，字段类型定义。
  - [x] 构建基于 `model_records` 的动态数据模型运行时 ORM（`ctx.model("{model_slug}")`），支持分页、GIN 索引过滤、排序与原子变更。
  - [x] 实现声明式 Configs API（`ctx.configs()`），方便在自定义逻辑中获取类型化配置。
  - [x] 实现内存级数据验证（类型、范围、正则、必填字段）。
  - [x] 集成 Redis 连接池，严格进行租户命名空间隔离（`foundry:{system_slug}:*`）。

- [x] **1.3 Auto-CRUD 与系统配置 API 引擎 (`crates/foundry_engine`)**
  - [x] 实现动态模型 Auto-CRUD 路由：`GET`（列表查询/过滤/分页）、`POST`、`GET /:id`、`PUT/PATCH /:id`、`DELETE /:id`。
  - [x] 实现系统配置路由：`GET /api/v1/s/{slug}/configs` 与 `PUT /api/v1/s/{slug}/configs`。
  - [x] 构建细粒度接口公开控制开关（公开读取、鉴权读取、公开写入、禁用）。

- [x] **1.4 管理员 IAM 与 Topic 范围 RBAC (`crates/foundry_auth`)**
  - [x] 实现管理员身份模型与 Argon2id 密码哈希。
  - [x] 初始化默认超级管理员 (`super_admin`)，具备全局通配权限 (`allowed_systems: ["*"]`)。
  - [x] 实现超级管理员创建/管理普通管理员 (`admin`) 并分配指定主题权限。
  - [x] JWT Token 签发、校验与刷新机制。
  - [x] Axum 权限判定中间件。

---

## 后续规划

- [/] **Wasmtime 动态沙箱插件支持**
- [/] **OpenAPI 3.0 规范实时聚合与 Swagger UI**
- [ ] **高可用多节点集群调度**
