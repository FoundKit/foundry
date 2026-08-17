import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, Edit2 } from 'lucide-react';
import { Card, Button, Input, Modal } from '../components/UiWidgets';
import { api } from '../services/api';
import { ModelFieldItem, ModelItem, ModelRecordItem, SystemItem } from '../types';

interface DataExplorerProps {
  currentSystem: SystemItem | null;
}

export function DataExplorerPage({ currentSystem }: DataExplorerProps) {
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

  const systemSlug = currentSystem?.slug || 'carnival_demo';

  const loadModels = async () => {
    try {
      const list = await api.listModels(systemSlug);
      setModels(list);
      if (list.length > 0 && !selectedModel) {
        setSelectedModel(list[0]);
      }
    } catch (err) {
      console.error(err);
    }
  };

  const loadRecords = async () => {
    if (!selectedModel) return;
    setLoading(true);
    try {
      const [f, recs] = await Promise.all([
        api.listModelFields(systemSlug, selectedModel.id),
        api.listRecords(systemSlug, selectedModel.slug, { page, page_size: 15 }),
      ]);
      setFields(f);
      setRecords(recs.items);
      setTotal(recs.pagination.total);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadModels();
  }, [systemSlug]);

  useEffect(() => {
    if (selectedModel) {
      loadRecords();
    }
  }, [selectedModel, page]);

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
    if (!selectedModel) return;
    if (!confirm(t('explorer.delete_confirm'))) return;
    try {
      await api.deleteRecord(systemSlug, selectedModel.slug, id);
      loadRecords();
    } catch (err: any) {
      alert(err.message || 'Delete failed');
    }
  };

  const handleSaveRecord = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedModel) return;
    try {
      if (editingRecord) {
        await api.updateRecord(systemSlug, selectedModel.slug, editingRecord.id, formData);
      } else {
        await api.createRecord(systemSlug, selectedModel.slug, formData);
      }
      setIsRecordModalOpen(false);
      loadRecords();
    } catch (err: any) {
      alert(err.message || 'Save failed');
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-slate-100">{t('explorer.title')}</h1>
          <p className="text-xs text-slate-400 mt-1">
            {t('explorer.desc')} ({total} records)
          </p>
        </div>

        {/* Model Dropdown Selector */}
        <div className="flex items-center gap-3">
          <select
            className="bg-slate-900 border border-slate-800 rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-emerald-500"
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
          <table className="w-full text-left text-sm text-slate-300">
            <thead className="bg-slate-950/60 text-xs uppercase font-medium text-slate-400 border-b border-slate-800">
              <tr>
                <th className="px-5 py-3 w-16">ID</th>
                {fields.slice(0, 5).map((f) => (
                  <th key={f.id} className="px-5 py-3">
                    {f.label} <span className="font-mono text-[10px] text-slate-500">({f.name})</span>
                  </th>
                ))}
                <th className="px-5 py-3">Created</th>
                <th className="px-5 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-xs">
              {records.map((rec) => (
                <tr key={rec.id} className="hover:bg-slate-800/30 transition">
                  <td className="px-5 py-3 font-mono text-slate-500">#{rec.id}</td>
                  {fields.slice(0, 5).map((f) => {
                    const val = rec.data[f.name];
                    const displayVal = typeof val === 'object' ? JSON.stringify(val) : String(val ?? '-');
                    return (
                      <td key={f.id} className="px-5 py-3 text-slate-200 max-w-xs truncate">
                        {displayVal}
                      </td>
                    );
                  })}
                  <td className="px-5 py-3 text-slate-400 font-mono text-[11px]">
                    {new Date(rec.created_at).toLocaleString()}
                  </td>
                  <td className="px-5 py-3 text-right space-x-2">
                    <button
                      onClick={() => openEditModal(rec)}
                      className="p-1 rounded text-slate-400 hover:text-emerald-400 transition"
                      title="Edit"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => handleDelete(rec.id)}
                      className="p-1 rounded text-slate-400 hover:text-rose-400 transition"
                      title="Delete"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </td>
                </tr>
              ))}
              {records.length === 0 && (
                <tr>
                  <td colSpan={fields.length + 2} className="text-center py-12 text-slate-500">
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
              <label className="block text-xs font-medium text-slate-300 mb-1">
                {f.label} <span className="font-mono text-slate-500">({f.name})</span>
                {f.is_required && <span className="text-rose-400 ml-1">*</span>}
              </label>
              {f.field_type === 'boolean' ? (
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={!!formData[f.name]}
                    onChange={(e) => setFormData({ ...formData, [f.name]: e.target.checked })}
                    className="rounded bg-slate-950 border-slate-800 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span className="text-xs text-slate-300">
                    {formData[f.name] ? 'True / Yes' : 'False / No'}
                  </span>
                </div>
              ) : f.field_type === 'richtext' ? (
                <textarea
                  rows={3}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3.5 py-2 text-sm text-slate-100"
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
