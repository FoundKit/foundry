---
title: 快速入门
description: Foundry 快速上手与本地开发指南。
---

# Foundry 快速入门

Foundry 是一个基于 Rust、PostgreSQL、Redis 与 React 构建的高性能开源多子系统后端平台。

## 环境要求

在开始前，请确保本地已安装以下环境：

- **Rust 1.85+ / 2024 edition**
- **Node.js 20+** 与 **pnpm 9+**
- **Docker 与 Docker Compose**（用于本地 PostgreSQL 与 Redis）

---

## 1. 启动本地基础设施

使用 Docker Compose 启动本地 PostgreSQL 与 Redis：

```bash
docker compose -f docker/docker-compose.yml up -d
```

服务端口：
- PostgreSQL: `localhost:5432`（数据库名: `foundry`, 用户: `postgres`, 密码: `password`）
- Redis: `localhost:6379`

---

## 2. 初始化数据库

执行基线数据表结构初始化：

```bash
psql -h localhost -U postgres -d foundry -f migrations/init.sql
```

---

## 3. 启动 Foundry 后端服务

启动 Axum 后端核心服务：

```bash
cargo run --bin foundry-server
```

服务默认监听在 `http://127.0.0.1:8080`。

---

## 4. 启动管理后台 SPA (Admin UI)

进入 `apps/admin` 目录并启动 Vite 开发服务器：

```bash
pnpm --dir apps/admin install
pnpm --dir apps/admin dev
```

在浏览器中打开 `http://localhost:5173`。默认超级管理员账号：

- **用户名**: `admin`
- **密码**: `admin123456`

---

## 5. 脚手架创建子系统

使用 `foundry-cli` 快速创建 Monorepo 编译型子系统：

```bash
cargo run --bin foundry-cli -- system new carnival_demo --name "Carnival Demo"
```

或者创建独立的外部仓库子系统：

```bash
cargo run --bin foundry-cli -- system new-external vip_mall --name "VIP Mall Standalone"
```

---

## 后续阅读

- 阅读 [架构蓝图设计](/zh/architecture/blueprint/) 深入了解 Foundry 核心原理。
- 阅读 [子系统与扩展开发](/zh/guides/extensions/) 学习如何编写自定义 Rust API 与管理后台页面。
- 查看 [开发路线图](/zh/roadmap/) 了解功能排期与进度。
