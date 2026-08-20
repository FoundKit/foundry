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

---

## 里程碑 1: 核心引擎与多系统隔离 (后端 MVP)

- [x] **1.1 系统上下文与租户路由 (`crates/foundry_core` & `crates/foundry_engine`)**
  - [x] 实现子系统元数据原语：`system_id` (UUID) 与不可变的唯一 `system_slug`。
  - [x] 构建 Axum 中间件，支持通过 URL 路径 (`/api/v1/s/{system_slug}/...`)、Header (`X-Foundry-System-ID`) 提取并校验 `SystemContext`。
  - [x] 解析 `Accept-Language` 请求头为 `SystemContext.locale`，用于多语言国际化响应。
  - [x] 实现全局错误处理与 i18n JSON 响应包。

- [x] **1.2 Zero-DDL 动态存储引擎与模型运行时 (`crates/foundry_storage`)**
  - [x] 实现系统配置引擎 (`system_configs`)：单行属性管理，可视化表单类型，自动聚合为单一 JSON 对象。
  - [x] 实现动态数据模型引擎 (`models` & `model_fields`)：Schema 元数据，字段类型定义。
  - [x] 构建基于 `model_records` 的动态数据模型运行时 ORM（`ctx.model("{model_slug}")`），支持分页、GIN 索引过滤、排序与原子变更。
  - [x] 实现声明式 Configs API（`ctx.configs()`），方便在自定义逻辑中获取类型化配置。
  - [x] 集成 Redis 连接池，严格进行租户命名空间隔离（`foundry:{system_slug}:*`）。

- [x] **1.3 Auto-CRUD 与系统配置 API 引擎 (`crates/foundry_engine`)**
  - [x] 实现动态模型 Auto-CRUD 路由：`GET`、`POST`、`GET /:id`、`PUT/PATCH /:id`、`DELETE /:id`。
  - [x] 实现系统配置路由：`GET /api/v1/s/{slug}/configs` 与 `PUT /api/v1/s/{slug}/configs`。

- [x] **1.4 管理员 IAM 与 Topic 范围 RBAC (`crates/foundry_auth`)**
  - [x] 实现管理员身份模型与 Argon2id 密码哈希。
  - [x] 初始化默认超级管理员 (`super_admin`)，具备全局通配权限 (`allowed_systems: ["*"]`)。
  - [x] 实现超级管理员创建/管理普通管理员 (`admin`) 并分配指定主题权限。
  - [x] JWT Token 签发、校验与刷新机制。

---

## 里程碑 2: Web Admin 管理控制中台 (`apps/admin`)

- [x] **2.1 现代化 UI 与国际化 (i18n)**
  - [x] React + Vite + Tailwind CSS 设计系统。
  - [x] 中英文多语言即时切换 (`react-i18next`)。
  - [x] 登录与 Token 持久化。

- [x] **2.2 平台总控中台与子系统工作台解耦**
  - [x] 平台控制层：总览大屏、多维度子系统搜索管理、管理员 IAM 权限分配、全局写操作审计日志。
  - [x] 子系统独立工作台：参数配置、动态数据模型设计、数据表浏览器、API 目录。
  - [x] 双向 URL 路由持久化与状态恢复。

---

## 里程碑 3: 非 GET 写操作全量审计拦截

- [x] **3.1 审计中间件与离散存储 (`crates/foundry_engine`)**
  - [x] 自动拦截非 GET 请求（POST, PUT, PATCH, DELETE, login）。
  - [x] 原始参数存储：`headers` (JSONB)、`query_params` (原始查询字符串)、`body_params` (原始负载文本)。
  - [x] 动态操作名称解析器。

---

## 里程碑 4: 子系统与基建完全解耦架构 & 统一发布流水线

- [x] **4.1 双代码仓库解耦架构 (Two-Repository Pattern)**
  - [x] 平台基建仓库与子系统自定义代码彻底解耦，支持子系统独立存放于私有 Git 仓库。
  - [x] 自包含子系统标准：每个子系统目录下聚合控制器、逻辑服务、DTO、自定义后台页面 (`custom_pages`) 与清单配置 (`subsystem.json`)。
  - [x] 消除上游基座更新时的代码冲突，支持 `git pull upstream` 平滑升级。

- [x] **4.2 子系统自定义管理后台 UI 与 SDK Bridge**
  - [x] 支持静态 HTML/JS/CSS 及 React 打包产物作为自定义管理后台页面。
  - [x] `FoundryBridge` 跨 iframe 通信机制，自动注入当前管理员身份、Token 及主题模式。
  - [x] 角色权限控制 (`required_role`)。

- [x] **4.3 CLI 脚手架工具升级 (`apps/cli`)**
  - [x] `foundry-cli system init-repo`: 一键初始化独立子系统 Git 仓库模板。
  - [x] `foundry-cli system new`: 脚手架创建自包含子系统。
  - [x] `foundry-cli system validate`: 校验子系统目录与清单文件完整性。

- [x] **4.4 统一构建与发布打包流水线 (`scripts/build-release.sh` & Dockerfile)**
  - [x] 一键合并基建与自定义子系统产物。
  - [x] 产出单体 Release 二进制部署包与多阶段轻量 Docker 镜像。

---

## 🔮 后续规划

- [/] **Wasmtime 动态沙箱插件支持**
- [/] **OpenAPI 3.0 规范实时聚合与 Swagger UI**
- [ ] **高可用多节点集群调度**
