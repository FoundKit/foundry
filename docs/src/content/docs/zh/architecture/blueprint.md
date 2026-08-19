---
title: 架构设计蓝图
description: Foundry 多系统多租户后端平台完整架构设计与蓝图。
---

# Foundry 架构蓝图设计

> **组织**: [foundkit](https://github.com/foundkit)  
> **项目**: `foundry`  
> **核心愿景**: *基于统一基座构建与运行独立的多系统后端。*  
> **仓库描述**: *Foundry 是一个用于在共享基座上构建和运行多个独立后端系统与管理后台的开源基础设施平台。*

---

## 1. 产品定位与愿景

### 1.1 什么是 Foundry？
**Foundry** 是一个完整、自包含的开源 **多系统后端平台与管理系统**（Backend-as-a-Service / 多租户基础设施引擎）。

与传统的单项目 BaaS 方案（如 Strapi, Directus, PocketBase 或 Supabase）不同，**Foundry 从一开始就以 Monorepo 架构为核心，支持在单一统一基座上管理多个完全独立的业务子系统。**

```
+-------------------------------------------------------------------------------------------------------+
|                                        FOUNDRY 平台生态架构                                           |
|                                                                                                       |
|  +--------------------------------+  +-------------------------------------------------------------+  |
|  |     Foundry Admin UI (SPA)     |  |         客户端调用层 (REST-First & OpenAPI-Native)          |  |
|  |  - 可视化系统构建器            |  |  - 标准 HTTP 客户端 (Fetch, Axios, Ktor, cURL 等)           |  |
|  |  - Zero-DDL 动态数据模型       |  |  - 基于 OpenAPI 3.0 规范生成的类型安全客户端                |  |
|  |  - 管理员与主题权限 RBAC       |  |  - 子系统自定义业务 API                                     |  |
|  |  - 非 GET 操作审计日志面板     |  |                                                             |  |
|  +---------------+----------------+  +------------------------------+------------------------------+  |
|                  |                                                  |                                 |
|                  +------------------------+-------------------------+                                 |
|                                           | 标准 RESTful / OpenAPI (Utoipa)                           |
|                                           v                                                           |
|  +-------------------------------------------------------------------------------------------------+  |
|  |                              Foundry 服务端引擎 (Rust Monorepo)                                 |  |
|  |                                                                                                 |  |
|  |  +--------------------+  +--------------------+  +--------------------+  +-------------------+  |  |
|  |  | 子系统 Alpha       |  | 子系统 Beta        |  | 子系统 Gamma       |  | 自定义后端 N      |  |  |
|  |  | (参数配置, 数据模型|  | (参数配置, 数据模型|  | (自定义逻辑, DTO,  |  | (自包含业务逻辑)  |  |  |
|  |  |  Auto-CRUD 接口)   |  |  Auto-CRUD 接口)   |  |  领域模型)         |  |                   |  |  |
|  |  +--------------------+  +--------------------+  +--------------------+  +-------------------+  |  |
|  |                                                                                                 |  |
|  |  +-------------------------------------------------------------------------------------------+  |  |
|  |  |                               平台核心基础设施与通用服务                                  |  |  |
|  |  |  - 多租户路由器 (`/api/v1/s/:slug`)         - Zero-DDL 动态存储引擎 (Postgres JSONB/GIN) |  |  |
|  |  |  - 管理员 IAM 与 Topic 范围 RBAC           - 非 GET 写入操作审计拦截器                   |  |  |
|  |  |  - Wasmtime 沙箱与 Trait 扩展钩子          - 编译期 OpenAPI 3.0 规范聚合器               |  |  |
|  |  +-------------------------------------------------------------------------------------------+  |  |
|  +-------------------------------------------------------------------------------------------------+  |
+-------------------------------------------------------------------------------------------------------+
```

### 1.2 核心价值
1. **多系统隔离引擎**：在单个 Foundry 实例中同时运行与管理多个独立业务子系统，保持严格的租户与数据隔离。
2. **可视化一键系统与模型生成 (Admin UI + Auto-CRUD)**：现代化管理面板，图形化创建子系统、定义动态数据模型，并自动生成高性能 RESTful API。
3. **Code-First 自定义接口与业务逻辑扩展**：支持为子系统编写原生 Rust 控制器、业务服务层与 DTO 校验层，与 Axum 路由无缝整合。
4. **纯粹的基础设施基座与子系统自治**：Foundry 保持中立与通用，不绑定多余公共业务模块，各子系统拥有自主的数据模型与业务体系。
5. **分级管理员 IAM 与 Topic 范围 RBAC**：内置超级管理员（`super_admin`）与针对具体子系统授权的主题管理员（`admin`）。
6. **非 GET 写操作全量审计追踪**：自动记录所有非 GET 操作（POST、PUT、PATCH、DELETE、登录）的操作人、子系统、路径、负载、IP 及耗时。
7. **混合扩展引擎 (Rust Traits + Wasmtime 沙箱)**：支持原生 Rust Trait 极致性能与 Wasm 插件动态热加载。
8. **REST-First 与原生 OpenAPI 规范**：无客户端厂商锁定，自动生成 OpenAPI 3.0 规范。

---

## 2. 仓库目录结构

```
foundry/                         # Monorepo 根目录
├── README.md                    # 项目概述与快速指南
├── docs/                        # Astro Starlight 文档站源码
│   ├── astro.config.mjs         # Starlight 配置文件
│   ├── package.json             # 文档依赖与构建脚本
│   └── src/content/docs/        # 多语言 Markdown 文档内容
├── migrations/                  # 数据库初始化与迁移
│   └── init.sql                 # 基线 PostgreSQL 表结构 (Zero-DDL 表)
├── apps/                        # 可执行应用程序
│   ├── server/                  # Foundry Core Server (Rust / Axum 后端服务入口)
│   ├── admin/                   # Visual Web Admin Dashboard SPA (React / Vite / Tailwind)
│   └── cli/                     # 开发者 CLI 工具 (子系统脚手架、代码生成)
├── systems/                     # 子系统编译型代码工作区
│   ├── Cargo.toml               # 子系统 Workspace 定义
│   └── src/                     # 注册中心与各子系统模块
├── external_systems/            # 独立子系统仓库 (解耦托管)
├── crates/                      # 核心模块 Crates (Rust Workspace)
│   ├── foundry_core/            # SystemContext, SubsystemModule, CustomAdminPageSpec
│   ├── foundry_storage/         # 动态模型存储引擎、ORM 抽象 (SQLx)
│   ├── foundry_auth/            # 管理员 IAM、Argon2id、JWT 与 Topic RBAC
│   ├── foundry_engine/          # 多系统路由、Auto-CRUD 生成器、审计拦截器
│   └── foundry_extension/       # 变更钩子与 WASM 扩展流水线
└── docker/                      # 容器化部署
    ├── Dockerfile               # 生产环境多阶段构建
    └── docker-compose.yml       # 本地开发环境编排
```

---

## 3. 多租户子系统路由与唯一标识标准

每个子系统拥有两个核心标识：

1. **`system_id` (内部不可变 UUID)**：数据库主键与底层关联字段。
2. **`system_slug` (人类可读代码与 URL 标识)**：小写字母、数字与下划线（如 `carnival_demo`），用于：
   - 接口前缀：`/api/v1/s/{system_slug}/*` 与 `/api/v1/s/{system_slug}/ext/*`
   - Redis 缓存隔离前缀：`foundry:{system_slug}:*`
   - 代码目录映射与 OpenAPI 分组

```
                        [ 客户端请求 ]
                              │
                  ┌───────────┴───────────┐
                  ▼                       ▼
        Path: /api/v1/s/:system_slug/...  Header: X-Foundry-System-ID
                  │                       │
                  └───────────┬───────────┘
                              ▼
                  [ Axum SystemContext ]
                              │
  ┌───────────────────────────┼───────────────────────────┐
  ▼                           ▼                           ▼
[ Auto-CRUD 动态接口 ]    [ 自定义扩展 API ]     [ 权限守卫 RBAC ]
```
