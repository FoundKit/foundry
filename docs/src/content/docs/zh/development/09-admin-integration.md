---
title: 管理控制台与 postMessage 桥接
description: 掌握 Foundry 解耦 Admin 控制台架构、子系统自定义大屏声明与前端 postMessage 免密鉴权协议。
---

# 管理控制台与 postMessage 桥接

本章内容：
1. 理解 Foundry 解耦的可视化 Admin 控制台架构；
2. 学会在子系统中声明专属运营大屏或自定义管理页面；
3. 掌握静态页面的目录约定与自动挂载机制；
4. 熟练运用 `postMessage` 免密握手协议获取管理员身份与主题设置。

---

## 1. 管理后台外壳架构

在传统全栈框架中，后台页面通常与业务控制器紧密耦合在一起（例如使用 Blade / ThinkTemplate 渲染）。

Foundry 采用现代前后端分离与沙箱微前端思想：
* **平台外壳 (`apps/admin`)**：基于 React 18/19、Tailwind CSS 与 Lucide 图标构建的高性能 SPA 控制台，提供系统概览、动态模型管理、审计日志与管理员管理；
* **业务子系统自定义大屏**：子系统开发者可以使用任意技术栈（原生 HTML/JS、Vue、React、Svelte 等）编写自己的专属报表看板；
* **沙箱隔离与单点免密登录**：Admin 外壳通过安全 iframe 内嵌子系统页面，并通过 `postMessage` 消息通道主动向子系统页面注入当前登录管理员的 JWT Token、用户信息与当前主题（Dark/Light）。

---

## 2. 声明子系统后台菜单项 (`custom_admin_pages`)

在 `src/systems/shop/mod.rs` 中重写 `custom_admin_pages` 方法：

```rust
// src/systems/shop/mod.rs
use foundry::prelude::*;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str { "shop" }
    fn display_name(&self) -> &'static str { "电子商务商城" }

    // 声明在 Admin 控制台侧边栏展示的自定义菜单项
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![
            CustomAdminPageSpec {
                key: "order_studio".to_string(),
                title: "订单运营大屏".to_string(),
                icon: "ShoppingCart".to_string(), // Lucide 图标名称
                page_type: "iframe".to_string(),
                // 页面静态资源挂载路径
                entry: "/api/v1/s/shop/ext/custom-pages/order_studio.html".to_string(),
                required_role: None, // None 表示继承子系统权限；亦可设为 Some("super_admin")
            }
        ]
    }
}
```

> [!TIP]
> **静态页面自动发现与托管**：
> 只需将 HTML 文件放入 `src/systems/<slug>/custom_pages/` 目录下（如 `src/systems/shop/custom_pages/order_studio.html`），Foundry 引擎启动时会自动检测并将其挂载到 `/api/v1/s/<slug>/ext/custom-pages/*`，无需手动配置静态文件服务！

---

## 3. 前端 postMessage 协议实战

在你的自定义 HTML 页面中（例如 `src/systems/shop/custom_pages/order_studio.html`），监听标准 `message` 事件即可无缝接入：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>订单运营大屏</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-slate-50 text-slate-800 transition-colors">
    <div class="max-w-4xl mx-auto">
        <header class="flex justify-between items-center pb-6 border-b border-slate-200">
            <div>
                <h1 id="page-title" class="text-2xl font-bold">订单运营实时看板</h1>
                <p id="user-badge" class="text-sm text-slate-500 mt-1">正在等待管理后台身份验证...</p>
            </div>
            <button id="refresh-btn" class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition">
                刷新数据
            </button>
        </header>

        <div class="grid grid-cols-3 gap-6 mt-6">
            <div class="p-5 bg-white rounded-xl shadow-sm border border-slate-100">
                <p class="text-sm text-slate-400">今日总订单数</p>
                <p id="total-orders" class="text-3xl font-extrabold text-slate-900 mt-2">--</p>
            </div>
        </div>
    </div>

    <script>
        let currentAuthToken = null;

        // 1. 监听来自父级 Admin 控制台的消息握手
        window.addEventListener('message', (event) => {
            const { type, token, user, system, theme } = event.data || {};

            // 握手认证初始化
            if (type === 'FOUNDRY_AUTH_INIT') {
                currentAuthToken = token;
                document.getElementById('user-badge').innerText = 
                    `当前操作员: ${user.username} (${user.role}) · 系统: ${system}`;
                
                // 加载业务数据
                loadDashboardData();
            }

            // 暗黑模式与浅色模式实时切换
            if (type === 'FOUNDRY_THEME_CHANGE') {
                if (theme === 'dark') {
                    document.body.classList.add('dark', 'bg-slate-900', 'text-slate-100');
                    document.body.classList.remove('bg-slate-50', 'text-slate-800');
                } else {
                    document.body.classList.remove('dark', 'bg-slate-900', 'text-slate-100');
                    document.body.classList.add('bg-slate-50', 'text-slate-800');
                }
            }
        });

        // 2. 使用 JWT 令牌向后端子系统 API 发起请求
        async function loadDashboardData() {
            if (!currentAuthToken) return;

            try {
                const res = await fetch('/api/v1/s/shop/ext/orders', {
                    headers: {
                        'Authorization': `Bearer ${currentAuthToken}`,
                        'Content-Type': 'application/json'
                    }
                });
                const json = await res.json();
                if (json.code === 0) {
                    const count = json.data && json.data.items ? json.data.items.length : 0;
                    document.getElementById('total-orders').innerText = count;
                }
            } catch (err) {
                console.error('加载看板数据异常:', err);
            }
        }

        document.getElementById('refresh-btn').addEventListener('click', loadDashboardData);
    </script>
</body>
</html>
```

---

## 4. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 不要将 Token 存储在持久化 localStorage 中**：
> 页面应该在内存变量中保存 `currentAuthToken`。当父级 Admin 控制台刷新或退出登录时，token 会自动失效，避免 XSS 窃取。
> 
> **2. 相对路径与挂载路径**：
> 在 `custom_admin_pages` 中声明的 `entry` 必须是以 `/api/v1/s/<slug>/ext/custom-pages/` 开头的完整路径。

下一节：[10. 综合实战：从零构建电商系统](./10-walkthrough-shop/)
