# Database Migrations (Engine: PostgreSQL (14+))

## 1. Migration History & Sequential Upgrade Chain

| Sequence | Version | Name | Description | Migration File |
| :--- | :--- | :--- | :--- | :--- |
| `0001` | `v0.1.0` | `baseline` | Core platform tables (systems, configs, models, fields, records, admins, audit_logs) | `0001_v0.1.0_baseline.sql` |
| `0002` | `v0.2.0` | `add_applications` | Add applications table for client application authentication and scoped API RBAC | `0002_v0.2.0_add_applications.sql` |

## 2. Usage Guide

### A. Fresh Database Initialization (全新数据库初始化)
For new installations, apply `init.sql` or execute:
```bash
foundry migrate
```

### B. Existing Database Upgrade (已有老数据库跨版本升级)
If your existing database is upgrading from an earlier version (e.g. from v0.1.0 to v0.2.0, or jumping directly across multiple versions like 0.1.0 -> 0.5.0):
Apply pending numbered migration files in `versions/` in strict sequence, or execute:
```bash
# 1. Check status and pending migrations:
foundry migrate --status

# 2. Apply all pending migrations sequentially in order:
foundry migrate
```
