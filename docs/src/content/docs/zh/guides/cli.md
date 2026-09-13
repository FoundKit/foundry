---
title: CLI 命令行工具指南
description: foundry 与 foundry-cli 开发者命令行工具完整参考手册。
---

# Foundry CLI 命令行工具指南

Foundry 提供了功能完备的开发者命令行工具，可同时通过 `foundry` 或 `foundry-cli` 命令调用。

---

## 📦 安装方式

```bash
# 方式 1: 直接从 GitHub 仓库安装 (测试与开发阶段)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 方式 2: 本地源码编译安装
cargo install --path crates/foundry_cli
```

---

## 🛠️ 命令概览

| 命令 | 说明 |
|---|---|
| `foundry new <name>` | 创建全新的独立业务应用工程 |
| `foundry system new <slug>` | 在当前工程中脚手架生成业务子系统代码骨架 |
| `foundry system list` | 扫描并列出当前工程所有已注册子系统 |
| `foundry migrate` | 执行数据库基准表与业务迁移脚本 |
| `foundry validate` | 校验当前工程与子系统的目录结构与规范完整性 |

---

## 1. 创建独立工程: `foundry new`

创建包含标准三层架构与 Git 依赖的独立 Rust 应用：

```bash
# 标准创建 (默认使用 Git 依赖指向 main 分支)
foundry new my-app

# 本地源码调试创建 (指定 local path 依赖)
foundry new my-app --path /path/to/foundry/crates/foundry

# 指定特定 Git 仓库地址或分支
foundry new my-app --git https://github.com/foundkit/foundry --branch feature/next

# 指定特定 crates.io 版本
foundry new my-app --version 0.1.0
```

> **本地开发资源目录**：新工程预置了 `dev/docker-compose.yml` 配置（已被 `.gitignore` 忽略以隔离本地环境），支持一键启动本地专用的 PostgreSQL 18 与 Redis 7：
> ```bash
> cd my-app
> docker compose -f dev/docker-compose.yml up -d
> ```

---

## 2. 业务子系统管理: `foundry system`

### 生成业务子系统骨架: `foundry system new`

在当前工程的 `src/systems/<slug>/` 下快速生成业务子系统代码骨架：

```bash
foundry system new billing --name "账单与支付中心"
```

自动生成的文件结构包括：
* `src/systems/billing/mod.rs`（实现 `SubsystemModule` 特征）
* `src/systems/billing/controllers/mod.rs`（Axum HTTP 路由控制器）
* `src/systems/billing/logic/mod.rs`（纯领域业务逻辑与数据库交互）
* `src/systems/billing/dto/mod.rs`（请求入参与 validator 规则校验）
* `src/systems/billing/custom_pages/`（嵌入管理后台的专属运营看板）

### 查看已注册子系统: `foundry system list`

扫描并列出工程内所有代码内聚子系统：

```bash
foundry system list
```

---

## 3. 数据库迁移: `foundry migrate`

手动对 PostgreSQL 数据库执行基准迁移与项目级 `migrations/` 脚本：

```bash
foundry migrate --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

---

## 4. 项目校验: `foundry validate`

检测当前工程结构与已注册子系统元数据是否规范：

```bash
foundry validate
```

---

## 5. 后台管理员与权限管控说明

Foundry 遵循职责分离的最佳实践原则：
* **CLI 聚焦研发工具链**：专注于项目创建、代码骨架生成、数据库迁移与工程校验。
* **Web 后台负责运行期运维**：管理员账号的增删、角色分配（`super_admin` / `admin` / `topic_admin`）以及密码重置已全面内置在开箱即用的 React 管理后台（`/admin`）中。
* 系统初始化时会自动创建默认初始超管（`admin / admin123456`），后续运维直接登录后台进行可视化安全管理，无需通过终端明文传递敏感凭据。
