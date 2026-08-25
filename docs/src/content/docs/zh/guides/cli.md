---
title: CLI 命令行工具指南
description: foundry 与 foundry-cli 开发者命令行工具完整参考手册。
---

# Foundry CLI 命令行工具指南

Foundry 提供了功能完备的开发者命令行工具，可同时通过 `foundry` 或 `foundry-cli` 命令调用。

---

## 📦 安装方式

```bash
# 直接从 GitHub 仓库安装
cargo install --git https://github.com/foundkit/foundry foundry_cli
```

---

## 🛠️ 命令概览

| 命令 | 说明 |
|---|---|
| `foundry new <name>` | 创建全新的独立业务应用工程 |
| `foundry system new <slug>` | 在当前工程中脚手架新的业务子系统 |
| `foundry migrate` | 执行数据库基准表与业务迁移 |
| `foundry admin create` | 创建后台管理员账号 |
| `foundry admin reset-password` | 重置后台管理员密码 |
| `foundry validate` | 校验当前工程与子系统的目录结构与元数据有效性 |

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
```

---

## 2. 脚手架子系统: `foundry system new`

在当前工程的 `src/systems/<slug>/` 下快速生成业务子系统代码骨架：

```bash
foundry system new billing --name "账单与支付中心"
```

自动生成的文件结构包括：
* `src/systems/billing/mod.rs`（实现 `SubsystemModule` 特征）
* `src/systems/billing/controllers/mod.rs`（Axum HTTP 路由）
* `src/systems/billing/logic/mod.rs`（纯领域业务逻辑）
* `src/systems/billing/dto/mod.rs`（请求参数与校验）
* `src/systems/billing/custom_pages/`（自定义 Admin 运营看板）

---

## 3. 数据库迁移: `foundry migrate`

手动对 PostgreSQL 数据库执行迁移：

```bash
foundry migrate --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

---

## 4. 管理员管理: `foundry admin`

### 创建管理员账户：
```bash
foundry admin create \
  --username admin \
  --password mysecretpassword \
  --role super_admin \
  --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

#### 可用角色说明：
* `super_admin`: 全局超级管理员，拥有管理平台所有子系统与管理员账户的最高权限。
* `admin`: 平台管理员。
* `topic_admin`: 专属专题管理员，仅被允许访问其关联分配的特定业务子系统。

### 重置管理员密码：
```bash
foundry admin reset-password \
  --username admin \
  --new-password newsecretpassword
```

---

## 5. 项目校验: `foundry validate`

检测当前工程结构与已注册子系统元数据是否规范：

```bash
foundry validate
```
