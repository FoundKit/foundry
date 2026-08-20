---
title: 子系统与扩展开发
description: 使用自定义后端 API、管理后台页面与单一自定义代码仓库扩展 Foundry。
---

# 子系统扩展与单一自定义代码仓库开发指南

Foundry 提供了 **“平台核心基建 + 统一自定义代码仓库”** 的扩展体系。团队或个人所有的业务子系统（所有自定义接口 + 业务逻辑 + 自定义 Admin 页面 + 清单配置）统一存放在 **一个单独的 Git 仓库** 中，每个子系统是该仓库下的独立目录。

---

## 1. 核心架构：为什么所有子系统放在同一个自定义代码仓库？

```
[基建开源仓库 foundry] ───(Git Submodule / 目录挂载)───> [统一自定义仓库 foundry-systems]
         │                                                            │
    (持续 pull 基座更新)                                     (存放团队所有的子系统)
   100% 升级 0 冲突!                                         • carnival_demo/
         │                                                   • vip_mall/
         │                                                   • order_center/
         │                                                   • payment_gateway/
         │                                                            │
         └─────────────────> 统一发布打包流水线 <──────────────────────┘
                                         │
                             (编译为一个生产产物/镜像)
```

- **维护成本最低**：使用者只需要维护自己的 **一个** 业务代码仓库，无需为每个子系统单独建仓。
- **基建升级 0 冲突**：所有的业务定制代码 100% 独立于 `foundry` 基建仓库。当 Foundry 官方发布新版本或安全修复时，直接在基建仓库拉取更新（`git pull upstream main`），绝不产生任何代码冲突。
- **发布打包一键合并**：在打包阶段，执行构建脚本即可将基座引擎与该自定义仓库中的所有子系统合并编译为一个统一可执行文件或 Docker 镜像。

---

## 2. 自包含子系统目录规范 (Self-Contained Standard)

在自定义代码仓库中，每个子系统目录（如 `carnival_demo/`、`vip_mall/`）完全自包含自身的所有定制资产：

```
foundry-systems/                     # 统一自定义代码仓库根目录
├── carnival_demo/                   # 子系统 A (目录完全自包含)
│   ├── mod.rs                       # [入口] 实现 SubsystemModule Trait，挂载路由与静态资源
│   ├── subsystem.json               # [清单] 声明子系统元数据与自定义后台大屏
│   ├── controllers/                 # [表现层] Axum HTTP 控制器 (挂载在 /api/v1/s/:slug/ext/*)
│   │   ├── mod.rs                   # 导出路由树
│   │   └── draw_controller.rs       # 处理函数与参数提取
│   ├── logic/                       # [领域层] 纯业务逻辑、服务与事务处理
│   │   ├── mod.rs                   # 导出领域服务
│   │   └── draw_service.rs          # 核心业务规则与持久化调用 (ctx.model / ctx.configs)
│   ├── dto/                         # [契约层] 请求/响应 DTO 与参数校验约束
│   │   ├── mod.rs                   # 导出 DTOs
│   │   └── draw_dto.rs              # 声明式参数校验 (validator)
│   └── custom_pages/                # [视图层] 自定义管理后台专属页面 (HTML/JS/CSS)
│       ├── lottery_dashboard.html   # 抽奖大屏可视化
│       └── wheel_control.html       # 专属调控面板
├── vip_mall/                        # 子系统 B
│   └── ...
└── order_center/                    # 子系统 C
    └── ...
```

---

## 3. 编写子系统自定义业务 (三层架构)

### 步骤 1：定义 DTO 与校验 (`dto/draw_dto.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DrawRequest {
    #[validate(length(min = 1, message = "用户 ID 不能为空"))]
    pub user_id: String,
    #[validate(range(min = 1, max = 100, message = "幸运数字必须在 1-100 之间"))]
    pub lucky_number: u32,
}

#[derive(Debug, Serialize)]
pub struct DrawResponse {
    pub prize_name: String,
    pub is_winner: bool,
}
```

### 步骤 2：实现领域业务服务 (`logic/draw_service.rs`)

```rust
use crate::carnival_demo::dto::{DrawRequest, DrawResponse};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;

pub struct DrawService;

impl DrawService {
    pub async fn execute(ctx: &SystemContext, req: DrawRequest) -> AppResult<DrawResponse> {
        let configs = ctx.configs();
        let is_winner = req.lucky_number == 14;
        let prize_name = if is_winner { "一等奖" } else { "感谢参与" };

        Ok(DrawResponse {
            prize_name: prize_name.to_string(),
            is_winner,
        })
    }
}
```

### 步骤 3：编写 HTTP 控制器 (`controllers/draw_controller.rs`)

```rust
use axum::{extract::Extension, Json};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::ApiResponse;
use validator::Validate;
use crate::carnival_demo::dto::{DrawRequest, DrawResponse};
use crate::carnival_demo::logic::DrawService;

pub async fn handle_draw(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<DrawRequest>,
) -> AppResult<Json<ApiResponse<DrawResponse>>> {
    payload.validate()?;
    let result = DrawService::execute(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

### 步骤 4：挂载路由与静态后台页面 (`mod.rs`)

```rust
pub mod controllers;
pub mod dto;
pub mod logic;

use std::path::PathBuf;
use axum::Router;
use foundry_core::SubsystemModule;
use tower_http::services::ServeDir;

pub struct CarnivalDemoModule;

impl SubsystemModule for CarnivalDemoModule {
    fn slug(&self) -> &'static str {
        "carnival_demo"
    }

    fn display_name(&self) -> &'static str {
        "Carnival 2026 Demo Subsystem"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());
        let possible_dirs = [
            PathBuf::from("systems/src/carnival_demo/custom_pages"),
            PathBuf::from("static/custom_pages/carnival_demo"),
        ];
        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<foundry_core::CustomAdminPageSpec> {
        vec![
            foundry_core::CustomAdminPageSpec {
                key: "lottery_dashboard".to_string(),
                title: "抽奖运营大屏".to_string(),
                icon: "Gift".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/carnival_demo/ext/custom-pages/lottery_dashboard.html".to_string(),
                required_role: None,
            },
        ]
    }
}
```

---

## 4. 自定义管理后台 UI 与 SDK Bridge

每个子系统专属的管理大屏放在 `custom_pages/` 目录下，Foundry Admin UI 外壳会自动将其嵌入在对应子系统的侧边栏与主工作区中：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>抽奖大屏</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold">🎁 自定义抽奖控制台</h1>
  <p id="admin-info" class="text-xs text-slate-500 mt-2">连接中...</p>

  <script>
    let authContext = null;

    // 1. 监听外壳发送的初始化上下文 (Token、当前子系统、主题、管理员角色)
    window.addEventListener('message', function(event) {
      if (event.data?.type === 'FOUNDRY_INIT') {
        authContext = event.data.payload;
        document.getElementById('admin-info').innerText =
          `当前管理员: ${authContext.admin.username} (${authContext.admin.role})`;

        // 自动适配深色模式
        if (authContext.theme === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
    });

    // 2. 向外壳发送通知 Toast
    function notifyParent(msg) {
      window.parent.postMessage({
        type: 'FOUNDRY_TOAST',
        payload: { message: msg, level: 'success' }
      }, '*');
    }
  </script>
</body>
</html>
```

---

## 5. 团队统一自定义代码仓库工作流

### 步骤 1：初始化团队单一自定义代码仓库

```bash
cargo run --bin foundry-cli -- system init-repo ../my-foundry-systems
```

### 步骤 2：在自定义仓库中管理所有子系统

```bash
cd ../my-foundry-systems
# 查看该仓库中已有的子系统
ls -l
# 包含 carnival_demo/ vip_mall/ order_center/ 等
```

### 步骤 3：一键校验所有子系统

```bash
cargo run --bin foundry-cli -- system validate ../my-foundry-systems
```

CLI 会自动扫描该自定义仓库下的所有子系统并输出校验状态：
```
🔍 Validating unified custom subsystems under: ["../my-foundry-systems"]
  👉 Found subsystem: 'carnival_demo'
     • Manifest (subsystem.json): ✅ Present
     • Custom Pages (custom_pages/): ✅ Present
     • Display Name: Carnival 2026 Demo
     • Version: 1.0.0
  👉 Found subsystem: 'vip_mall'
     • Manifest (subsystem.json): ✅ Present
     • Custom Pages (custom_pages/): ✅ Present
     • Display Name: VIP 尊享商城
     • Version: 1.0.0
✅ Validation completed: Found 2 valid subsystem package(s).
```

---

## 6. 一键合并打包生产产物

在持续集成或生产发布时，只需传入自定义仓库路径，构建流水线即可自动完成所有子系统与基建的合并打包：

```bash
# 传入自定义代码仓库路径一键打包
./scripts/build-release.sh --systems-dir ../my-foundry-systems
```

打包流水线自动完成：
1. 构建 React Admin SPA (`apps/admin`) 产物。
2. 汇集自定义仓库中所有子系统的 `custom_pages` 静态后台页面与清单配置。
3. 编译 release 优化的 Rust 服务端与 CLI 命令行工具。
4. 输出完整的独立生产发布包至 `dist/release/` 目录。

