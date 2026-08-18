import {
  AdminProfile,
  ApiResponse,
  AuditLogItem,
  ModelFieldItem,
  ModelItem,
  ModelRecordItem,
  PaginatedResult,
  SystemConfigItem,
  SystemItem,
} from '../types';

const API_BASE = '/api/v1';

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

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
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
  login: (data: any) =>
    request<{ token: string; admin: AdminProfile }>('/admin/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  me: () => request<AdminProfile>('/admin/auth/me'),

  // Systems
  listSystems: () => request<SystemItem[]>('/admin/systems'),
  createSystem: (data: any) =>
    request<SystemItem>('/admin/systems', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateSystem: (id: string, data: any) =>
    request<SystemItem>(`/admin/systems/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // System Configs
  getAggregatedConfigs: (systemSlug: string) =>
    request<Record<string, any>>(`/s/${systemSlug}/configs`),
  updateAggregatedConfigs: (systemSlug: string, data: Record<string, any>) =>
    request<void>(`/s/${systemSlug}/configs`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  listConfigSchema: (systemSlug: string) =>
    request<SystemConfigItem[]>(`/admin/s/${systemSlug}/configs/schema`),
  upsertConfigSchema: (systemSlug: string, data: any) =>
    request<SystemConfigItem>(`/admin/s/${systemSlug}/configs/schema`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // Models & Fields
  listModels: (systemSlug: string) => request<ModelItem[]>(`/admin/s/${systemSlug}/models`),
  createModel: (systemSlug: string, data: any) =>
    request<ModelItem>(`/admin/s/${systemSlug}/models`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  listModelFields: (systemSlug: string, modelId: number) =>
    request<ModelFieldItem[]>(`/admin/s/${systemSlug}/models/${modelId}/fields`),
  addModelField: (systemSlug: string, modelId: number, data: any) =>
    request<ModelFieldItem>(`/admin/s/${systemSlug}/models/${modelId}/fields`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // Dynamic Records Auto-CRUD
  listRecords: (
    systemSlug: string,
    modelSlug: string,
    params?: { page?: number; page_size?: number },
  ) => {
    const qs = params ? `?page=${params.page || 1}&page_size=${params.page_size || 20}` : '';
    return request<PaginatedResult<ModelRecordItem>>(`/s/${systemSlug}/${modelSlug}${qs}`);
  },
  createRecord: (systemSlug: string, modelSlug: string, data: any) =>
    request<ModelRecordItem>(`/s/${systemSlug}/${modelSlug}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateRecord: (systemSlug: string, modelSlug: string, id: number, data: any) =>
    request<ModelRecordItem>(`/s/${systemSlug}/${modelSlug}/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  deleteRecord: (systemSlug: string, modelSlug: string, id: number) =>
    request<void>(`/s/${systemSlug}/${modelSlug}/${id}`, {
      method: 'DELETE',
    }),

  // Admins
  listAdmins: () => request<AdminProfile[]>('/admin/admins'),
  createAdmin: (data: any) =>
    request<AdminProfile>('/admin/admins', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // Audit Logs
  listAuditLogs: (params?: { page?: number; system_slug?: string }) => {
    const search = new URLSearchParams();
    if (params?.page) search.set('page', params.page.toString());
    if (params?.system_slug) search.set('system_slug', params.system_slug);
    const qs = search.toString() ? `?${search.toString()}` : '';
    return request<PaginatedResult<AuditLogItem>>(`/admin/audit-logs${qs}`);
  },
};
