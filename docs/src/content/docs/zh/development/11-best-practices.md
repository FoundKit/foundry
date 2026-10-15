---
title: 11. 架构最佳实践与生产部署
description: Foundry 生产级架构设计、性能调优、连接池管理、深度分页优化、Redis 缓存策略与容器化部署完全参考。
---

# 11. 架构最佳实践与生产部署

本章内容：
1. 掌握保障高吞吐、强韧性与高可维护性的分层设计规范；
2. 学会调优数据库连接池、联合索引与深度分页性能；
3. 掌握生产环境安全防范与敏感信息管控规范；
4. 熟练运用 Docker 容器化打包与生产部署流程。

---

## 1. 架构分层严谨性规范

为了确保团队协作时代码风格高度一致，请遵循以下开发约定：

* **控制器保持精简 (Thin Controller)**：控制器的职责仅限于 HTTP 协议相关的参数反序列化、格式校验与响应封装。切忌在控制器中直接堆叠复杂业务规则或手写 SQL 逻辑；
* **业务下沉到 Logic/Service 层 (Fat Service)**：所有与计算规则、跨模型调度、事务一致性相关的代码，均应封装在子系统的 `logic/` 独立服务中，不仅利于单元测试，也有利于未来拆分；
* **显式参数校验 (Explicit Validation)**：在进入领域逻辑前，务必调用 `payload.validate()?`，拒绝非法数据污染下层业务；
* **单向依赖原则**：Controller 依赖 Logic 与 DTO，Logic 依赖 ORM 与 Storage，底层绝不反向依赖上层 Web 控制器。

---

## 2. 数据库性能与连接池调优

### 2.1 连接池大小配置
在高并发生产环境下，过小的连接池会导致请求排队阻塞，过大则会耗尽数据库内存。通过 `.env` 环境变量按宿主机硬件规格合理配置：

```bash
# 最大连接数 (推荐计算公式：核心数 * 2 + 磁盘主轴数，通常 20 ~ 50 较为均衡)
DB_MAX_CONNECTIONS=50
# 最小空闲连接数
DB_MIN_CONNECTIONS=10
# 连接最大生命周期 (秒)
DB_MAX_LIFETIME_SECS=1800
```

### 2.2 联合索引最佳左前缀原则
在多系统架构中，任何业务数据表均以 `system_id`（或 `system_slug`）进行逻辑隔离。因此，在定义 `IndexDef` 复合索引时，**第一列必须是 `system_id`**：

```rust
// 推荐：符合最佳左前缀，能同时加速基于租户的各种条件查询
IndexDef::new("idx_orders_sys_status", &["system_id", "deleted_at", "status"])
```

### 2.3 深度分页性能优化
当数据表记录达到百万级以上时，传统的 `LIMIT 20 OFFSET 1000000` 会触发全表遍历。推荐采用**游标分页 (Cursor Pagination)**：

```rust
// 使用上一页最后一条记录的 id 进行游标过滤
OrmSelect::from_table("shop_orders")
    .where_eq("system_id", "shop")
    .where_gt("id", last_seen_id)
    .order_by_asc("id")
    .limit(20)
```

---

## 3. 缓存策略与 Redis 集成

* **读多写少热点数据缓存**：对于全局平台配置、商品静态详情，应优先读取 `state.redis` 缓存。在更新商品后，触发生命周期钩子（`after_update`）主动驱逐对应缓存键；
* **防缓存雪崩**：为缓存键设置随机扰动的 TTL（如 `300s + rand(1..30)s`），避免大批数据在同一瞬间集体过期导致底层数据库被击穿；
* **防缓存穿透**：对于查询结果为空的记录，可以写入简短的空对象占位符（TTL 设置为 30 秒），防止外部恶意请求打爆数据库。

---

## 4. 生产环境安全管控

* **参数化绑定杜绝 SQL 注入**：无论使用 `OrmSelect`、`RecordStore` 还是原生 SQL，严禁使用 `format!` 拼接 SQL 字符串；
* **生产密钥脱敏**：`JWT_SECRET`、数据库密码与第三方私钥一律通过系统环境变量注入，严禁硬编码提交至 Git 仓库；
* **生产环境关闭 `AUTO_MIGRATE`**：
  * 开发测试环境：`AUTO_MIGRATE=true`，实现全自动平滑建表；
  * 企业生产环境：必须配置 `AUTO_MIGRATE=false`，通过 `foundry ddl --db <type>` 导出方言脚本，经 DBA 审计后统一由数据库发布系统（如 Flyway / Liquibase / GitLab CI）执行。

---

## 5. Docker 容器化打包与部署

### 5.1 生产级多阶段构建 Dockerfile

在你的业务应用根目录下创建 `Dockerfile`：

```dockerfile
# 阶段 1: 依赖缓存与编译
FROM rust:1.85-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# 复制依赖与源码
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建发布版本二进制 (开启全量优化)
RUN cargo build --release

# 阶段 2: 极简精简运行镜像
FROM alpine:3.20
WORKDIR /app
RUN apk add --no-cache ca-certificates libgcc

# 从构建器中复制二进制文件
COPY --from=builder /app/target/release/my-app /app/server
COPY .env.example /app/.env

# 暴露服务端口
EXPOSE 8080

CMD ["/app/server"]
```

### 5.2 生产环境 Docker Compose 编排

```yaml
version: '3.8'

services:
  server:
    build: .
    restart: always
    ports:
      - "8080:8080"
    environment:
      - HOST=0.0.0.0
      - PORT=8080
      - DATABASE_TYPE=postgres
      - DATABASE_URL=postgres://postgres:securepassword@db:5432/foundry
      - REDIS_URL=redis://redis:6379
      - AUTO_MIGRATE=false
      - RUST_LOG=info
    depends_on:
      - db
      - redis

  db:
    image: postgres:18-alpine
    restart: always
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=securepassword
      - POSTGRES_DB=foundry
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    restart: always
    volumes:
      - redisdata:/data

volumes:
  pgdata:
  redisdata:
```

### 5.3 Nginx 反向代理配置建议

```nginx
server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket 支持 (用于 Admin 控制台实时刷新)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

---

至此，你已掌握了 Foundry 系统开发的完整全貌！你可以进一步探索 [架构设计蓝图](../../architecture/blueprint/) 或 [专题与进阶指南](../../guides/configuration/)。
