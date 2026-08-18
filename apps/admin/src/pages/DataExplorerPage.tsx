import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, Edit2, Database, Layers, AlertTriangle } from 'lucide-react';
import { Card, Button, Input, Modal, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import { ModelFieldItem, ModelItem, ModelRecordItem, SystemItem } from '../types';

interface DataExplorerProps {
  currentSystem: SystemItem | null;
  onNavigate?: (tab: string) => void;
}

export function DataExplorerPage({ currentSystem, onNavigate }: DataExplorerProps) {
  const { t } = useTranslation();
  const [models, setModels] = useState<ModelItem[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelItem | null>(null);
  const [fields, setFields] = useState<ModelFieldItem[]>([]);
  const [records, setRecords] = useState<ModelRecordItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [page] = useState(1);
  const [total, setTotal] = useState(0);

  // Record Create/Edit Modal
  const [isRecordModalOpen, setIsRecordModalOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<ModelRecordItem | null>(null);
  const [formData, setFormData] = useState<Record<string, any>>({});

  const loadModels = async (slug: string) => {
    try {
      const list = await api.listModels(slug);
      setModels(list);
      if (list.length > 0) {
        setSelectedModel(list[0]);
      } else {
        setSelectedModel(null);
      }
    } catch (err) {
      console.error(err);
      setModels([]);
      setSelectedModel(null);
    }
  };

  const loadRecords = async (slug: string, model: ModelItem) => {
    setLoading(true);
    try {
      const [f, recs] = await Promise.all([
        api.listModelFields(slug, model.id),
        api.listRecords(slug, model.slug, { page, page_size: 15 }),
      ]);
      setFields(f);
      setRecords(recs.items);
      setTotal(recs.pagination.total);
    } catch (err) {
      console.error(err);
      setFields([]);
      setRecords([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (currentSystem?.slug) {
      loadModels(currentSystem.slug);
    } else {
      setModels([]);
      setSelectedModel(null);
      setFields([]);
      setRecords([]);
      setTotal(0);
    }
  }, [currentSystem?.slug]);

  useEffect(() => {
    if (currentSystem?.slug && selectedModel) {
      loadRecords(currentSystem.slug, selectedModel);
    } else {
      setFields([]);
      setRecords([]);
      setTotal(0);
    }
  }, [currentSystem?.slug, selectedModel, page]);

  const openCreateModal = () => {
    setEditingRecord(null);
    const initial: Record<string, any> = {};
    fields.forEach((f) => {
      initial[f.name] = f.default_value !== undefined ? f.default_value : '';
    });
    setFormData(initial);
    setIsRecordModalOpen(true);
  };

  const openEditModal = (rec: ModelRecordItem) => {
    setEditingRecord(rec);
    setFormData({ ...rec.data });
    setIsRecordModalOpen(true);
  };

  const handleDelete = async (id: number) => {
    if (!currentSystem?.slug || !selectedModel) return;
    if (!confirm(t('explorer.delete_confirm'))) return;
    try {
      await api.deleteRecord(currentSystem.slug, selectedModel.slug, id);
      loadRecords(currentSystem.slug, selectedModel);
    } catch (err: any) {
      alert(err.message || 'Delete failed');
    }
  };

  const handleSaveRecord = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug || !selectedModel) return;
    try {
      if (editingRecord) {
        await api.updateRecord(currentSystem.slug, selectedModel.slug, editingRecord.id, formData);
      } else {
        await api.createRecord(currentSystem.slug, selectedModel.slug, formData);
      }
      setIsRecordModalOpen(false);
      loadRecords(currentSystem.slug, selectedModel);
    } catch (err: any) {
      alert(err.message || 'Save failed');
    }
  };

  // If no subsystem is available
  if (!currentSystem) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">{t('explorer.title')}</h1>
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">{t('explorer.desc')}</p>
          </div>
          <Button disabled title={t('explorer.no_system_desc')}>
            <Plus className="w-4 h-4" />
            <span>{t('explorer.create_record')}</span>
          </Button>
        </div>

        <Card className="text-center py-16 px-6 max-w-2xl mx-auto">
          <div className="w-14 h-14 rounded-2xl bg-amber-50 dark:bg-amber-950/60 border border-amber-200 dark:border-amber-800/60 flex items-center justify-center mx-auto mb-4 text-amber-600 dark:text-amber-400">
            <AlertTriangle className="w-7 h-7" />
          </div>
          <h2 className="text-lg font-bold text-slate-900 dark:text-slate-100 mb-2">
            {t('explorer.no_system_title')}
          </h2>
          <p className="text-sm text-slate-500 dark:text-slate-400 max-w-md mx-auto mb-6">
            {t('explorer.no_system_desc')}
          </p>
          {onNavigate && (
            <Button onClick={() => onNavigate('systems')} className="gap-2">
              <Layers className="w-4 h-4" />
              <span>{t('explorer.go_to_systems')}</span>
            </Button>
          )}
        </Card>
      </div>
    );
  }

  // If subsystem exists, but has 0 models
  if (models.length === 0 && !loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">{t('explorer.title')}</h1>
              <Badge variant="success">/{currentSystem.slug}</Badge>
            </div>
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">{t('explorer.desc')}</p>
          </div>
          <Button disabled title={t('explorer.no_models_desc')}>
            <Plus className="w-4 h-4" />
            <span>{t('explorer.create_record')}</span>
          </Button>
        </div>

        <Card className="text-center py-16 px-6 max-w-2xl mx-auto">
          <div className="w-14 h-14 rounded-2xl bg-sky-50 dark:bg-sky-950/60 border border-sky-200 dark:border-sky-800/60 flex items-center justify-center mx-auto mb-4 text-sky-600 dark:text-sky-400">
            <Database className="w-7 h-7" />
          </div>
          <h2 className="text-lg font-bold text-slate-900 dark:text-slate-100 mb-2">
            {t('explorer.no_models_title')}
          </h2>
          <p className="text-sm text-slate-500 dark:text-slate-400 max-w-md mx-auto mb-6">
            {t('explorer.no_models_desc')}
          </p>
          {onNavigate && (
            <Button onClick={() => onNavigate('models')} className="gap-2">
              <Database className="w-4 h-4" />
              <span>{t('explorer.go_to_models')}</span>
            </Button>
          )}
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">{t('explorer.title')}</h1>
            <Badge variant="success">/{currentSystem.slug}</Badge>
          </div>
          <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
            {t('explorer.desc')} ({total} records)
          </p>
        </div>

        {/* Model Dropdown Selector */}
        <div className="flex items-center gap-3">
          <select
            className="bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-800 rounded-lg px-3 py-2 text-sm text-slate-800 dark:text-slate-200 focus:outline-none focus:border-emerald-500"
            value={selectedModel?.slug || ''}
            onChange={(e) => {
              const m = models.find((x) => x.slug === e.target.value);
              if (m) setSelectedModel(m);
            }}
          >
            {models.map((m) => (
              <option key={m.id} value={m.slug}>
                {m.name} ({m.slug})
              </option>
            ))}
          </select>

          {selectedModel && (
            <Button onClick={openCreateModal}>
              <Plus className="w-4 h-4" />
              <span>{t('explorer.create_record')}</span>
            </Button>
          )}
        </div>
      </div>

      {/* Dynamic Data Table */}
      <Card className="p-0 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
            <thead className="bg-slate-50 dark:bg-slate-950/60 text-xs uppercase font-medium text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-800">
              <tr>
                <th className="px-5 py-3 w-16">ID</th>
                {fields.slice(0, 5).map((f) => (
                  <th key={f.id} className="px-5 py-3">
                    {f.label} <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500">({f.name})</span>
                  </th>
                ))}
                <th className="px-5 py-3">Created</th>
                <th className="px-5 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 dark:divide-slate-800/60 text-xs">
              {records.map((rec) => (
                <tr key={rec.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/30 transition">
                  <td className="px-5 py-3 font-mono text-slate-400 dark:text-slate-500">#{rec.id}</td>
                  {fields.slice(0, 5).map((f) => {
                    const val = rec.data[f.name];
                    const displayVal = typeof val === 'object' ? JSON.stringify(val) : String(val ?? '-');
                    return (
                      <td key={f.id} className="px-5 py-3 text-slate-800 dark:text-slate-200 max-w-xs truncate">
                        {displayVal}
                      </td>
                    );
                  })}
                  <td className="px-5 py-3 text-slate-500 dark:text-slate-400 font-mono text-[11px]">
                    {new Date(rec.created_at).toLocaleString()}
                  </td>
                  <td className="px-5 py-3 text-right space-x-2">
                    <button
                      onClick={() => openEditModal(rec)}
                      className="p-1 rounded text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400 transition"
                      title="Edit"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => handleDelete(rec.id)}
                      className="p-1 rounded text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 transition"
                      title="Delete"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
              {records.length === 0 && (
                <tr>
                  <td colSpan={fields.length + 2} className="text-center py-12 text-slate-500 dark:text-slate-400">
                    {loading ? t('common.loading') : t('explorer.no_records')}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Record Edit/Create Modal */}
      <Modal
        isOpen={isRecordModalOpen}
        onClose={() => setIsRecordModalOpen(false)}
        title={editingRecord ? t('explorer.edit_record') : t('explorer.create_record')}
      >
        <form onSubmit={handleSaveRecord} className="space-y-4">
          {fields.map((f) => (
            <div key={f.id}>
              <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                {f.label} <span className="font-mono text-slate-400 dark:text-slate-500">({f.name})</span>
                {f.is_required && <span className="text-rose-600 dark:text-rose-400 ml-1">*</span>}
              </label>
              {f.field_type === 'boolean' ? (
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={!!formData[f.name]}
                    onChange={(e) => setFormData({ ...formData, [f.name]: e.target.checked })}
                    className="rounded bg-white dark:bg-slate-950 border-slate-300 dark:border-slate-800 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span className="text-xs text-slate-700 dark:text-slate-300">
                    {formData[f.name] ? 'True / Yes' : 'False / No'}
                  </span>
                </div>
              ) : f.field_type === 'richtext' ? (
                <textarea
                  rows={3}
                  className="w-full bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-800 rounded-lg px-3.5 py-2 text-sm text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500 focus:outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500"
                  value={formData[f.name] || ''}
                  onChange={(e) => setFormData({ ...formData, [f.name]: e.target.value })}
                />
              ) : (
                <Input
                  required={f.is_required}
                  type={f.field_type === 'integer' || f.field_type === 'number' ? 'number' : 'text'}
                  value={formData[f.name] !== undefined ? formData[f.name] : ''}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      [f.name]:
                        f.field_type === 'integer' || f.field_type === 'number'
                          ? Number(e.target.value)
                          : e.target.value,
                    })
                  }
                />
              )}
            </div>
          ))}

          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsRecordModalOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.save')}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
