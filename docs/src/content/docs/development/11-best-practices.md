---
title: 11. Architectural Best Practices & Production
description: Production-grade architecture rules, connection pool tuning, cursor pagination, Redis caching, and Docker containerization deployment.
---

# Architectural Best Practices & Production

This chapter covers:
1. Adhere to Thin Controller / Fat Service conventions;
2. Tune database connection pool sizes, index prefixes, and cursor pagination;
3. Enforce production security policies and secrets hygiene;
4. Deploy Foundry applications with multi-stage Dockerfiles and Docker Compose.

---

## 1. Architectural Rules

* **Thin Controller / Fat Service**: Keep handlers restricted to deserialization, validation, and serialization. Keep domain rules inside `logic/`.
* **Explicit Validation**: Always call `payload.validate()?` before business logic execution.
* **Compound Indexes**: Always make `system_id` the first column in multi-tenant composite indexes (`IndexDef`).
* **Cursor Pagination**: For large tables, avoid deep `OFFSET`. Use `WHERE id > $last_id LIMIT $n`.

---

## 2. Docker Containerization

### Multi-Stage Dockerfile

```dockerfile
FROM rust:1.85-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.20
WORKDIR /app
RUN apk add --no-cache ca-certificates libgcc
COPY --from=builder /app/target/release/my-app /app/server
COPY .env.example /app/.env
EXPOSE 8080
CMD ["/app/server"]
```

---

## 3. Production Migration Strategy

* **Development & CI**: `AUTO_MIGRATE=true` for zero-friction schema creation.
* **Production**: `AUTO_MIGRATE=false`. Export offline migration scripts via `foundry ddl --db <type>` into `migrations/init.sql` for DBA auditing and controlled deployment.
