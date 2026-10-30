import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  KeyRound,
  Plus,
  RefreshCw,
  Trash2,
  Edit2,
  Copy,
  Check,
  Eye,
  EyeOff,
  Code2,
  CheckSquare,
  Square,
  Shield,
  Layers,
} from 'lucide-react';
import { Card, Button, Input, Modal, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import {
  ApplicationItem,
  ApplicationModelPermissions,
  ApplicationPermissions,
  ModelItem,
  SystemItem,
} from '../types';

interface ApplicationsPageProps {
  currentSystem: SystemItem;
}

export function ApplicationsPage({ currentSystem }: ApplicationsPageProps) {
  const { t } = useTranslation();
  const [apps, setApps] = useState<ApplicationItem[]>([]);
  const [models, setModels] = useState<ModelItem[]>([]);
  const [loading, setLoading] = useState(false);

  // Modals state
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [editingApp, setEditingApp] = useState<ApplicationItem | null>(null);

  // Form state
  const [appName, setAppName] = useState('');
  const [appDesc, setAppDesc] = useState('');
  const [appStatus, setAppStatus] = useState<number>(1);
  const [configsRead, setConfigsRead] = useState(true);
  const [configsWrite, setConfigsWrite] = useState(false);
  const [modelPermissions, setModelPermissions] = useState<
    Record<string, ApplicationModelPermissions>
  >({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  // Secret display & copy
  const [revealedSecrets, setRevealedSecrets] = useState<Record<string, boolean>>({});
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  // Newly created credential modal
  const [createdCredential, setCreatedCredential] = useState<ApplicationItem | null>(null);

  const loadData = async () => {
    if (!currentSystem?.slug) {
      setApps([]);
      setModels([]);
      return;
    }
    setLoading(true);
    try {
      const [appList, modelList] = await Promise.all([
        api.listApplications(currentSystem.slug),
        api.listModels(currentSystem.slug),
      ]);
      setApps(appList);
      setModels(modelList);
    } catch (err) {
      console.error('Failed to load applications or models:', err);
      setApps([]);
      setModels([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, [currentSystem?.slug]);

  const copyToClipboard = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  const toggleRevealSecret = (appId: string) => {
    setRevealedSecrets((prev) => ({
      ...prev,
      [appId]: !prev[appId],
    }));
  };

  const openCreateModal = () => {
    setAppName('');
    setAppDesc('');
    setAppStatus(1);
    setConfigsRead(true);
    setConfigsWrite(false);
    setFormError(null);

    // Default permissions: read = true for all models
    const initialPerms: Record<string, ApplicationModelPermissions> = {};
    models.forEach((m) => {
      initialPerms[m.slug] = {
        create: false,
        read: true,
        update: false,
        delete: false,
      };
    });
    setModelPermissions(initialPerms);
    setIsCreateOpen(true);
  };

  const openEditModal = (app: ApplicationItem) => {
    setEditingApp(app);
    setAppName(app.name);
    setAppDesc(app.description || '');
    setAppStatus(app.status);
    setConfigsRead(app.permissions?.configs?.read ?? true);
    setConfigsWrite(app.permissions?.configs?.write ?? false);
    setFormError(null);

    const initialPerms: Record<string, ApplicationModelPermissions> = {};
    models.forEach((m) => {
      const existing = app.permissions?.models?.[m.slug];
      initialPerms[m.slug] = {
        create: existing?.create ?? false,
        read: existing?.read ?? false,
        update: existing?.update ?? false,
        delete: existing?.delete ?? false,
      };
    });
    setModelPermissions(initialPerms);
    setIsEditOpen(true);
  };

  const handleToggleModelAction = (
    modelSlug: string,
    action: keyof ApplicationModelPermissions,
  ) => {
    setModelPermissions((prev) => {
      const current = prev[modelSlug] || {
        create: false,
        read: false,
        update: false,
        delete: false,
      };
      return {
        ...prev,
        [modelSlug]: {
          ...current,
          [action]: !current[action],
        },
      };
    });
  };

  const handleToggleAllForModel = (modelSlug: string) => {
    setModelPermissions((prev) => {
      const current = prev[modelSlug];
      const allChecked =
        current && current.create && current.read && current.update && current.delete;
      return {
        ...prev,
        [modelSlug]: {
          create: !allChecked,
          read: !allChecked,
          update: !allChecked,
          delete: !allChecked,
        },
      };
    });
  };

  const handleBatchSetPermissions = (preset: 'all' | 'read_only' | 'clear') => {
    const updated: Record<string, ApplicationModelPermissions> = {};
    models.forEach((m) => {
      switch (preset) {
        case 'all':
          updated[m.slug] = { create: true, read: true, update: true, delete: true };
          break;
        case 'read_only':
          updated[m.slug] = { create: false, read: true, update: false, delete: false };
          break;
        case 'clear':
          updated[m.slug] = { create: false, read: false, update: false, delete: false };
          break;
      }
    });
    setModelPermissions(updated);
    if (preset === 'all') {
      setConfigsRead(true);
      setConfigsWrite(true);
    } else if (preset === 'read_only') {
      setConfigsRead(true);
      setConfigsWrite(false);
    } else {
      setConfigsRead(false);
      setConfigsWrite(false);
    }
  };

  const buildPermissionsPayload = (): ApplicationPermissions => ({
    configs: {
      read: configsRead,
      write: configsWrite,
    },
    models: modelPermissions,
  });

  const handleCreateSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem) return;
    if (!appName.trim()) {
      setFormError(t('apps.name_required') || '请输入应用名称');
      return;
    }

    setSubmitting(true);
    setFormError(null);
    try {
      const newApp = await api.createApplication(currentSystem.slug, {
        name: appName.trim(),
        description: appDesc.trim() || undefined,
        permissions: buildPermissionsPayload(),
      });
      setIsCreateOpen(false);
      setCreatedCredential(newApp);
      loadData();
    } catch (err: any) {
      setFormError(err.message || 'Failed to create application');
    } finally {
      setSubmitting(false);
    }
  };

  const handleEditSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem || !editingApp) return;
    if (!appName.trim()) {
      setFormError(t('apps.name_required') || '请输入应用名称');
      return;
    }

    setSubmitting(true);
    setFormError(null);
    try {
      await api.updateApplication(currentSystem.slug, editingApp.id, {
        name: appName.trim(),
        description: appDesc.trim() || undefined,
        status: appStatus,
        permissions: buildPermissionsPayload(),
      });
      setIsEditOpen(false);
      setEditingApp(null);
      loadData();
    } catch (err: any) {
      setFormError(err.message || 'Failed to update application');
    } finally {
      setSubmitting(false);
    }
  };

  const handleRegenerateSecret = async (app: ApplicationItem) => {
    if (!currentSystem) return;
    if (
      !confirm(
        t('apps.regenerate_confirm') ||
          `确定要重新生成应用 "${app.name}" 的访问 Token 吗？旧的凭据将立即失效，接入方需同步更新！`,
      )
    ) {
      return;
    }
    try {
      const updated = await api.regenerateAppToken(currentSystem.slug, app.id);
      setCreatedCredential(updated);
      loadData();
    } catch (err: any) {
      alert(err.message || 'Failed to regenerate token');
    }
  };

  const handleDeleteApp = async (app: ApplicationItem) => {
    if (!currentSystem) return;
    if (
      !confirm(
        t('apps.delete_confirm') ||
          `确定要删除应用 "${app.name}" 吗？删除后该应用将无法再访问任何系统接口！`,
      )
    ) {
      return;
    }
    try {
      await api.deleteApplication(currentSystem.slug, app.id);
      loadData();
    } catch (err: any) {
      alert(err.message || 'Failed to delete application');
    }
  };

  if (!currentSystem) {
    return (
      <div className="space-y-6">
        <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
          <div>
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('apps.title') || '应用接入与权限管理'}
            </h1>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
              {t('apps.desc') ||
                '配置接入该后端的客户端应用（如 Next.js SSR），分配独立的 Bearer Token（Authorization 鉴权），控制对系统基础接口及各数据模型的增删改查权限。'}
            </p>
          </div>
        </div>

        <Card className="p-12 text-center">
          <KeyRound className="mx-auto mb-3 h-12 w-12 text-slate-300 dark:text-slate-600" />
          <h2 className="text-base font-semibold text-slate-800 dark:text-slate-200">
            {t('apps.no_system_selected') || '未选择或暂无子系统/专题'}
          </h2>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
            应用接入与 Token 鉴权是按子系统维度隔离的。请先在【专题管理】中创建一个子系统。
          </p>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <div className="flex flex-wrap items-center gap-2.5">
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('apps.title') || '应用接入与权限管理'}
            </h1>
            <Badge variant="success">/{currentSystem.slug}</Badge>
            <span className="text-xs text-slate-500 dark:text-slate-400">
              ({currentSystem.name})
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
            {t('apps.desc') ||
              '配置接入当前子系统的客户端应用（如 Next.js SSR），分配独立的 Bearer Token（Authorization 鉴权），控制对系统基础接口及各数据模型的增删改查权限。'}
          </p>
        </div>

        <Button onClick={openCreateModal} className="gap-2">
          <Plus className="h-4 w-4" />
          <span>{t('apps.create_app') || '添加接入应用'}</span>
        </Button>
      </div>

      {/* Applications Table Card */}
      <Card className="overflow-hidden p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
            <thead className="border-b border-slate-200 bg-slate-50 text-xs font-medium uppercase text-slate-500 dark:border-slate-800 dark:bg-slate-950/60 dark:text-slate-400">
              <tr>
                <th className="px-5 py-3">{t('apps.app_name') || '应用名称 / 描述'}</th>
                <th className="px-5 py-3">
                  {t('apps.app_credentials') || 'API 访问凭据 (Bearer Token)'}
                </th>
                <th className="px-5 py-3">{t('apps.status') || '状态'}</th>
                <th className="px-5 py-3">{t('apps.permissions_summary') || '已授权接口范围'}</th>
                <th className="px-5 py-3">{t('apps.created_at') || '创建时间'}</th>
                <th className="px-5 py-3 text-right">{t('common.actions')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 text-xs dark:divide-slate-800/60">
              {apps.map((app) => {
                const isSecretRevealed = !!revealedSecrets[app.id];
                const modelPerms = app.permissions?.models || {};
                const permittedModelsCount = Object.values(modelPerms).filter(
                  (p) => p.read || p.create || p.update || p.delete,
                ).length;

                return (
                  <tr
                    key={app.id}
                    className="transition hover:bg-slate-50 dark:hover:bg-slate-800/30"
                  >
                    <td className="px-5 py-3">
                      <div className="font-semibold text-slate-900 dark:text-slate-100">
                        {app.name}
                      </div>
                      {app.description && (
                        <div className="mt-0.5 text-[11px] text-slate-500 dark:text-slate-400">
                          {app.description}
                        </div>
                      )}
                    </td>

                    <td className="px-5 py-3 font-mono text-xs">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                          {isSecretRevealed
                            ? app.token
                            : `${app.token.slice(0, 5)}••••••••••••••••••••••••`}
                        </span>
                        <button
                          onClick={() => toggleRevealSecret(app.id)}
                          className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                          title={isSecretRevealed ? 'Hide' : 'Show'}
                        >
                          {isSecretRevealed ? (
                            <EyeOff className="h-3.5 w-3.5" />
                          ) : (
                            <Eye className="h-3.5 w-3.5" />
                          )}
                        </button>
                        <button
                          onClick={() => copyToClipboard(app.token, `token_${app.id}`)}
                          className="text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400"
                          title={t('common.copy')}
                        >
                          {copiedKey === `token_${app.id}` ? (
                            <Check className="h-3.5 w-3.5 text-emerald-600" />
                          ) : (
                            <Copy className="h-3.5 w-3.5" />
                          )}
                        </button>
                      </div>
                    </td>

                    <td className="px-5 py-3">
                      {app.status === 1 ? (
                        <Badge variant="success">{t('common.active') || '启用'}</Badge>
                      ) : (
                        <Badge variant="danger">{t('common.disabled') || '禁用'}</Badge>
                      )}
                    </td>

                    <td className="px-5 py-3">
                      <div className="flex flex-wrap items-center gap-1.5">
                        {app.permissions?.configs?.read && (
                          <span className="inline-flex items-center rounded bg-emerald-50 px-1.5 py-0.5 text-[10px] font-medium text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300">
                            配置查
                          </span>
                        )}
                        {app.permissions?.configs?.write && (
                          <span className="inline-flex items-center rounded bg-amber-50 px-1.5 py-0.5 text-[10px] font-medium text-amber-700 dark:bg-amber-950/60 dark:text-amber-300">
                            配置改
                          </span>
                        )}
                        <span className="inline-flex items-center gap-1 rounded bg-sky-50 px-1.5 py-0.5 text-[10px] font-medium text-sky-700 dark:bg-sky-950/60 dark:text-sky-300">
                          <Layers className="h-2.5 w-2.5" />
                          {permittedModelsCount} / {models.length} 个模型授权
                        </span>
                      </div>
                    </td>

                    <td className="px-5 py-3 font-mono text-[11px] text-slate-500 dark:text-slate-400">
                      {new Date(app.created_at).toLocaleString()}
                    </td>

                    <td className="space-x-1 px-5 py-3 text-right">
                      <button
                        onClick={() => openEditModal(app)}
                        className="rounded p-1 text-slate-400 transition hover:text-emerald-600 dark:hover:text-emerald-400"
                        title={t('common.edit')}
                      >
                        <Edit2 className="h-3.5 w-3.5" />
                      </button>
                      <button
                        onClick={() => handleRegenerateSecret(app)}
                        className="rounded p-1 text-slate-400 transition hover:text-amber-600 dark:hover:text-amber-400"
                        title={t('apps.regenerate_secret') || '重置 Secret'}
                      >
                        <RefreshCw className="h-3.5 w-3.5" />
                      </button>
                      <button
                        onClick={() => handleDeleteApp(app)}
                        className="rounded p-1 text-slate-400 transition hover:text-rose-600 dark:hover:text-rose-400"
                        title={t('common.delete')}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </td>
                  </tr>
                );
              })}
              {apps.length === 0 && (
                <tr>
                  <td colSpan={6} className="py-12 text-center text-slate-500 dark:text-slate-400">
                    {loading ? (
                      t('common.loading')
                    ) : (
                      <div className="space-y-3">
                        <KeyRound className="mx-auto h-8 w-8 text-slate-400 opacity-60" />
                        <p>
                          {t('apps.no_apps') || '暂无接入应用。点击右上角“添加接入应用”进行配置。'}
                        </p>
                      </div>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Permissions Matrix Form Component */}
      {/* Create / Edit Application Modal */}
      <Modal
        isOpen={isCreateOpen || isEditOpen}
        onClose={() => {
          setIsCreateOpen(false);
          setIsEditOpen(false);
          setEditingApp(null);
        }}
        title={
          isCreateOpen
            ? t('apps.create_app_title') || '添加接入应用'
            : t('apps.edit_app_title') || '修改接入应用与接口权限'
        }
      >
        <form onSubmit={isCreateOpen ? handleCreateSubmit : handleEditSubmit} className="space-y-4">
          {formError && (
            <div className="rounded-lg bg-rose-50 p-3 text-xs text-rose-600 dark:bg-rose-950/60 dark:text-rose-300">
              {formError}
            </div>
          )}

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('apps.app_name') || '应用名称'} <span className="text-rose-500">*</span>
            </label>
            <Input
              required
              placeholder="e.g. Next.js SSR Web / Mobile App Backend"
              value={appName}
              onChange={(e) => setAppName(e.target.value)}
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('apps.app_desc') || '应用描述'}
            </label>
            <Input
              placeholder="e.g. 部署在 Vercel 上的前台 SSR 网站"
              value={appDesc}
              onChange={(e) => setAppDesc(e.target.value)}
            />
          </div>

          {isEditOpen && (
            <div>
              <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('apps.status') || '应用状态'}
              </label>
              <select
                className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
                value={appStatus}
                onChange={(e) => setAppStatus(Number(e.target.value))}
              >
                <option value={1}>{t('common.active') || '启用'}</option>
                <option value={0}>{t('common.disabled') || '禁用'}</option>
              </select>
            </div>
          )}

          {/* Section: Base System Interface Permissions */}
          <div className="space-y-2 rounded-xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-800 dark:bg-slate-900/50">
            <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
              <Shield className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
              <span>{t('apps.base_apis_permission') || '系统基础接口调用权限'}</span>
            </div>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <label className="flex cursor-pointer items-center gap-2 rounded-lg border border-slate-200 bg-white p-2.5 text-xs transition hover:border-slate-300 dark:border-slate-800 dark:bg-slate-950">
                <input
                  type="checkbox"
                  checked={configsRead}
                  onChange={(e) => setConfigsRead(e.target.checked)}
                  className="rounded border-slate-300 text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-900"
                />
                <div>
                  <div className="font-medium text-slate-900 dark:text-slate-100">
                    {t('apps.configs_read') || '读取系统聚合配置'}
                  </div>
                  <div className="font-mono text-[10px] text-slate-500">
                    GET /api/s/{currentSystem.slug}/configs
                  </div>
                </div>
              </label>

              <label className="flex cursor-pointer items-center gap-2 rounded-lg border border-slate-200 bg-white p-2.5 text-xs transition hover:border-slate-300 dark:border-slate-800 dark:bg-slate-950">
                <input
                  type="checkbox"
                  checked={configsWrite}
                  onChange={(e) => setConfigsWrite(e.target.checked)}
                  className="rounded border-slate-300 text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-900"
                />
                <div>
                  <div className="font-medium text-slate-900 dark:text-slate-100">
                    {t('apps.configs_write') || '修改系统聚合配置'}
                  </div>
                  <div className="font-mono text-[10px] text-slate-500">
                    PUT /api/s/{currentSystem.slug}/configs
                  </div>
                </div>
              </label>
            </div>
          </div>

          {/* Section: Dynamic Models CRUD Permissions Matrix */}
          <div className="space-y-2 rounded-xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-800 dark:bg-slate-900/50">
            <div className="flex flex-col justify-between gap-2 sm:flex-row sm:items-center">
              <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
                <Code2 className="h-4 w-4 text-sky-600 dark:text-sky-400" />
                <span>
                  {t('apps.models_crud_permission') || '数据模型接口增删改查权限控制 (4种接口)'}
                </span>
              </div>
              <div className="flex items-center gap-1.5 text-xs">
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  onClick={() => handleBatchSetPermissions('all')}
                  className="h-6 px-2 text-[10px]"
                >
                  {t('apps.select_all') || '全选'}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  onClick={() => handleBatchSetPermissions('read_only')}
                  className="h-6 px-2 text-[10px]"
                >
                  {t('apps.read_only') || '仅查 (Read Only)'}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  onClick={() => handleBatchSetPermissions('clear')}
                  className="h-6 px-2 text-[10px]"
                >
                  {t('apps.clear_all') || '清空'}
                </Button>
              </div>
            </div>

            <div className="max-h-60 overflow-y-auto rounded-lg border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-950">
              <table className="w-full text-left text-xs">
                <thead className="sticky top-0 border-b border-slate-200 bg-slate-100/90 backdrop-blur dark:border-slate-800 dark:bg-slate-900/90">
                  <tr>
                    <th className="px-3 py-2 font-medium text-slate-600 dark:text-slate-400">
                      {t('models.title') || '数据模型'}
                    </th>
                    <th className="px-2 py-2 text-center font-medium text-sky-600 dark:text-sky-400">
                      查 (Read)
                    </th>
                    <th className="px-2 py-2 text-center font-medium text-emerald-600 dark:text-emerald-400">
                      增 (Create)
                    </th>
                    <th className="px-2 py-2 text-center font-medium text-amber-600 dark:text-amber-400">
                      改 (Update)
                    </th>
                    <th className="px-2 py-2 text-center font-medium text-rose-600 dark:text-rose-400">
                      删 (Delete)
                    </th>
                    <th className="px-2 py-2 text-center font-medium text-slate-500">
                      {t('apps.toggle_row') || '整行'}
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100 dark:divide-slate-800/60">
                  {models.map((m) => {
                    const p = modelPermissions[m.slug] || {
                      create: false,
                      read: false,
                      update: false,
                      delete: false,
                    };
                    const isAllSelected = p.create && p.read && p.update && p.delete;

                    return (
                      <tr key={m.id} className="hover:bg-slate-50/60 dark:hover:bg-slate-800/20">
                        <td className="px-3 py-2">
                          <div className="font-medium text-slate-900 dark:text-slate-100">
                            {m.name}
                          </div>
                          <div className="font-mono text-[10px] text-slate-400">/{m.slug}</div>
                        </td>

                        <td className="px-2 py-2 text-center">
                          <input
                            type="checkbox"
                            checked={p.read}
                            onChange={() => handleToggleModelAction(m.slug, 'read')}
                            className="rounded border-slate-300 text-sky-600 focus:ring-sky-500 dark:border-slate-800 dark:bg-slate-900"
                          />
                        </td>

                        <td className="px-2 py-2 text-center">
                          <input
                            type="checkbox"
                            checked={p.create}
                            onChange={() => handleToggleModelAction(m.slug, 'create')}
                            className="rounded border-slate-300 text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-900"
                          />
                        </td>

                        <td className="px-2 py-2 text-center">
                          <input
                            type="checkbox"
                            checked={p.update}
                            onChange={() => handleToggleModelAction(m.slug, 'update')}
                            className="rounded border-slate-300 text-amber-600 focus:ring-amber-500 dark:border-slate-800 dark:bg-slate-900"
                          />
                        </td>

                        <td className="px-2 py-2 text-center">
                          <input
                            type="checkbox"
                            checked={p.delete}
                            onChange={() => handleToggleModelAction(m.slug, 'delete')}
                            className="rounded border-slate-300 text-rose-600 focus:ring-rose-500 dark:border-slate-800 dark:bg-slate-900"
                          />
                        </td>

                        <td className="px-2 py-2 text-center">
                          <button
                            type="button"
                            onClick={() => handleToggleAllForModel(m.slug)}
                            className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                            title="切换该模型所有权限"
                          >
                            {isAllSelected ? (
                              <CheckSquare className="h-3.5 w-3.5 text-emerald-600" />
                            ) : (
                              <Square className="h-3.5 w-3.5" />
                            )}
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                  {models.length === 0 && (
                    <tr>
                      <td colSpan={6} className="py-6 text-center text-slate-400">
                        {t('models.no_models') || '该子系统尚未定义任何数据模型'}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => {
                setIsCreateOpen(false);
                setIsEditOpen(false);
                setEditingApp(null);
              }}
            >
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={submitting}>
              {submitting ? t('common.loading') : t('common.save')}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Created Credential Modal */}
      {createdCredential && (
        <Modal
          isOpen={!!createdCredential}
          onClose={() => setCreatedCredential(null)}
          title={t('apps.credential_title') || '应用凭据生成成功'}
        >
          <div className="space-y-4">
            <div className="rounded-lg border border-emerald-200 bg-emerald-50/70 p-3 text-xs text-emerald-800 dark:border-emerald-800/60 dark:bg-emerald-950/40 dark:text-emerald-300">
              <p className="font-semibold">
                {t('apps.credential_notice') || '请及时复制并妥善保管您的 API Token 访问凭据！'}
              </p>
              <p className="mt-1">
                接入端应用（如 Next.js SSR 服务器、API
                客户端）向该后端发送请求时，只需在请求头中携带该 Token：
                <code className="ml-1 rounded bg-emerald-100 px-1 py-0.5 font-mono text-[11px] text-emerald-900 dark:bg-emerald-900/60 dark:text-emerald-200">
                  Authorization: Bearer &lt;Token&gt;
                </code>
                <span className="ml-1 text-[11px] text-emerald-700 dark:text-emerald-300">
                  （系统支持 <code>Authorization: Bearer &lt;Token&gt;</code> 或直接{' '}
                  <code>Authorization: &lt;Token&gt;</code>）
                </span>
              </p>
            </div>

            <div className="space-y-3 font-mono text-xs">
              <div>
                <label className="mb-1 block text-slate-500 dark:text-slate-400">
                  App Name (应用名称)
                </label>
                <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 font-sans text-sm font-semibold text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-100">
                  {createdCredential.name}
                </div>
              </div>

              <div>
                <label className="mb-1 block text-slate-500 dark:text-slate-400">
                  API Token (Bearer 凭据)
                </label>
                <div className="flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 dark:border-slate-800 dark:bg-slate-900">
                  <span className="break-all font-semibold text-emerald-600 dark:text-emerald-400">
                    {createdCredential.token}
                  </span>
                  <button
                    onClick={() => copyToClipboard(createdCredential.token, 'modal_token')}
                    className="ml-2 text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400"
                    title={t('common.copy')}
                  >
                    {copiedKey === 'modal_token' ? (
                      <Check className="h-4 w-4 text-emerald-600" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </button>
                </div>
              </div>

              <div>
                <label className="mb-1 block text-slate-500 dark:text-slate-400">
                  Next.js SSR 调用代码示例
                </label>
                <pre className="overflow-x-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-[11px] text-slate-200 dark:border-slate-800">
                  {`// Next.js Server-Side Request Example
const res = await fetch("http://localhost:8080/api/s/${currentSystem.slug}/configs", {
  headers: {
    "Authorization": "Bearer ${createdCredential.token}",
    "Content-Type": "application/json",
  },
});
const data = await res.json();`}
                </pre>
              </div>
            </div>

            <div className="flex justify-end pt-2">
              <Button onClick={() => setCreatedCredential(null)}>
                {t('common.confirm') || '我已妥善保存'}
              </Button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
