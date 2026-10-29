# Database Migrations (Engine: PostgreSQL (14+))

## 1. 迁移版本演进链 (Sequential Upgrade Chain)

| Sequence | Version | Name | Description | Migration File |
| :--- | :--- | :--- | :--- | :--- |
| `0001` | `v0.1.0` | `baseline` | Core platform tables (systems, configs, models, fields, records, admins, audit_logs) | `0001_v0.1.0_baseline.sql` |
| `0002` | `v0.2.0` | `add_applications` | Add applications table for client application authentication and scoped API RBAC | `0002_v0.2.0_add_applications.sql` |

## 2. 生产环境安全与最小权限规范 (Zero DDL & Zero DELETE Security Matrix)

生产环境中，数据库账号必须实行**严格的最小权限原则**：
- **应用服务连接账号 (无需 DDL 权限、无需 DELETE 权限)**：
Foundry 全站严格采用**软删除（Soft-Delete）机制**（基于 `deleted_at` 字段标记），运行时**绝不执行物理 `DELETE FROM` 语句**，更禁止任何运行时 DDL 操作（`CREATE`、`ALTER`、`DROP`）。
因此，生产环境分配给应用的数据库账号**仅需且仅应**授予 `SELECT`, `INSERT`, `UPDATE` 权限。即便应用遭受攻击或注入，底层表结构与物理数据也绝对安全。

```sql
-- PostgreSQL 生产环境最小权限配置 (无需 DDL 权限，无需 DELETE 权限)
CREATE USER foundry_app WITH PASSWORD 'YourSecurePasswordHere';
GRANT CONNECT ON DATABASE foundry TO foundry_app;
GRANT USAGE ON SCHEMA public TO foundry_app;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO foundry_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE ON TABLES TO foundry_app;
```

- **DBA / 升级发布账号**：
仅在版本变更窗口，由 DBA 或 CI/CD 流水线使用具备 DDL 权限的管理账号手动执行版本升级补丁。

## 3. DBA 手动执行与审查升级流程 (Manual Execution & DBA Review)

### A. 全新数据库初始化 (Fresh Database Installation)
全新部署时，由 DBA 手动执行 `init.sql`，或执行：
```bash
# 方式一：DBA 命令行直接导入
psql -h 127.0.0.1 -U postgres -d foundry -f migrations/init.sql

# 方式二：CLI 自动执行
foundry migrate
```

### B. 生产环境跨版本安全升级 (Zero-DDL Upgrade Workflow)
当系统跨版本升级（例如从 `v0.1.0` 升级到 `v0.2.0`，或跳过多版本直接升级到最新版）：

1. **只读查看待升级版本 (无需 DDL 权限)**：
```bash
foundry migrate --status
```
状态列表中标记为 `[ ] PENDING` 的即为当前环境缺失的数据库补丁。

2. **导出或审查待执行的增量 SQL**：
- **方式一（直观查看）**：直接打开 `migrations/versions/` 目录中对应的补丁文件（按序号从小到大顺序，如 `0002_*.sql`）。
- **方式二（动态导出当前数据库未应用的补丁）**：
```bash
foundry migrate --pending-sql > pending_patch.sql
```
- **方式三（离线打印指定版本的增量 SQL）**：
```bash
foundry ddl -d postgres -v v0.2.0
```

3. **DBA 手工执行增量补丁**：
DBA 通过 Navicat、DataGrip、DBeaver 或终端客户端执行补丁文件。
> **自记录与幂等性保障 (Self-recording & Idempotency)**：
> 每个补丁文件末尾均自带向 `_foundry_migrations` 表插入版本记录的 SQL（带防重更新）。
> 因此，**DBA 手动执行完 SQL 的同时，版本元数据已自动入库**，完全无需应用在启动时具有 DDL 权限。

4. **启动验证**：
DBA 执行完毕后，使用 `foundry migrate --status` 或 `foundry migrate --check` 验证。所有项显示 `[✓] applied`，服务即可在 `AUTO_MIGRATE=false` 下秒级安全启动！

## 4. 开发与测试环境自动化 (Local Dev / CI)
本地开发或测试环境中，如账号具备 DDL 权限，可直接一键执行：
```bash
foundry migrate
```
