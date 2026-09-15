---
title: "08. Admin Frontend Console (apps/admin)"
description: "Stage 8: In-depth analysis of React 18 + TailwindCSS admin console and dynamic form generation."
---

> **Codebase Navigation**：⬅️ Previous: [07. Blog Platform Full-Stack Example (blog_platform)](./07-blog-platform/) · [📋 Overview & Guide](./)

# 阶段 8：Admin 前端管理后台 (apps/admin)

## 概述

`apps/admin` 是 Foundry 框架的**完整前端管理后台**，采用现代 Web 技术栈构建：

- **技术栈**：React 18 + TypeScript + Vite + TailwindCSS
- **核心功能**：双模式 UI（平台模式 + 子系统模式）、Auto-CRUD 数据浏览器、RBAC 权限控制
- **架构特点**：SPA 单页应用、客户端路由、响应式设计、暗黑模式、国际化

---

## 1. 技术栈 (package.json)

### 生产依赖

```json
{
  "react": "^18.3.1",              // UI 框架
  "react-dom": "^18.3.1",
  "react-i18next": "^15.4.1",      // 国际化（支持中英文）
  "i18next": "^24.2.2",
  "i18next-browser-languagedetector": "^8.0.4",
  "lucide-react": "^0.475.0",      // 图标库（2000+ SVG 图标）
  "clsx": "^2.1.1",                // 条件类名工具
  "tailwind-merge": "^2.6.0"       // TailwindCSS 类名合并
}
```

### 开发工具

```json
{
  "vite": "^6.1.0",                // 构建工具（替代 Webpack）
  "@vitejs/plugin-react": "^4.3.4",
  "typescript": "^5.7.3",
  "tailwindcss": "^3.4.17",        // 原子化 CSS 框架
  "eslint": "^10.8.1",             // 代码检查
  "prettier": "^3.9.6"             // 代码格式化
}
```

### Scripts 命令

| 命令 | 功能 |
|------|------|
| `pnpm dev` | 启动开发服务器（端口 3000，代理 API 到 8080） |
| `pnpm build` | 生产构建（输出到 dist/） |
| `pnpm lint` | 运行 ESLint 检查 |
| `pnpm format` | 格式化代码 |
| `pnpm preview` | 预览生产构建 |

---

## 2. 构建配置 (vite.config.ts)

```typescript
export default defineConfig({
  base: '/admin/',              // 部署基础路径
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',  // 代理 API 请求到后端
        changeOrigin: true,
      },
    },
  },
});
```

**关键配置**：
- **base: '/admin/'**：前端部署在 `/admin` 路径下，对应后端静态文件服务
- **proxy**：开发环境中，所有 `/api/*` 请求转发到 `localhost:8080`
- **生产环境**：打包后直接由后端 Axum 的 `ServeDir` 服务（Stage 5 中 `router.rs` 配置）

---

## 3. 应用入口 (main.tsx & index.html)

### index.html
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Foundry - Platform Management</title>
  </head>
  <body class="bg-slate-50 text-slate-900 antialiased dark:bg-slate-950 dark:text-slate-100">
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

### main.tsx
```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';
import { initTheme } from './utils/theme';

initTheme();  // 初始化主题（从 localStorage 读取）

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

**启动流程**：
1. `initTheme()` 从 `localStorage` 读取暗黑模式偏好
2. 渲染 `<App />` 组件到 `#root` DOM 节点
3. TailwindCSS 通过 `dark:` 类名前缀支持暗黑模式

---

## 4. 类型定义 (types/index.ts)

### 核心类型

```typescript
// API 响应格式（对应 Stage 1 的 ApiResponse）
export interface ApiResponse<T> {
  code: number;       // 0 表示成功
  message: string;    // 错误消息
  data: T;            // 实际数据
  meta?: any;
}

// 管理员配置文件
export interface AdminProfile {
  id: string;
  username: string;
  email?: string;
  role: 'super_admin' | 'admin' | 'topic_admin';  // 三级角色
  allowed_systems: string[];  // ['*'] 表示所有系统
}

// 子系统实体
export interface SystemItem {
  id: string;
  slug: string;
  name: string;
  description?: string;
  status: number;         // 1=active, 0=archived
  created_at: string;
  updated_at: string;
  models_count?: number;  // 聚合字段
  configs_count?: number;
  records_count?: number;
}

// 平台统计摘要
export interface PlatformSummary {
  total_systems: number;
  active_systems: number;
  total_models: number;
  total_records: number;
  total_admins: number;
  total_audit_logs: number;
}

// 分页结果（对应 Stage 1 的 PaginatedData）
export interface PaginatedResult<T> {
  items: T[];
  pagination: {
    page: number;
    page_size: number;
    total: number;
    total_pages: number;
  };
}
```

### 路由状态

```typescript
export interface RouteState {
  mode: 'platform' | 'subsystem';  // 双模式
  platformTab: 'dashboard' | 'systems' | 'admins' | 'audit_logs';
  subsystemSlug: string | null;
  subsystemTab: 'overview' | 'configs' | 'models' | 'data' | 'apis' | 'audit_logs' | 'settings' | 'custom';
  customPageKey?: string | null;  // 自定义页面标识
  params: Record<string, string>;  // 查询参数（如 page、model）
}
```

---

## 5. API 服务层 (services/api.ts)

### 请求封装

```typescript
const ADMIN_API_BASE = '/api/v1/admin';
const AUTO_API_BASE = '/api/v1';

function getHeaders(): HeadersInit {
  const token = localStorage.getItem('foundry_token');
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;  // JWT 认证
  }
  return headers;
}

async function request<T>(baseUrl: string, path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${baseUrl}${path}`, {
    ...options,
    headers: { ...getHeaders(), ...options.headers },
  });

  const json: ApiResponse<T> = await res.json();
  if (!res.ok || json.code !== 0) {
    throw new Error(json.message || 'Request failed');
  }
  return json.data;  // 自动解包 ApiResponse
}
```

### API 方法分类

**认证相关**：
```typescript
export const api = {
  login: (data: any) =>
    request<{ token: string; admin: AdminProfile }>(ADMIN_API_BASE, '/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  me: () => request<AdminProfile>(ADMIN_API_BASE, '/auth/me'),
}
```

**子系统管理**：
```typescript
listSystems: (params?: SystemQuery) => {
  const search = new URLSearchParams();
  if (params?.page) search.set('page', params.page.toString());
  if (params?.keyword) search.set('keyword', params.keyword);
  const qs = search.toString() ? `?${search.toString()}` : '';
  return request<PaginatedResult<SystemItem>>(ADMIN_API_BASE, `/systems${qs}`);
},
getSystemBySlug: (systemSlug: string) =>
  request<SystemItem>(ADMIN_API_BASE, `/s/${systemSlug}/details`),
createSystem: (data: any) =>
  request<SystemItem>(ADMIN_API_BASE, '/systems', {
    method: 'POST',
    body: JSON.stringify(data),
  }),
```

**Auto-CRUD 动态记录**：
```typescript
listRecords: (systemSlug: string, modelSlug: string, params?: { page?: number; page_size?: number }) => {
  const qs = params ? `?page=${params.page || 1}&page_size=${params.page_size || 20}` : '';
  return request<PaginatedResult<ModelRecordItem>>(
    AUTO_API_BASE,
    `/s/${systemSlug}/${modelSlug}${qs}`,
  );
},
createRecord: (systemSlug: string, modelSlug: string, data: any) =>
  request<ModelRecordItem>(AUTO_API_BASE, `/s/${systemSlug}/${modelSlug}`, {
    method: 'POST',
    body: JSON.stringify(data),
  }),
updateRecord: (systemSlug: string, modelSlug: string, id: number, data: any) =>
  request<ModelRecordItem>(AUTO_API_BASE, `/s/${systemSlug}/${modelSlug}/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  }),
deleteRecord: (systemSlug: string, modelSlug: string, id: number) =>
  request<void>(AUTO_API_BASE, `/s/${systemSlug}/${modelSlug}/${id}`, {
    method: 'DELETE',
  }),
```

**配置管理**：
```typescript
getAggregatedConfigs: (systemSlug: string) =>
  request<Record<string, any>>(AUTO_API_BASE, `/s/${systemSlug}/configs`),
updateAggregatedConfigs: (systemSlug: string, data: Record<string, any>) =>
  request<void>(AUTO_API_BASE, `/s/${systemSlug}/configs`, {
    method: 'PUT',
    body: JSON.stringify(data),
  }),
```

---

## 6. 客户端路由 (utils/router.ts)

### 路由解析器

```typescript
export function parseRoute(pathname: string, search: string): RouteState {
  const searchParams = new URLSearchParams(search);
  const params: Record<string, string> = {};
  searchParams.forEach((value, key) => { params[key] = value; });

  const cleanPath = pathname.replace(/\/+$/, '') || '/';

  // 匹配子系统路由：/admin/s/:slug/*
  const subsystemMatch = cleanPath.match(/^\/admin\/s\/([^/]+)(?:\/(.*))?$/);
  if (subsystemMatch) {
    const slug = subsystemMatch[1];
    const subPath = subsystemMatch[2] || 'overview';

    // 解析子路径
    if (subPath.startsWith('custom/')) {
      return {
        mode: 'subsystem',
        platformTab: 'systems',
        subsystemSlug: slug,
        subsystemTab: 'custom',
        customPageKey: subPath.replace(/^custom\//, ''),
        params,
      };
    } else if (subPath === 'data' || subPath === 'data_explorer') {
      return { mode: 'subsystem', subsystemSlug: slug, subsystemTab: 'data', ... };
    }
    // ... 其他 tab 匹配
  }

  // 匹配平台路由：/admin/:tab
  let platformTab: RouteState['platformTab'];
  if (cleanPath === '/admin/systems') platformTab = 'systems';
  else if (cleanPath === '/admin/admins') platformTab = 'admins';
  else platformTab = 'dashboard';

  return { mode: 'platform', platformTab, ... };
}
```

### 路由 Hook

```typescript
export function useAppRouter() {
  const [route, setRoute] = useState<RouteState>(() =>
    parseRoute(window.location.pathname, window.location.search)
  );

  useEffect(() => {
    const handlePopState = () => {
      setRoute(parseRoute(window.location.pathname, window.location.search));
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const navigatePlatform = useCallback((tab, params?, replace = false) => {
    const newUrl = buildRouteUrl({ mode: 'platform', platformTab: tab, params });
    if (replace) {
      window.history.replaceState(null, '', newUrl);
    } else {
      window.history.pushState(null, '', newUrl);
    }
    setRoute(parseRoute(window.location.pathname, window.location.search));
  }, []);

  const navigateSubsystem = useCallback((slug, tab = 'overview', params?, replace = false, customPageKey?) => {
    const newUrl = buildRouteUrl({
      mode: 'subsystem',
      subsystemSlug: slug,
      subsystemTab: tab,
      customPageKey,
      params,
    });
    // ... 同上
  }, []);

  const updateParams = useCallback((newParams, replace = true) => {
    setRoute((prev) => {
      const mergedParams = { ...prev.params, ...newParams };
      // 删除空值参数
      Object.keys(mergedParams).forEach((k) => {
        if (mergedParams[k] === undefined || mergedParams[k] === null || mergedParams[k] === '') {
          delete mergedParams[k];
        }
      });
      // ... 构建新 URL 并更新历史
    });
  }, []);

  return { route, navigatePlatform, navigateSubsystem, updateParams };
}
```

**设计要点**：
- **无依赖路由库**：手动实现基于 History API 的客户端路由
- **双模式切换**：平台模式 ↔ 子系统模式
- **查询参数管理**：`updateParams` 用于分页、筛选等场景
- **浏览器前进/后退**：通过 `popstate` 事件同步状态

---

## 7. 应用主组件 (App.tsx)

### 核心状态

```typescript
export function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('foundry_token'));
  const [admin, setAdmin] = useState<AdminProfile | null>(null);
  const [systems, setSystems] = useState<SystemItem[]>([]);
  const [currentSystem, setCurrentSystem] = useState<SystemItem | null>(null);
  const [loading, setLoading] = useState(true);

  const { route, navigatePlatform, navigateSubsystem, updateParams } = useAppRouter();
```

### 认证流程

```typescript
const handleLoginSuccess = (newToken: string, newAdmin: AdminProfile) => {
  localStorage.setItem('foundry_token', newToken);
  setToken(newToken);
  setAdmin(newAdmin);
  fetchProfileAndSystems();  // 加载用户信息和系统列表
};

const handleLogout = useCallback(() => {
  localStorage.removeItem('foundry_token');
  setToken(null);
  setAdmin(null);
  setSystems([]);
  setCurrentSystem(null);
}, []);
```

### 权限守卫（客户端）

```typescript
useEffect(() => {
  if (!admin) return;
  if (route.mode === 'platform') {
    // 只有 Super Admin 可以访问 Admins 页面
    if (route.platformTab === 'admins' && admin.role !== 'super_admin') {
      navigatePlatform('dashboard');
    }
    // Topic Admin 不能访问全局审计日志
    else if (route.platformTab === 'audit_logs' && admin.role === 'topic_admin') {
      navigatePlatform('dashboard');
    }
  } else if (route.mode === 'subsystem' && route.subsystemSlug) {
    // Topic Admin 只能访问授权的子系统
    if (admin.role === 'topic_admin') {
      const hasAccess =
        admin.allowed_systems.includes('*') ||
        admin.allowed_systems.includes(route.subsystemSlug);
      if (!hasAccess && systems.length > 0) {
        navigatePlatform('systems');
      }
    }
  }
}, [route.mode, route.platformTab, route.subsystemSlug, admin, systems, navigatePlatform]);
```

**注意**：
- 客户端权限守卫仅用于 UI 控制
- 后端 `auth_middleware` 是真正的安全屏障（Stage 5）
- 前端守卫防止用户手动修改 URL 看到无权限页面

### 页面路由分发

```typescript
return (
  <Layout
    route={route}
    onNavigatePlatform={navigatePlatform}
    onNavigateSubsystem={navigateSubsystem}
    admin={admin}
    systems={systems}
    currentSystem={currentSystem}
    onSelectSystem={handleSelectSystem}
    onLogout={handleLogout}
  >
    {/* 1. 子系统模式 */}
    {route.mode === 'subsystem' && currentSystem && (
      <>
        {route.subsystemTab === 'overview' && <SubsystemOverviewPage {...} />}
        {route.subsystemTab === 'configs' && <ConfigsPage {...} />}
        {route.subsystemTab === 'models' && <ModelsPage {...} />}
        {route.subsystemTab === 'data' && <DataExplorerPage {...} />}
        {route.subsystemTab === 'apis' && <SubsystemApisPage {...} />}
        {route.subsystemTab === 'audit_logs' && <AuditLogsPage {...} />}
        {route.subsystemTab === 'settings' && <SubsystemSettingsPage {...} />}
        {route.subsystemTab === 'custom' && route.customPageKey && (
          <CustomAdminPageViewer currentSystem={currentSystem} pageKey={route.customPageKey} admin={admin} />
        )}
      </>
    )}

    {/* 2. 平台模式 */}
    {route.mode === 'platform' && (
      <>
        {route.platformTab === 'dashboard' && <DashboardPage {...} />}
        {route.platformTab === 'systems' && <SystemsPage {...} />}
        {route.platformTab === 'admins' && <AdminsPage {...} />}
        {route.platformTab === 'audit_logs' && <AuditLogsPage {...} />}
      </>
    )}
  </Layout>
);
```

---

## 8. 布局组件 (components/Layout.tsx)

### 顶部导航栏

```typescript
<header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b bg-white/80 backdrop-blur dark:bg-slate-900/80">
  <div className="flex items-center gap-4">
    {/* Logo */}
    <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-500">
      F
    </div>
    <span className="font-bold">FOUNDRY</span>
    <span className="font-mono text-[10px]">v0.1.0</span>

    {/* 子系统模式：显示面包屑 */}
    {isSubsystemMode && (
      <>
        <button onClick={() => onNavigatePlatform('systems')}>
          <ArrowLeft /> 返回平台
        </button>
        <div className="flex items-center gap-2">
          <span>Subsystem: {currentSystem.name}</span>
          <span className="font-mono">/{currentSystem.slug}</span>
        </div>
      </>
    )}
  </div>

  <div className="flex items-center gap-2">
    <ThemeToggle />  {/* 暗黑模式切换 */}
    <button onClick={toggleLanguage}>  {/* 中英文切换 */}
      <Globe /> {i18n.language.startsWith('zh') ? '中文' : 'EN'}
    </button>
    {/* 用户头像 + 登出按钮 */}
    <div className="flex items-center gap-2">
      <div className="rounded-full">{admin.username[0]?.toUpperCase()}</div>
      <div>{admin.username} - {getRoleDisplayName(admin.role)}</div>
      <button onClick={onLogout}><LogOut /></button>
    </div>
  </div>
</header>
```

### 侧边栏导航

**平台模式**：
```typescript
const platformNavItems = [
  { id: 'dashboard', label: '仪表盘', icon: LayoutDashboard },
  { id: 'systems', label: '子系统', icon: Layers },
  { id: 'audit_logs', label: '审计日志', icon: FileClock, hideForTopicAdmin: true },
  { id: 'admins', label: '管理员', icon: ShieldCheck, superOnly: true },
];

// 渲染
{platformNavItems.map((item) => {
  if (item.superOnly && admin.role !== 'super_admin') return null;
  if (item.hideForTopicAdmin && admin.role === 'topic_admin') return null;
  const isActive = route.platformTab === item.id;
  return (
    <button
      onClick={() => onNavigatePlatform(item.id)}
      className={isActive ? 'bg-emerald-50 text-emerald-700' : 'text-slate-600'}
    >
      <item.icon /> {item.label}
    </button>
  );
})}
```

**子系统模式**：
```typescript
const subsystemNavItems = [
  { id: 'overview', label: '概览', icon: Compass },
  { id: 'configs', label: '配置', icon: Sliders },
  { id: 'models', label: '数据模型', icon: Database },
  { id: 'data', label: '数据浏览器', icon: TableProperties },
  { id: 'apis', label: 'API 接口', icon: Code2 },
  { id: 'audit_logs', label: '审计日志', icon: FileClock },
  { id: 'settings', label: '设置', icon: Settings },
];

// 自定义页面（从后端加载）
{customPages.map((page) => (
  <button
    key={page.key}
    onClick={() => onNavigateSubsystem(currentSystem.slug, 'custom', undefined, false, page.key)}
  >
    <Sparkles /> {page.title}
  </button>
))}
```

---

## 9. 核心页面组件

### 9.1 登录页面 (LoginPage.tsx)

```typescript
export function LoginPage({ onLoginSuccess }: LoginPageProps) {
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('admin123456');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const res = await api.login({ username, password });
      onLoginSuccess(res.token, res.admin);  // 保存 JWT 和用户信息
    } catch (err: any) {
      setError(err.message || '登录失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <Input type="text" value={username} onChange={(e) => setUsername(e.target.value)} />
      <Input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
      <Button type="submit" loading={loading}>登录</Button>
      {error && <div className="text-rose-700">{error}</div>}
    </form>
  );
}
```

### 9.2 仪表盘页面 (DashboardPage.tsx)

```typescript
export function DashboardPage({ admin, systems, onNavigatePlatform, onNavigateSubsystem }) {
  const [summary, setSummary] = useState<PlatformSummary | null>(null);

  useEffect(() => {
    if (admin.role === 'super_admin' || admin.role === 'admin') {
      api.getPlatformSummary().then(setSummary);
    }
  }, [admin.role]);

  const statCards = [
    { title: '子系统', value: summary?.total_systems, icon: Layers, tab: 'systems' },
    { title: '数据模型', value: summary?.total_models, icon: Database, tab: 'systems' },
    { title: '管理员', value: summary?.total_admins, icon: ShieldCheck, tab: 'admins' },
    { title: '审计日志', value: summary?.total_audit_logs, icon: FileClock, tab: 'audit_logs' },
  ];

  return (
    <div>
      {/* 统计卡片 */}
      <div className="grid grid-cols-4 gap-4">
        {statCards.map((card) => (
          <Card onClick={() => onNavigatePlatform(card.tab)}>
            <card.icon />
            <div className="text-2xl font-bold">{card.value}</div>
            <div className="text-sm text-slate-500">{card.title}</div>
          </Card>
        ))}
      </div>

      {/* 快速搜索 */}
      <form onSubmit={(e) => { e.preventDefault(); onNavigatePlatform('systems', { keyword: quickSearch }); }}>
        <Input placeholder="搜索子系统..." />
        <Button type="submit">搜索</Button>
      </form>

      {/* 子系统列表 */}
      <div className="grid grid-cols-3 gap-4">
        {systems.map((sys) => (
          <Card key={sys.id} onClick={() => onNavigateSubsystem(sys.slug, 'overview')}>
            <h3>{sys.name}</h3>
            <p>{sys.description}</p>
            <Badge>{sys.status === 1 ? 'Active' : 'Archived'}</Badge>
          </Card>
        ))}
      </div>
    </div>
  );
}
```

### 9.3 数据浏览器页面 (DataExplorerPage.tsx)

**核心功能**：
1. **模型选择器**：左侧列出所有模型（如 `posts`、`subscribers`）
2. **字段动态渲染**：根据 `ModelField` 动态生成表单
3. **Auto-CRUD 操作**：
   - **列表**：`api.listRecords(systemSlug, modelSlug, { page, page_size })`
   - **创建**：弹出 Modal，填写字段，调用 `api.createRecord()`
   - **编辑**：填充现有数据到 Modal，调用 `api.updateRecord()`
   - **删除**：确认后调用 `api.deleteRecord()`

```typescript
export function DataExplorerPage({ currentSystem, queryParams, onUpdateParams }) {
  const [models, setModels] = useState<ModelItem[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelItem | null>(null);
  const [fields, setFields] = useState<ModelFieldItem[]>([]);
  const [records, setRecords] = useState<ModelRecordItem[]>([]);
  const [total, setTotal] = useState(0);

  const currentPage = Number(queryParams?.page) || 1;
  const currentPageSize = Number(queryParams?.page_size) || 15;

  // 加载模型列表
  const loadModels = async (slug: string) => {
    const list = await api.listModels(slug);
    setModels(list);
    const targetModelSlug = queryParams?.model;
    const matched = list.find((m) => m.slug === targetModelSlug);
    if (matched) {
      setSelectedModel(matched);
    } else if (list.length > 0) {
      setSelectedModel(list[0]);
      onUpdateParams({ model: list[0].slug });
    }
  };

  // 加载记录数据
  const loadRecords = async (slug: string, model: ModelItem, page: number, pageSize: number) => {
    const [f, recs] = await Promise.all([
      api.listModelFields(slug, model.id),
      api.listRecords(slug, model.slug, { page, page_size: pageSize }),
    ]);
    setFields(f);
    setRecords(recs.items);
    setTotal(recs.pagination.total);
  };

  // 创建记录
  const handleCreate = async () => {
    await api.createRecord(currentSystem.slug, selectedModel.slug, formData);
    loadRecords(currentSystem.slug, selectedModel, currentPage, currentPageSize);
  };

  // 更新记录
  const handleUpdate = async (id: number) => {
    await api.updateRecord(currentSystem.slug, selectedModel.slug, id, formData);
    loadRecords(currentSystem.slug, selectedModel, currentPage, currentPageSize);
  };

  // 删除记录
  const handleDelete = async (id: number) => {
    if (confirm('确定删除此记录？')) {
      await api.deleteRecord(currentSystem.slug, selectedModel.slug, id);
      loadRecords(currentSystem.slug, selectedModel, currentPage, currentPageSize);
    }
  };

  return (
    <div className="flex gap-4">
      {/* 左侧：模型列表 */}
      <aside className="w-64">
        {models.map((m) => (
          <button
            key={m.id}
            onClick={() => {
              setSelectedModel(m);
              onUpdateParams({ model: m.slug, page: 1 });
            }}
            className={selectedModel?.id === m.id ? 'bg-emerald-50' : ''}
          >
            {m.name} ({m.slug})
          </button>
        ))}
      </aside>

      {/* 右侧：数据表格 */}
      <main className="flex-1">
        <div className="flex justify-between">
          <h2>{selectedModel?.name} 数据列表</h2>
          <Button onClick={openCreateModal}><Plus /> 创建记录</Button>
        </div>

        {/* 表格 */}
        <table>
          <thead>
            <tr>
              <th>ID</th>
              {fields.map((f) => <th key={f.name}>{f.label}</th>)}
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {records.map((rec) => (
              <tr key={rec.id}>
                <td>{rec.id}</td>
                {fields.map((f) => <td key={f.name}>{rec.data[f.name]}</td>)}
                <td>
                  <Button onClick={() => openEditModal(rec)}><Edit2 /></Button>
                  <Button onClick={() => handleDelete(rec.id)}><Trash2 /></Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {/* 分页器 */}
        <Pagination
          current={currentPage}
          pageSize={currentPageSize}
          total={total}
          onChange={(page) => onUpdateParams({ page })}
          onPageSizeChange={(page_size) => onUpdateParams({ page: 1, page_size })}
        />
      </main>
    </div>
  );
}
```

---

## 10. 国际化 (locales/i18n.ts)

### 配置

```typescript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import enUS from './en-US.json';
import zhCN from './zh-CN.json';

i18n
  .use(LanguageDetector)  // 自动检测浏览器语言
  .use(initReactI18next)
  .init({
    resources: {
      'en-US': { translation: enUS },
      'zh-CN': { translation: zhCN },
    },
    fallbackLng: 'en-US',
    interpolation: { escapeValue: false },
  });

export default i18n;
```

### 翻译文件示例 (zh-CN.json)

```json
{
  "app": {
    "platform_hub": "平台控制中心",
    "subsystem_hub": "子系统管理",
    "back_to_platform": "返回平台",
    "logout": "登出"
  },
  "nav": {
    "dashboard": "仪表盘",
    "systems": "子系统",
    "admins": "管理员",
    "audit_logs": "审计日志",
    "sub_overview": "概览",
    "configs": "配置",
    "models": "数据模型",
    "data_explorer": "数据浏览器",
    "sub_apis": "API 接口"
  },
  "auth": {
    "login_title": "登录 Foundry",
    "username": "用户名",
    "password": "密码",
    "login_failed": "登录失败，请检查用户名和密码"
  }
}
```

### 使用方式

```typescript
import { useTranslation } from 'react-i18next';

export function MyComponent() {
  const { t, i18n } = useTranslation();

  return (
    <div>
      <h1>{t('nav.dashboard')}</h1>  {/* 输出：仪表盘 */}
      <button onClick={() => i18n.changeLanguage('en-US')}>Switch to English</button>
    </div>
  );
}
```

---

## 11. 主题系统 (utils/theme.ts)

```typescript
export function initTheme() {
  const savedTheme = localStorage.getItem('foundry_theme');
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  if (savedTheme === 'dark' || (!savedTheme && prefersDark)) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

export function toggleTheme() {
  const isDark = document.documentElement.classList.toggle('dark');
  localStorage.setItem('foundry_theme', isDark ? 'dark' : 'light');
}
```

**TailwindCSS 暗黑模式配置** (tailwind.config.js)：
```javascript
module.exports = {
  darkMode: 'class',  // 通过 .dark 类名切换
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        // 自定义颜色变量
      },
    },
  },
};
```

**使用示例**：
```tsx
<div className="bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100">
  {/* 亮色模式：白底黑字，暗色模式：深灰底白字 */}
</div>
```

---

## 12. 完整数据流示例

### 场景：在 Blog 子系统中创建一篇文章

**1. 用户操作流程**：
```
登录 → 选择 Blog 子系统 → 点击"数据浏览器" → 选择"posts"模型 → 点击"创建记录" 
→ 填写表单（title、content、author） → 点击"提交"
```

**2. 前端代码执行流程**：

```typescript
// Step 1: 用户点击"创建记录"按钮
const openCreateModal = () => {
  const initial: Record<string, any> = {};
  fields.forEach((f) => {
    initial[f.name] = f.default_value !== undefined ? f.default_value : '';
  });
  setFormData(initial);  // { title: '', content: '', author: '' }
  setIsRecordModalOpen(true);
};

// Step 2: 用户填写表单
<Input
  value={formData['title']}
  onChange={(e) => setFormData({ ...formData, title: e.target.value })}
/>

// Step 3: 用户点击"提交"
const handleSubmit = async () => {
  try {
    await api.createRecord(
      'blog',           // systemSlug
      'posts',          // modelSlug
      formData          // { title: '...', content: '...', author: '...' }
    );
    setIsRecordModalOpen(false);
    loadRecords(currentSystem.slug, selectedModel, currentPage, currentPageSize);  // 刷新列表
  } catch (err) {
    alert(err.message);
  }
};
```

**3. HTTP 请求**：
```http
POST /api/v1/s/blog/posts
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "title": "Foundry 架构深度解析",
  "content": "本文详细介绍...",
  "author": "dev@example.com"
}
```

**4. 后端处理**（对应 Stage 5）：
```
extract_context → audit_middleware → auth_middleware 
→ handlers::autocrud::create
→ validate_record → execute_before_create (Hook)
→ RecordStore::create (插入 PostgreSQL)
→ execute_after_create (Hook)
→ 返回 ApiResponse
```

**5. 响应返回前端**：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "id": 1001,
    "system_id": "blog",
    "model_slug": "posts",
    "data": {
      "title": "Foundry 架构深度解析",
      "content": "本文详细介绍...",
      "author": "dev@example.com",
      "hook_processed": true  // Hook 注入的字段
    },
    "created_at": "2026-09-15T10:30:00Z",
    "updated_at": "2026-09-15T10:30:00Z"
  }
}
```

**6. 前端处理响应**：
```typescript
// api.ts 中的 request 函数自动解包
const record: ModelRecordItem = await api.createRecord(...);
// 返回值直接是 data 字段，不包含 code/message

// 更新本地状态
setRecords([...records, record]);
```

---

## 13. 与后端 8 阶段的关联

| 后端阶段 | 前端对应功能 |
|---------|-------------|
| **Stage 1 (Core)** | `ApiResponse<T>` 类型定义、`AdminProfile` 类型、错误消息显示 |
| **Stage 2 (Storage)** | `SystemItem`、`ModelItem`、`ModelRecordItem` 类型映射 |
| **Stage 3 (Auth)** | JWT 存储（localStorage）、`Authorization` 头注入、角色权限守卫 |
| **Stage 4 (Extension)** | 创建记录时自动触发 Hook（前端无需感知，后端透明处理） |
| **Stage 5 (Engine)** | Auto-CRUD API 调用（`/api/v1/s/:slug/:model`）、审计日志查看 |
| **Stage 6 (Foundry)** | 整体集成，前端通过 Vite 代理访问后端 API |
| **Stage 7 (Blog)** | 在数据浏览器中可操作 `blog` 子系统的 `posts` 模型 |
| **Stage 8 (Admin)** | 本阶段 - 完整的前端实现 |

---

## 14. 前端架构设计模式

### 14.1 状态管理

**无全局状态库**：
- 不使用 Redux/Zustand 等状态管理库
- 通过 `App.tsx` 中的 `useState` + `props drilling` 传递状态
- 优点：简单直接，适合中小型后台管理系统

**本地状态提升**：
```typescript
// App.tsx 中定义全局状态
const [admin, setAdmin] = useState<AdminProfile | null>(null);
const [systems, setSystems] = useState<SystemItem[]>([]);

// 通过 props 传递给子组件
<Layout admin={admin} systems={systems} onLogout={handleLogout}>
  <DashboardPage admin={admin} systems={systems} />
</Layout>
```

### 14.2 数据获取模式

**组件内 fetch + useEffect**：
```typescript
useEffect(() => {
  if (currentSystem?.slug && selectedModel) {
    loadRecords(currentSystem.slug, selectedModel, currentPage, currentPageSize);
  }
}, [currentSystem, selectedModel, currentPage, currentPageSize]);
```

**优化空间**：
- 可引入 SWR 或 React Query 实现缓存、自动重试
- 当前实现每次切换页面都会重新请求

### 14.3 表单处理

**受控组件 + 本地状态**：
```typescript
const [formData, setFormData] = useState<Record<string, any>>({});

<Input
  value={formData['title']}
  onChange={(e) => setFormData({ ...formData, title: e.target.value })}
/>
```

**优化空间**：
- 可引入 React Hook Form 减少重新渲染
- 可引入 Zod 实现客户端验证

### 14.4 路由设计

**无依赖路由库的优势**：
- 完全掌控路由逻辑
- 无需学习 React Router 复杂 API
- 适合固定结构的后台系统

**局限性**：
- 不支持嵌套路由、动态路由匹配
- 需要手动维护 URL 解析逻辑

---

## 15. 前端技术亮点

### 15.1 TailwindCSS 原子化 CSS

**优势**：
- 无需写 CSS 文件，所有样式在 JSX 中定义
- 暗黑模式切换（`dark:` 前缀）
- 响应式设计（`sm:` `md:` `lg:` 前缀）

**示例**：
```tsx
<div className="
  flex items-center gap-3               // Flexbox 布局
  rounded-xl                            // 圆角
  px-3.5 py-2.5                         // 内边距
  text-sm font-medium                   // 文字样式
  bg-white dark:bg-slate-900            // 亮色/暗色背景
  border border-slate-200 dark:border-slate-800  // 边框
  hover:bg-slate-100 dark:hover:bg-slate-800     // 悬停效果
  transition                            // 动画过渡
">
  按钮
</div>
```

### 15.2 Lucide React 图标库

**特点**：
- 2000+ 高质量 SVG 图标
- 树摇优化（只打包使用的图标）
- 统一设计风格

**使用**：
```tsx
import { Layers, Database, Settings } from 'lucide-react';

<Layers className="h-4 w-4 text-emerald-600" />
```

### 15.3 暗黑模式实现

**HTML class 切换**：
```typescript
// 切换暗黑模式
document.documentElement.classList.toggle('dark');
```

**CSS 自动适配**：
```tsx
<div className="bg-white text-slate-900 dark:bg-slate-950 dark:text-slate-100">
  {/* 亮色模式：白底黑字 */}
  {/* 暗色模式：深灰底白字 */}
</div>
```

**浏览器偏好检测**：
```typescript
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
if (prefersDark) {
  document.documentElement.classList.add('dark');
}
```

---

## 16. 生产部署流程

### 构建步骤

```bash
cd apps/admin
pnpm install
pnpm build
```

**输出**：
```
apps/admin/dist/
├── index.html
├── assets/
│   ├── index-[hash].js
│   └── index-[hash].css
└── favicon.svg
```

### 后端集成（对应 Stage 5 router.rs）

```rust
// 静态文件服务
let admin_static_path = PathBuf::from("apps/admin/dist");
if admin_static_path.exists() {
    router = router.nest_service(
        "/admin",
        ServeDir::new(admin_static_path).not_found_service(
            ServeFile::new("apps/admin/dist/index.html")  // SPA fallback
        )
    );
}
```

**路由规则**：
- `/admin` → 返回 `index.html`
- `/admin/s/blog/data` → 返回 `index.html`（客户端路由接管）
- `/admin/assets/index-abc123.js` → 返回静态资源
- `/api/v1/*` → 后端 API（不受 ServeDir 影响）

### Nginx 反向代理（可选）

```nginx
server {
    listen 80;
    server_name example.com;

    # 前端静态资源
    location /admin {
        root /var/www/foundry/apps/admin/dist;
        try_files $uri $uri/ /admin/index.html;
    }

    # 后端 API
    location /api {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 17. 学习检查点

掌握 Admin 前端后，你应该能够：

✅ **理解**：React SPA 应用的基本结构（入口、路由、API 层）  
✅ **理解**：如何使用 TypeScript 定义类型与后端 API 对接  
✅ **理解**：客户端路由实现原理（History API + parseRoute）  
✅ **理解**：JWT 认证流程（登录 → 存储 token → 请求头注入）  
✅ **掌握**：TailwindCSS 原子化 CSS 编写方式  
✅ **掌握**：暗黑模式实现（class 切换 + CSS 变量）  
✅ **掌握**：国际化实现（i18next + 语言切换）  
✅ **实践**：为 Blog 子系统添加自定义管理页面  
✅ **实践**：在数据浏览器中添加批量操作功能  
✅ **实践**：实现管理员密码修改功能  
✅ **实践**：添加新的统计图表到仪表盘

---

## 18. 总结：Foundry 完整技术栈

### 后端（Rust）
- **Core**: SystemContext、AppError、ApiResponse、SubsystemModule trait
- **Storage**: PostgreSQL + SQLx、Redis、RecordStore（Zero-DDL）
- **Auth**: Argon2id、JWT、RBAC
- **Extension**: MutationHook、HookPipeline
- **Engine**: Axum、Auto-CRUD、中间件
- **Foundry**: FoundryApp、FoundryBuilder

### 前端（TypeScript）
- **UI**: React 18、TailwindCSS、Lucide Icons
- **构建**: Vite、TypeScript、ESLint、Prettier
- **路由**: 自实现客户端路由（History API）
- **状态**: useState + props drilling（无 Redux）
- **API**: fetch + 自封装 request 函数
- **国际化**: i18next（中英文）
- **主题**: 暗黑模式（class 切换）

### 部署
- **开发环境**: Vite dev server (3000) + Cargo run (8080)
- **生产环境**: Rust 后端 + 静态文件服务（或 Nginx 反向代理）

---

**Stage 8 完成！你已经完整掌握了 Foundry 项目的所有代码！**

现在你可以：
1. 修改任意模块而不会破坏架构
2. 添加新的子系统和功能
3. 优化性能和安全性
4. 定制前端 UI 和交互
5. 部署到生产环境

🎉 **恭喜！你现在完全掌控了 Foundry 框架！**

---

### 📚 Stage Navigation
- ⬅️ **Previous Stage**: [07. Blog Platform Full-Stack Example (blog_platform)](./07-blog-platform/)
- 📋 **Guide Overview**: [Return to Overview](./)
