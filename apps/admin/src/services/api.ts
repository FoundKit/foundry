import {
  AdminProfile,
  ApiResponse,
  AuditLogItem,
  CustomAdminPageItem,
  ModelFieldItem,
  ModelItem,
  ModelRecordItem,
  PaginatedResult,
  PlatformLoginSettings,
  PlatformSummary,
  PublicAuthSettings,
  SystemConfigItem,
  SystemItem,
  SystemQuery,
  SystemStats,
} from '../types';

const ADMIN_API_BASE = '/api/admin';
const AUTO_API_BASE = '/api';

function getHeaders(): HeadersInit {
  const token = localStorage.getItem('foundry_token');
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

async function request<T>(baseUrl: string, path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${baseUrl}${path}`, {
    ...options,
    headers: {
      ...getHeaders(),
      ...options.headers,
    },
  });

  const json: ApiResponse<T> = await res.json();
  if (!res.ok || json.code !== 0) {
    throw new Error(json.message || 'Request failed');
  }
  return json.data;
}

export const api = {
  // Auth
  getPublicAuthSettings: () => request<PublicAuthSettings>(ADMIN_API_BASE, '/auth/settings'),
  login: (data: any) =>
    request<{ token: string; admin: AdminProfile }>(ADMIN_API_BASE, '/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  sendLoginEmailCode: (email: string) =>
    request<{ message: string }>(ADMIN_API_BASE, '/auth/email/send-code', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  loginWithEmail: (data: { email: string; code: string }) =>
    request<{ token: string; admin: AdminProfile }>(ADMIN_API_BASE, '/auth/email/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  sendBindEmailCode: (email: string) =>
    request<{ message: string }>(ADMIN_API_BASE, '/auth/bind-email/send-code', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  verifyBindEmail: (data: { email: string; code: string }) =>
    request<AdminProfile>(ADMIN_API_BASE, '/auth/bind-email/verify', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  changePassword: (data: { old_password: string; new_password: string }) =>
    request<AdminProfile>(ADMIN_API_BASE, '/auth/password', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  me: () => request<AdminProfile>(ADMIN_API_BASE, '/auth/me'),

  // Platform Login Settings (Super Admin Only)
  getPlatformLoginSettings: () =>
    request<PlatformLoginSettings>(ADMIN_API_BASE, '/platform/login-settings'),
  updatePlatformLoginSettings: (data: PlatformLoginSettings) =>
    request<PlatformLoginSettings>(ADMIN_API_BASE, '/platform/login-settings', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // Platform Summary
  getPlatformSummary: () => request<PlatformSummary>(ADMIN_API_BASE, '/platform/summary'),

  // Systems
  listSystems: (params?: SystemQuery) => {
    const search = new URLSearchParams();
    if (params?.page) search.set('page', params.page.toString());
    if (params?.page_size) search.set('page_size', params.page_size.toString());
    if (params?.id) search.set('id', params.id);
    if (params?.slug) search.set('slug', params.slug);
    if (params?.name) search.set('name', params.name);
    if (params?.keyword) search.set('keyword', params.keyword);
    if (params?.status !== undefined && params?.status !== null) {
      search.set('status', params.status.toString());
    }
    const qs = search.toString() ? `?${search.toString()}` : '';
    return request<PaginatedResult<SystemItem>>(ADMIN_API_BASE, `/systems${qs}`);
  },
  getSystem: (id: string) => request<SystemItem>(ADMIN_API_BASE, `/systems/${id}`),
  getSystemBySlug: (systemSlug: string) =>
    request<SystemItem>(ADMIN_API_BASE, `/s/${systemSlug}/details`),
  getSystemStats: (systemSlug: string) =>
    request<SystemStats>(ADMIN_API_BASE, `/s/${systemSlug}/stats`),
  createSystem: (data: any) =>
    request<SystemItem>(ADMIN_API_BASE, '/systems', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateSystem: (id: string, data: any) =>
    request<SystemItem>(ADMIN_API_BASE, `/systems/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // Subsystem Custom Admin Pages
  listCustomPages: (systemSlug: string) =>
    request<CustomAdminPageItem[]>(ADMIN_API_BASE, `/s/${systemSlug}/custom-pages`),

  // System Configs
  getAggregatedConfigs: (systemSlug: string) =>
    request<Record<string, any>>(AUTO_API_BASE, `/s/${systemSlug}/configs`),
  updateAggregatedConfigs: (systemSlug: string, data: Record<string, any>) =>
    request<void>(AUTO_API_BASE, `/s/${systemSlug}/configs`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  listConfigSchema: (systemSlug: string) =>
    request<SystemConfigItem[]>(ADMIN_API_BASE, `/s/${systemSlug}/configs/schema`),
  upsertConfigSchema: (systemSlug: string, data: any) =>
    request<SystemConfigItem>(ADMIN_API_BASE, `/s/${systemSlug}/configs/schema`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // Models & Fields
  listModels: (systemSlug: string) =>
    request<ModelItem[]>(ADMIN_API_BASE, `/s/${systemSlug}/models`),
  listDeletedModels: (systemSlug: string) =>
    request<ModelItem[]>(ADMIN_API_BASE, `/s/${systemSlug}/models/trash`),
  createModel: (systemSlug: string, data: any) =>
    request<ModelItem>(ADMIN_API_BASE, `/s/${systemSlug}/models`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateModel: (systemSlug: string, modelId: number, data: any) =>
    request<ModelItem>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteModel: (systemSlug: string, modelId: number) =>
    request<void>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}`, {
      method: 'DELETE',
    }),
  restoreModel: (systemSlug: string, modelId: number) =>
    request<ModelItem>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}/restore`, {
      method: 'POST',
    }),
  listModelFields: (systemSlug: string, modelId: number) =>
    request<ModelFieldItem[]>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}/fields`),
  addModelField: (systemSlug: string, modelId: number, data: any) =>
    request<ModelFieldItem>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}/fields`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateModelField: (systemSlug: string, modelId: number, fieldId: number, data: any) =>
    request<ModelFieldItem>(
      ADMIN_API_BASE,
      `/s/${systemSlug}/models/${modelId}/fields/${fieldId}`,
      {
        method: 'PUT',
        body: JSON.stringify(data),
      },
    ),
  deleteModelField: (systemSlug: string, modelId: number, fieldId: number) =>
    request<void>(ADMIN_API_BASE, `/s/${systemSlug}/models/${modelId}/fields/${fieldId}`, {
      method: 'DELETE',
    }),

  // Dynamic Records Auto-CRUD
  listRecords: (
    systemSlug: string,
    modelSlug: string,
    params?: { page?: number; page_size?: number },
  ) => {
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

  // Admins
  listAdmins: () => request<AdminProfile[]>(ADMIN_API_BASE, '/admins'),
  createAdmin: (data: any) =>
    request<AdminProfile>(ADMIN_API_BASE, '/admins', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateAdmin: (id: string, data: any) =>
    request<AdminProfile>(ADMIN_API_BASE, `/admins/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // Audit Logs
  listAuditLogs: (params?: { page?: number; system_slug?: string }) => {
    const search = new URLSearchParams();
    if (params?.page) search.set('page', params.page.toString());
    if (params?.system_slug) search.set('system_slug', params.system_slug);
    const qs = search.toString() ? `?${search.toString()}` : '';
    return request<PaginatedResult<AuditLogItem>>(ADMIN_API_BASE, `/audit-logs${qs}`);
  },
};
