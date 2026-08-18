import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Plus,
  Layers,
  AlertCircle,
  PlusCircle,
  Search,
  RotateCcw,
  Copy,
  Check,
  ArrowRight,
  Edit2,
  Database,
  Sliders,
  TableProperties,
} from 'lucide-react';
import { Card, Button, Input, Textarea, Badge, Modal, Pagination } from '../components/UiWidgets';
import { api } from '../services/api';
import { AdminProfile, SystemItem } from '../types';

interface SystemsPageProps {
  admin: AdminProfile;
  queryParams: Record<string, string>;
  onUpdateParams: (params: Record<string, any>) => void;
  onRefresh: () => void;
  onEnterSubsystem: (system: SystemItem) => void;
}

export function SystemsPage({
  admin,
  queryParams,
  onUpdateParams,
  onRefresh,
  onEnterSubsystem,
}: SystemsPageProps) {
  const { t } = useTranslation();

  // Data state
  const [systems, setSystems] = useState<SystemItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [total, setTotal] = useState(0);

  // Search filter form states initialized from URL query params
  const [searchId, setSearchId] = useState(queryParams.id || '');
  const [searchSlug, setSearchSlug] = useState(queryParams.slug || '');
  const [searchName, setSearchName] = useState(queryParams.name || '');
  const [searchKeyword, setSearchKeyword] = useState(queryParams.keyword || '');
  const [searchStatus, setSearchStatus] = useState(queryParams.status || '');

  // Pagination parameters from URL
  const currentPage = Number(queryParams.page) || 1;
  const currentPageSize = Number(queryParams.page_size) || 10;

  // Modals state
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [createSlug, setCreateSlug] = useState('');
  const [createName, setCreateName] = useState('');
  const [createDescription, setCreateDescription] = useState('');
  const [createLoading, setCreateLoading] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const [editingSystem, setEditingSystem] = useState<SystemItem | null>(null);
  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editStatus, setEditStatus] = useState(1);
  const [editLoading, setEditLoading] = useState(false);

  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Fetch systems matching current URL query params
  const loadSystems = async () => {
    setLoading(true);
    try {
      const res = await api.listSystems({
        page: currentPage,
        page_size: currentPageSize,
        id: queryParams.id || undefined,
        slug: queryParams.slug || undefined,
        name: queryParams.name || undefined,
        keyword: queryParams.keyword || undefined,
        status:
          queryParams.status !== undefined && queryParams.status !== ''
            ? Number(queryParams.status)
            : undefined,
      });
      setSystems(res.items);
      setTotal(res.pagination.total);
    } catch (err) {
      console.error(err);
      setSystems([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSystems();
  }, [
    queryParams.page,
    queryParams.page_size,
    queryParams.id,
    queryParams.slug,
    queryParams.name,
    queryParams.keyword,
    queryParams.status,
  ]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onUpdateParams({
      page: 1, // Reset to page 1 on new search
      id: searchId.trim() || undefined,
      slug: searchSlug.trim() || undefined,
      name: searchName.trim() || undefined,
      keyword: searchKeyword.trim() || undefined,
      status: searchStatus !== '' ? searchStatus : undefined,
    });
  };

  const handleResetFilters = () => {
    setSearchId('');
    setSearchSlug('');
    setSearchName('');
    setSearchKeyword('');
    setSearchStatus('');
    onUpdateParams({
      page: 1,
      id: undefined,
      slug: undefined,
      name: undefined,
      keyword: undefined,
      status: undefined,
    });
  };

  const handlePageChange = (page: number) => {
    onUpdateParams({ page });
  };

  const handlePageSizeChange = (page_size: number) => {
    onUpdateParams({ page: 1, page_size });
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setCreateLoading(true);
    setCreateError(null);
    try {
      await api.createSystem({
        slug: createSlug.trim(),
        name: createName.trim(),
        description: createDescription.trim() || undefined,
      });
      setIsCreateOpen(false);
      setCreateSlug('');
      setCreateName('');
      setCreateDescription('');
      onRefresh();
      loadSystems();
    } catch (err: any) {
      setCreateError(err.message || 'Failed to create sub-system');
    } finally {
      setCreateLoading(false);
    }
  };

  const openEditModal = (sys: SystemItem) => {
    setEditingSystem(sys);
    setEditName(sys.name);
    setEditDescription(sys.description || '');
    setEditStatus(sys.status);
  };

  const handleEditSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingSystem) return;
    setEditLoading(true);
    try {
      await api.updateSystem(editingSystem.id, {
        name: editName.trim(),
        description: editDescription.trim() || undefined,
        status: editStatus,
      });
      setEditingSystem(null);
      onRefresh();
      loadSystems();
    } catch (err: any) {
      alert(err.message || 'Failed to update system');
    } finally {
      setEditLoading(false);
    }
  };

  const copyIdToClipboard = (id: string) => {
    navigator.clipboard.writeText(id);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
        <div>
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
            {t('systems.title')}
          </h1>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('systems.desc')}</p>
        </div>
        {(admin.role === 'super_admin' || admin.role === 'admin') && (
          <Button onClick={() => setIsCreateOpen(true)} className="gap-2">
            <Plus className="h-4 w-4" />
            <span>{t('systems.create')}</span>
          </Button>
        )}
      </div>

      {/* Filter & Search Bar */}
      <Card className="space-y-4">
        <form onSubmit={handleSearchSubmit} className="space-y-3">
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {/* 1. Keyword search */}
            <div>
              <label className="mb-1 block text-[11px] font-medium text-slate-600 dark:text-slate-400">
                {t('common.search')}
              </label>
              <Input
                placeholder={t('systems.search_keyword')}
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                className="text-xs"
              />
            </div>

            {/* 2. Slug filter */}
            <div>
              <label className="mb-1 block text-[11px] font-medium text-slate-600 dark:text-slate-400">
                {t('systems.slug')}
              </label>
              <Input
                placeholder={t('systems.search_slug')}
                value={searchSlug}
                onChange={(e) => setSearchSlug(e.target.value)}
                className="font-mono text-xs"
              />
            </div>

            {/* 3. Name filter */}
            <div>
              <label className="mb-1 block text-[11px] font-medium text-slate-600 dark:text-slate-400">
                {t('systems.name')}
              </label>
              <Input
                placeholder={t('systems.search_name')}
                value={searchName}
                onChange={(e) => setSearchName(e.target.value)}
                className="text-xs"
              />
            </div>

            {/* 4. ID filter */}
            <div>
              <label className="mb-1 block text-[11px] font-medium text-slate-600 dark:text-slate-400">
                {t('systems.id')}
              </label>
              <Input
                placeholder={t('systems.search_id')}
                value={searchId}
                onChange={(e) => setSearchId(e.target.value)}
                className="font-mono text-xs"
              />
            </div>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-slate-100 pt-1 dark:border-slate-800/80">
            {/* Status select */}
            <div className="flex items-center gap-2">
              <span className="text-xs text-slate-500">{t('systems.status')}:</span>
              <select
                value={searchStatus}
                onChange={(e) => setSearchStatus(e.target.value)}
                className="rounded-lg border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-800 focus:border-emerald-500 focus:outline-none dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
              >
                <option value="">{t('systems.all_status')}</option>
                <option value="1">{t('systems.active')}</option>
                <option value="0">{t('systems.disabled')}</option>
              </select>
            </div>

            {/* Action buttons */}
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="secondary"
                size="sm"
                onClick={handleResetFilters}
                className="gap-1.5"
              >
                <RotateCcw className="h-3.5 w-3.5" />
                <span>{t('systems.reset_button')}</span>
              </Button>
              <Button type="submit" size="sm" className="gap-1.5">
                <Search className="h-3.5 w-3.5" />
                <span>{t('systems.filter_button')}</span>
              </Button>
            </div>
          </div>
        </form>
      </Card>

      {/* Systems Table */}
      <Card className="overflow-hidden p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
            <thead className="border-b border-slate-200 bg-slate-50 text-xs font-medium uppercase text-slate-500 dark:border-slate-800 dark:bg-slate-950/60 dark:text-slate-400">
              <tr>
                <th className="px-5 py-3.5">{t('systems.name')}</th>
                <th className="px-5 py-3.5">{t('systems.slug')}</th>
                <th className="px-5 py-3.5">Scope & Stats</th>
                <th className="px-5 py-3.5">{t('systems.id')}</th>
                <th className="px-5 py-3.5">{t('systems.status')}</th>
                <th className="px-5 py-3.5 text-right">{t('systems.actions')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 text-xs dark:divide-slate-800/60">
              {systems.map((sys) => (
                <tr
                  key={sys.id}
                  className="transition hover:bg-slate-50 dark:hover:bg-slate-800/30"
                >
                  <td className="px-5 py-3.5">
                    <div className="flex items-center gap-2 font-semibold text-slate-900 dark:text-slate-100">
                      <Layers className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
                      <span>{sys.name}</span>
                    </div>
                    {sys.description && (
                      <div className="mt-0.5 max-w-xs truncate text-[11px] text-slate-500 dark:text-slate-400">
                        {sys.description}
                      </div>
                    )}
                  </td>

                  <td className="px-5 py-3.5 font-mono text-xs font-semibold text-emerald-600 dark:text-emerald-400">
                    /{sys.slug}
                  </td>

                  <td className="px-5 py-3.5">
                    <div className="flex items-center gap-3 text-[11px] text-slate-600 dark:text-slate-400">
                      <span className="flex items-center gap-1" title="Data Models">
                        <Database className="h-3 w-3 text-slate-400" />
                        <strong>{sys.models_count ?? 0}</strong> models
                      </span>
                      <span className="flex items-center gap-1" title="Topic Configs">
                        <Sliders className="h-3 w-3 text-slate-400" />
                        <strong>{sys.configs_count ?? 0}</strong> configs
                      </span>
                      <span className="flex items-center gap-1" title="Total Records">
                        <TableProperties className="h-3 w-3 text-slate-400" />
                        <strong>{sys.records_count ?? 0}</strong> recs
                      </span>
                    </div>
                  </td>

                  <td className="px-5 py-3.5">
                    <div className="flex items-center gap-1 font-mono text-[11px] text-slate-500 dark:text-slate-400">
                      <span className="max-w-[120px] truncate">{sys.id}</span>
                      <button
                        onClick={() => copyIdToClipboard(sys.id)}
                        className="rounded p-1 text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400"
                        title={t('common.copy')}
                      >
                        {copiedId === sys.id ? (
                          <Check className="h-3 w-3 text-emerald-600" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                      </button>
                    </div>
                  </td>

                  <td className="px-5 py-3.5">
                    <Badge variant={sys.status === 1 ? 'success' : 'default'}>
                      {sys.status === 1 ? t('systems.active') : t('systems.disabled')}
                    </Badge>
                  </td>

                  <td className="space-x-2 px-5 py-3.5 text-right">
                    {(admin.role === 'super_admin' || admin.role === 'admin') && (
                      <button
                        onClick={() => openEditModal(sys)}
                        className="rounded-lg p-1.5 text-slate-400 transition hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-slate-800 dark:hover:text-slate-200"
                        title={t('systems.edit')}
                      >
                        <Edit2 className="h-3.5 w-3.5" />
                      </button>
                    )}

                    <Button
                      size="sm"
                      onClick={() => onEnterSubsystem(sys)}
                      className="gap-1 bg-emerald-600 font-semibold text-white shadow-sm hover:bg-emerald-500"
                    >
                      <span>{t('systems.manage')}</span>
                      <ArrowRight className="h-3 w-3" />
                    </Button>
                  </td>
                </tr>
              ))}

              {systems.length === 0 && (
                <tr>
                  <td colSpan={6} className="px-4 py-16 text-center">
                    {loading ? (
                      <div className="text-xs text-slate-500 dark:text-slate-400">
                        {t('common.loading')}
                      </div>
                    ) : (
                      <>
                        <Layers className="mx-auto mb-3 h-10 w-10 text-slate-400 dark:text-slate-600" />
                        <h3 className="mb-1 text-base font-semibold text-slate-900 dark:text-slate-100">
                          {t('systems.empty_title')}
                        </h3>
                        <p className="mx-auto mb-4 max-w-sm text-xs text-slate-500 dark:text-slate-400">
                          {admin.role === 'topic_admin'
                            ? 'No authorized sub-systems found matching your criteria.'
                            : t('systems.empty_desc')}
                        </p>
                        {(admin.role === 'super_admin' || admin.role === 'admin') && (
                          <Button onClick={() => setIsCreateOpen(true)} className="mx-auto gap-2">
                            <PlusCircle className="h-4 w-4" />
                            <span>{t('systems.create')}</span>
                          </Button>
                        )}
                      </>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {/* Reusable Pagination */}
        <Pagination
          page={currentPage}
          pageSize={currentPageSize}
          total={total}
          onPageChange={handlePageChange}
          onPageSizeChange={handlePageSizeChange}
        />
      </Card>

      {/* Create System Modal */}
      <Modal
        isOpen={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
        title={t('systems.create')}
      >
        {createError && (
          <div className="mb-4 flex items-center gap-2 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{createError}</span>
          </div>
        )}
        <form onSubmit={handleCreate} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.slug')} (Immutable identifier)
            </label>
            <Input
              required
              placeholder="carnival_2026"
              value={createSlug}
              onChange={(e) => setCreateSlug(e.target.value.toLowerCase())}
            />
            <span className="mt-1 block text-[10px] text-slate-500 dark:text-slate-400">
              Alphanumeric with underscores/hyphens (e.g. carnival_2026, vip_mall)
            </span>
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.name')}
            </label>
            <Input
              required
              placeholder="Marketing Carnival 2026"
              value={createName}
              onChange={(e) => setCreateName(e.target.value)}
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.description')}
            </label>
            <Textarea
              rows={3}
              placeholder="Short description of this sub-system..."
              value={createDescription}
              onChange={(e) => setCreateDescription(e.target.value)}
            />
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsCreateOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={createLoading}>
              {createLoading ? t('common.loading') : t('common.confirm')}
            </Button>
          </div>
        </form>
      </Modal>

      {/* Edit System Modal */}
      <Modal
        isOpen={!!editingSystem}
        onClose={() => setEditingSystem(null)}
        title={t('systems.edit')}
      >
        {editingSystem && (
          <form onSubmit={handleEditSubmit} className="space-y-4">
            <div>
              <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('systems.slug')}
              </label>
              <Input value={editingSystem.slug} disabled className="font-mono text-xs opacity-70" />
            </div>

            <div>
              <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('systems.name')}
              </label>
              <Input required value={editName} onChange={(e) => setEditName(e.target.value)} />
            </div>

            <div>
              <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('systems.description')}
              </label>
              <Textarea
                rows={3}
                value={editDescription}
                onChange={(e) => setEditDescription(e.target.value)}
              />
            </div>

            <div>
              <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('systems.status')}
              </label>
              <select
                value={editStatus}
                onChange={(e) => setEditStatus(Number(e.target.value))}
                className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
              >
                <option value={1}>{t('systems.active')}</option>
                <option value={0}>{t('systems.disabled')}</option>
              </select>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button type="button" variant="ghost" onClick={() => setEditingSystem(null)}>
                {t('common.cancel')}
              </Button>
              <Button type="submit" disabled={editLoading}>
                {editLoading ? t('common.loading') : t('common.save')}
              </Button>
            </div>
          </form>
        )}
      </Modal>
    </div>
  );
}
