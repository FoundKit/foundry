export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T;
  meta?: any;
}

export interface AdminProfile {
  id: string;
  username: string;
  email?: string;
  role: 'super_admin' | 'admin' | 'topic_admin';
  allowed_systems: string[];
  is_initial_password?: boolean;
}

export interface SystemItem {
  id: string;
  slug: string;
  name: string;
  description?: string;
  status: number;
  created_at: string;
  updated_at: string;
  models_count?: number;
  configs_count?: number;
  records_count?: number;
}

export interface SystemQuery {
  page?: number;
  page_size?: number;
  id?: string;
  slug?: string;
  name?: string;
  keyword?: string;
  status?: number;
}

export interface SystemStats {
  id: string;
  slug: string;
  name: string;
  description?: string;
  status: number;
  created_at: string;
  models_count: number;
  configs_count: number;
  records_count: number;
  audit_logs_count: number;
}

export interface PlatformSummary {
  total_systems: number;
  active_systems: number;
  total_models: number;
  total_records: number;
  total_admins: number;
  total_audit_logs: number;
}

export interface CustomAdminPageItem {
  key: string;
  title: string;
  icon: string;
  type: 'iframe' | 'remote_component' | 'native_react' | 'custom_html';
  entry: string;
  required_role?: string;
}

export interface PublicAuthSettings {
  enable_password_login: boolean;
  enable_email_login: boolean;
  show_default_admin_hint?: boolean;
}

export interface PlatformLoginSettings {
  enable_password_login: boolean;
  enable_email_login: boolean;
  login_email_subject: string;
  login_email_template: string;
  smtp_configured?: boolean;
  super_admin_has_logged_in?: boolean;
}

export interface RouteState {
  mode: 'platform' | 'subsystem';
  platformTab: 'dashboard' | 'systems' | 'apps' | 'admins' | 'audit_logs' | 'settings';
  subsystemSlug: string | null;
  subsystemTab:
    | 'overview'
    | 'apps'
    | 'configs'
    | 'models'
    | 'data'
    | 'apis'
    | 'audit_logs'
    | 'settings'
    | 'custom';
  customPageKey?: string | null;
  params: Record<string, string>;
}

export interface SystemConfigItem {
  id: number;
  system_id: string;
  key: string;
  label: string;
  value_type:
    | 'string'
    | 'richtext'
    | 'image'
    | 'file'
    | 'integer'
    | 'number'
    | 'boolean'
    | 'datetime'
    | 'array';
  value: any;
  options: Record<string, any>;
  sort_order: number;
}

export interface ModelItem {
  id: number;
  system_id: string;
  slug: string;
  name: string;
  description?: string;
  is_system: boolean;
  status: number;
  permissions: Record<string, boolean>;
  created_at: string;
  updated_at?: string;
  deleted_at?: string | null;
}

export interface ModelFieldItem {
  id: number;
  model_id: number;
  name: string;
  label: string;
  field_type: string;
  is_required: boolean;
  default_value?: any;
  options: Record<string, any>;
  sort_order: number;
  created_at?: string;
  updated_at?: string;
  deleted_at?: string | null;
}

export interface ModelRecordItem {
  id: number;
  system_id: string;
  model_slug: string;
  data: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface PaginatedResult<T> {
  items: T[];
  pagination: {
    page: number;
    page_size: number;
    total: number;
    total_pages: number;
  };
}

export interface AuditLogItem {
  id: number;
  admin_id?: string;
  admin_username?: string;
  system_slug?: string;
  method: string;
  path: string;
  action_name?: string;
  headers: Record<string, any>;
  query_params?: string;
  body_params?: string;
  ip_address?: string;
  user_agent?: string;
  status_code?: number;
  duration_ms?: number;
  created_at: string;
}

export interface ApplicationModelPermissions {
  create: boolean;
  read: boolean;
  update: boolean;
  delete: boolean;
}

export interface ApplicationPermissions {
  configs?: {
    read?: boolean;
    write?: boolean;
  };
  models?: Record<string, ApplicationModelPermissions>;
}

export interface ApplicationItem {
  id: string;
  system_id: string;
  name: string;
  token: string;
  description?: string;
  status: number;
  permissions: ApplicationPermissions;
  created_at: string;
  updated_at: string;
  deleted_at?: string | null;
}
