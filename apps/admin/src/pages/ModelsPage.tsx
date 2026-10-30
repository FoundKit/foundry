import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Database,
  Plus,
  Layers,
  AlertTriangle,
  Trash2,
  Edit2,
  RotateCcw,
  Link as LinkIcon,
  Download,
  Upload,
} from 'lucide-react';
import { Card, Button, Input, Textarea, Badge, Modal } from '../components/UiWidgets';
import { api } from '../services/api';
import { ModelExportItem, ModelFieldItem, ModelItem, SystemItem } from '../types';

interface ModelsPageProps {
  currentSystem: SystemItem | null;
  queryParams?: Record<string, string>;
  onUpdateParams?: (params: Record<string, any>) => void;
  onNavigate?: (tab: string) => void;
}

export function ModelsPage({
  currentSystem,
  queryParams,
  onUpdateParams,
  onNavigate,
}: ModelsPageProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'active' | 'trash'>('active');
  const [models, setModels] = useState<ModelItem[]>([]);
  const [deletedModels, setDeletedModels] = useState<ModelItem[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelItem | null>(null);
  const [fields, setFields] = useState<ModelFieldItem[]>([]);
  const [loading, setLoading] = useState(false);

  // Multi-selection state
  const [selectedModelIds, setSelectedModelIds] = useState<number[]>([]);

  // Import Models Modal
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [importJsonText, setImportJsonText] = useState('');
  const [importParsedModels, setImportParsedModels] = useState<ModelExportItem[]>([]);
  const [importLoading, setImportLoading] = useState(false);
  const [importError, setImportError] = useState('');

  // Create Model Modal
  const [isCreateModelOpen, setIsCreateModelOpen] = useState(false);
  const [modelSlug, setModelSlug] = useState('');
  const [modelName, setModelName] = useState('');
  const [modelDesc, setModelDesc] = useState('');

  // Edit Model Modal
  const [isEditModelOpen, setIsEditModelOpen] = useState(false);
  const [editModelSlug, setEditModelSlug] = useState('');
  const [editModelName, setEditModelName] = useState('');
  const [editModelDesc, setEditModelDesc] = useState('');

  // Add Field Modal
  const [isAddFieldOpen, setIsAddFieldOpen] = useState(false);
  const [fieldName, setFieldName] = useState('');
  const [fieldLabel, setFieldLabel] = useState('');
  const [fieldType, setFieldType] = useState('string');
  const [isRequired, setIsRequired] = useState(false);
  const [targetSystem, setTargetSystem] = useState('');
  const [targetModel, setTargetModel] = useState('');
  const [targetDisplayField, setTargetDisplayField] = useState('');
  const [targetFields, setTargetFields] = useState<ModelFieldItem[]>([]);

  // Edit Field Modal
  const [isEditFieldOpen, setIsEditFieldOpen] = useState(false);
  const [editingField, setEditingField] = useState<ModelFieldItem | null>(null);
  const [editFieldName, setEditFieldName] = useState('');
  const [editFieldLabel, setEditFieldLabel] = useState('');
  const [editFieldType, setEditFieldType] = useState('string');
  const [editFieldRequired, setEditFieldRequired] = useState(false);
  const [editFieldTargetSystem, setEditFieldTargetSystem] = useState('');
  const [editFieldTargetModel, setEditFieldTargetModel] = useState('');
  const [editTargetDisplayField, setEditTargetDisplayField] = useState('');
  const [editTargetFields, setEditTargetFields] = useState<ModelFieldItem[]>([]);

  const loadModels = async (slug: string) => {
    setLoading(true);
    try {
      const list = await api.listModels(slug);
      setModels(list);

      // Restore selected model from URL query if present
      const targetModelSlug = queryParams?.model;
      const matched = list.find((m) => m.slug === targetModelSlug);

      if (matched) {
        setSelectedModel(matched);
      } else if (list.length > 0) {
        setSelectedModel(list[0]);
        if (onUpdateParams && !targetModelSlug) {
          onUpdateParams({ model: list[0].slug });
        }
      } else {
        setSelectedModel(null);
      }
    } catch (err) {
      console.error(err);
      setModels([]);
      setSelectedModel(null);
    } finally {
      setLoading(false);
    }
  };

  const loadDeletedModels = async (slug: string) => {
    try {
      const list = await api.listDeletedModels(slug);
      setDeletedModels(list);
    } catch (err) {
      console.error(err);
      setDeletedModels([]);
    }
  };

  const loadFields = async (slug: string, modelId: number) => {
    try {
      const f = await api.listModelFields(slug, modelId);
      setFields(f);
    } catch (err) {
      console.error(err);
      setFields([]);
    }
  };

  useEffect(() => {
    if (currentSystem?.slug) {
      loadModels(currentSystem.slug);
      loadDeletedModels(currentSystem.slug);
    } else {
      setModels([]);
      setDeletedModels([]);
      setSelectedModel(null);
      setFields([]);
    }
  }, [currentSystem?.slug]);

  // Sync selectedModel if queryParams.model changes externally
  useEffect(() => {
    if (queryParams?.model && models.length > 0) {
      const found = models.find((m) => m.slug === queryParams.model);
      if (found && found.id !== selectedModel?.id) {
        setSelectedModel(found);
      }
    }
  }, [queryParams?.model, models]);

  useEffect(() => {
    if (currentSystem?.slug && selectedModel) {
      loadFields(currentSystem.slug, selectedModel.id);
    } else {
      setFields([]);
    }
  }, [currentSystem?.slug, selectedModel]);

  const handleSelectModel = (m: ModelItem) => {
    setSelectedModel(m);
    if (onUpdateParams) {
      onUpdateParams({ model: m.slug });
    }
  };

  const handleCreateModel = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug) {
      alert(t('models.no_system_desc'));
      return;
    }
    try {
      const created = await api.createModel(currentSystem.slug, {
        slug: modelSlug.trim(),
        name: modelName.trim(),
        description: modelDesc.trim() || undefined,
      });
      setIsCreateModelOpen(false);
      setModelSlug('');
      setModelName('');
      setModelDesc('');
      await loadModels(currentSystem.slug);
      setSelectedModel(created);
      if (onUpdateParams) {
        onUpdateParams({ model: created.slug });
      }
    } catch (err: any) {
      alert(err.message || 'Failed to create model');
    }
  };

  const openEditModelModal = () => {
    if (!selectedModel) return;
    setEditModelSlug(selectedModel.slug);
    setEditModelName(selectedModel.name);
    setEditModelDesc(selectedModel.description || '');
    setIsEditModelOpen(true);
  };

  const handleUpdateModel = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug || !selectedModel) return;
    try {
      const updated = await api.updateModel(currentSystem.slug, selectedModel.id, {
        slug: editModelSlug.trim(),
        name: editModelName.trim(),
        description: editModelDesc.trim() || undefined,
      });
      setIsEditModelOpen(false);
      await loadModels(currentSystem.slug);
      setSelectedModel(updated);
      if (onUpdateParams) {
        onUpdateParams({ model: updated.slug });
      }
    } catch (err: any) {
      alert(err.message || 'Failed to update model');
    }
  };

  const handleDeleteModel = async () => {
    if (!currentSystem?.slug || !selectedModel) return;
    if (!confirm(t('models.delete_model_confirm'))) return;
    try {
      await api.deleteModel(currentSystem.slug, selectedModel.id);
      await Promise.all([loadModels(currentSystem.slug), loadDeletedModels(currentSystem.slug)]);
    } catch (err: any) {
      alert(err.message || 'Failed to delete model');
    }
  };

  const handleRestoreModel = async (m: ModelItem) => {
    if (!currentSystem?.slug) return;
    if (!confirm(t('models.restore_confirm'))) return;
    try {
      await api.restoreModel(currentSystem.slug, m.id);
      await Promise.all([loadModels(currentSystem.slug), loadDeletedModels(currentSystem.slug)]);
      setActiveTab('active');
      setSelectedModel(m);
      if (onUpdateParams) {
        onUpdateParams({ model: m.slug });
      }
    } catch (err: any) {
      alert(err.message || 'Failed to restore model');
    }
  };

  const handleTargetModelChange = async (slug: string) => {
    setTargetModel(slug);
    setTargetDisplayField('');
    if (!slug || !currentSystem?.slug) {
      setTargetFields([]);
      return;
    }
    const tm = models.find((m) => m.slug === slug);
    if (tm) {
      try {
        const fList = await api.listModelFields(currentSystem.slug, tm.id);
        setTargetFields(fList);
      } catch (err) {
        console.error(err);
        setTargetFields([]);
      }
    }
  };

  const handleEditTargetModelChange = async (slug: string) => {
    setEditFieldTargetModel(slug);
    setEditTargetDisplayField('');
    if (!slug || !currentSystem?.slug) {
      setEditTargetFields([]);
      return;
    }
    const tm = models.find((m) => m.slug === slug);
    if (tm) {
      try {
        const fList = await api.listModelFields(currentSystem.slug, tm.id);
        setEditTargetFields(fList);
      } catch (err) {
        console.error(err);
        setEditTargetFields([]);
      }
    }
  };

  const handleAddField = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug || !selectedModel) return;
    try {
      const options: Record<string, any> = {};
      if (fieldType === 'relation' && targetModel) {
        options.target_system = targetSystem.trim() || currentSystem.slug;
        options.target_model = targetModel;
        if (targetDisplayField) {
          options.display_field = targetDisplayField;
        }
      }

      await api.addModelField(currentSystem.slug, selectedModel.id, {
        name: fieldName.trim(),
        label: fieldLabel.trim(),
        field_type: fieldType,
        is_required: isRequired,
        options,
        sort_order: fields.length + 1,
      });
      setIsAddFieldOpen(false);
      setFieldName('');
      setFieldLabel('');
      setTargetSystem('');
      setTargetModel('');
      setTargetDisplayField('');
      setTargetFields([]);
      loadFields(currentSystem.slug, selectedModel.id);
    } catch (err: any) {
      alert(err.message || 'Failed to add field');
    }
  };

  const openEditFieldModal = (f: ModelFieldItem) => {
    setEditingField(f);
    setEditFieldName(f.name);
    setEditFieldLabel(f.label);
    setEditFieldType(f.field_type);
    setEditFieldRequired(f.is_required);
    const tmSys = f.options?.target_system || currentSystem?.slug || '';
    const tmSlug = f.options?.target_model || '';
    setEditFieldTargetSystem(tmSys);
    setEditFieldTargetModel(tmSlug);
    setEditTargetDisplayField(f.options?.display_field || '');
    if (f.field_type === 'relation' && tmSlug && currentSystem?.slug) {
      const sysToQuery = tmSys || currentSystem.slug;
      api
        .listModels(sysToQuery)
        .then((mList) => {
          const tm = mList.find((m) => m.slug === tmSlug);
          if (tm) {
            api
              .listModelFields(sysToQuery, tm.id)
              .then(setEditTargetFields)
              .catch(() => setEditTargetFields([]));
          } else {
            setEditTargetFields([]);
          }
        })
        .catch(() => setEditTargetFields([]));
    } else {
      setEditTargetFields([]);
    }
    setIsEditFieldOpen(true);
  };

  const handleUpdateField = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug || !selectedModel || !editingField) return;
    try {
      const options: Record<string, any> = { ...(editingField.options || {}) };
      if (editFieldType === 'relation') {
        options.target_system = editFieldTargetSystem.trim() || currentSystem.slug;
        options.target_model = editFieldTargetModel;
        if (editTargetDisplayField) {
          options.display_field = editTargetDisplayField;
        } else {
          delete options.display_field;
        }
      } else {
        delete options.target_system;
        delete options.target_model;
        delete options.display_field;
      }

      await api.updateModelField(currentSystem.slug, selectedModel.id, editingField.id, {
        name: editFieldName.trim(),
        label: editFieldLabel.trim(),
        field_type: editFieldType,
        is_required: editFieldRequired,
        options,
        sort_order: editingField.sort_order,
      });
      setIsEditFieldOpen(false);
      setEditingField(null);
      setEditTargetFields([]);
      loadFields(currentSystem.slug, selectedModel.id);
    } catch (err: any) {
      alert(err.message || 'Failed to update field');
    }
  };

  const handleDeleteField = async (fieldId: number) => {
    if (!currentSystem?.slug || !selectedModel) return;
    if (!confirm(t('models.delete_field_confirm'))) return;
    try {
      await api.deleteModelField(currentSystem.slug, selectedModel.id, fieldId);
      loadFields(currentSystem.slug, selectedModel.id);
    } catch (err: any) {
      alert(err.message || 'Failed to delete field');
    }
  };

  const toggleSelectModel = (id: number) => {
    setSelectedModelIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
    );
  };

  const toggleSelectAll = () => {
    if (selectedModelIds.length === models.length) {
      setSelectedModelIds([]);
    } else {
      setSelectedModelIds(models.map((m) => m.id));
    }
  };

  const handleExportModels = async () => {
    if (!currentSystem?.slug) return;
    if (models.length === 0) {
      alert(t('models.export_empty'));
      return;
    }
    try {
      const exportData = await api.exportModels(
        currentSystem.slug,
        selectedModelIds.length > 0 ? selectedModelIds : undefined,
      );
      const jsonStr = JSON.stringify(exportData, null, 2);
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${currentSystem.slug}-models-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err: any) {
      alert(err.message || 'Failed to export models');
    }
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const content = event.target?.result as string;
      setImportJsonText(content);
      parseImportJson(content);
    };
    reader.readAsText(file);
  };

  const parseImportJson = (text: string) => {
    try {
      const parsed = JSON.parse(text);
      if (!Array.isArray(parsed)) {
        setImportError(t('models.invalid_json'));
        setImportParsedModels([]);
        return;
      }
      setImportError('');
      setImportParsedModels(parsed);
    } catch (err: any) {
      setImportError(err.message || t('models.invalid_json'));
      setImportParsedModels([]);
    }
  };

  const handleConfirmImport = async () => {
    if (!currentSystem?.slug || importParsedModels.length === 0) return;
    setImportLoading(true);
    try {
      const res = await api.importModels(currentSystem.slug, importParsedModels);
      setIsImportModalOpen(false);
      setImportJsonText('');
      setImportParsedModels([]);
      await loadModels(currentSystem.slug);
      alert(t('models.import_success', { count: res.total || importParsedModels.length }));
    } catch (err: any) {
      alert(err.message || 'Import failed');
    } finally {
      setImportLoading(false);
    }
  };

  // If no current system is selected or exists
  if (!currentSystem) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('models.title')}
            </h1>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('models.desc')}</p>
          </div>
          <Button disabled title={t('models.no_system_desc')}>
            <Plus className="h-4 w-4" />
            <span>{t('models.create_model')}</span>
          </Button>
        </div>

        <Card className="mx-auto max-w-2xl px-6 py-16 text-center">
          <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl border border-amber-200 bg-amber-50 text-amber-600 dark:border-amber-800/60 dark:bg-amber-950/60 dark:text-amber-400">
            <AlertTriangle className="h-7 w-7" />
          </div>
          <h2 className="mb-2 text-lg font-bold text-slate-900 dark:text-slate-100">
            {t('models.no_system_title')}
          </h2>
          <p className="mx-auto mb-6 max-w-md text-sm text-slate-500 dark:text-slate-400">
            {t('models.no_system_desc')}
          </p>
          {onNavigate && (
            <Button onClick={() => onNavigate('systems')} className="gap-2">
              <Layers className="h-4 w-4" />
              <span>{t('models.go_to_systems')}</span>
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
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('models.title')}
            </h1>
            <Badge variant="success">/{currentSystem.slug}</Badge>
          </div>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('models.desc')}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={handleExportModels}
            title={
              selectedModelIds.length > 0
                ? t('models.export_selected', { count: selectedModelIds.length })
                : t('models.export_models')
            }
            className="gap-1.5"
          >
            <Download className="h-4 w-4" />
            <span>
              {selectedModelIds.length > 0
                ? t('models.export_selected', { count: selectedModelIds.length })
                : t('models.export_models')}
            </span>
          </Button>
          <Button
            variant="secondary"
            onClick={() => setIsImportModalOpen(true)}
            className="gap-1.5"
          >
            <Upload className="h-4 w-4" />
            <span>{t('models.import_models')}</span>
          </Button>
          <Button onClick={() => setIsCreateModelOpen(true)} className="gap-1.5">
            <Plus className="h-4 w-4" />
            <span>{t('models.create_model')}</span>
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
        {/* Left: Models List & Trash Tab */}
        <div className="space-y-3">
          <div className="flex items-center gap-1 border-b border-slate-200 pb-2 dark:border-slate-800">
            <button
              onClick={() => setActiveTab('active')}
              className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition ${
                activeTab === 'active'
                  ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300'
                  : 'text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
              }`}
            >
              {t('models.active_models_tab')} ({models.length})
            </button>
            <button
              onClick={() => setActiveTab('trash')}
              className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition ${
                activeTab === 'trash'
                  ? 'bg-rose-50 text-rose-700 dark:bg-rose-950/60 dark:text-rose-300'
                  : 'text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
              }`}
            >
              {t('models.deleted_models_tab')} ({deletedModels.length})
            </button>
          </div>

          {activeTab === 'active' && (
            <div className="space-y-2">
              {models.length > 0 && (
                <div className="flex items-center justify-between rounded-lg bg-slate-50 px-3 py-1.5 text-xs text-slate-500 dark:bg-slate-900/60 dark:text-slate-400">
                  <label className="flex cursor-pointer select-none items-center gap-2">
                    <input
                      type="checkbox"
                      checked={models.length > 0 && selectedModelIds.length === models.length}
                      onChange={toggleSelectAll}
                      className="cursor-pointer rounded border-slate-300 text-emerald-600 focus:ring-emerald-500 dark:border-slate-700 dark:bg-slate-800"
                    />
                    <span className="font-medium text-slate-600 dark:text-slate-300">
                      {t('models.select_all')}
                    </span>
                  </label>
                  {selectedModelIds.length > 0 && (
                    <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                      {t('models.selected_count', { count: selectedModelIds.length })}
                    </span>
                  )}
                </div>
              )}

              {models.map((m) => {
                const isSelected = selectedModel?.id === m.id;
                const isChecked = selectedModelIds.includes(m.id);
                return (
                  <Card
                    key={m.id}
                    className={`cursor-pointer p-4 transition ${
                      isSelected
                        ? 'border-emerald-500 bg-emerald-50/40 shadow-md shadow-emerald-500/5 ring-1 ring-emerald-500/30 dark:bg-slate-900/90'
                        : 'bg-white hover:border-slate-300 dark:bg-slate-900/40 dark:hover:border-slate-700'
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <div
                        className="flex cursor-pointer items-center pt-0.5"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <input
                          type="checkbox"
                          checked={isChecked}
                          onChange={() => toggleSelectModel(m.id)}
                          className="cursor-pointer rounded border-slate-300 text-emerald-600 focus:ring-emerald-500 dark:border-slate-700 dark:bg-slate-800"
                        />
                      </div>
                      <div className="min-w-0 flex-1" onClick={() => handleSelectModel(m)}>
                        <div className="flex items-center justify-between">
                          <span className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                            {m.name}
                          </span>
                          <Badge variant="info">Zero-DDL</Badge>
                        </div>
                        <div className="mt-1 font-mono text-xs text-emerald-600 dark:text-emerald-400">
                          /{m.slug}
                        </div>
                        <div className="mt-2 truncate text-[11px] text-slate-500 dark:text-slate-400">
                          {m.description || 'Auto-CRUD collection'}
                        </div>
                      </div>
                    </div>
                  </Card>
                );
              })}
              {models.length === 0 && !loading && (
                <div className="rounded-xl border border-dashed border-slate-300 p-6 text-center text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
                  {t('models.empty_models')}
                </div>
              )}
            </div>
          )}

          {activeTab === 'trash' && (
            <div className="space-y-2">
              {deletedModels.map((m) => (
                <Card key={m.id} className="bg-white/80 p-4 opacity-90 dark:bg-slate-900/60">
                  <div className="flex items-center justify-between">
                    <div>
                      <span className="text-sm font-semibold text-slate-700 line-through dark:text-slate-300">
                        {m.name}
                      </span>
                      <div className="mt-0.5 font-mono text-xs text-slate-400 dark:text-slate-500">
                        /{m.slug}
                      </div>
                    </div>
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => handleRestoreModel(m)}
                      className="gap-1 text-xs"
                      title={t('models.restore')}
                    >
                      <RotateCcw className="h-3.5 w-3.5" />
                      <span>{t('models.restore')}</span>
                    </Button>
                  </div>
                  {m.deleted_at && (
                    <div className="mt-2 text-[10px] text-rose-500/80">
                      {t('models.deleted_at')}: {new Date(m.deleted_at).toLocaleString()}
                    </div>
                  )}
                </Card>
              ))}
              {deletedModels.length === 0 && (
                <div className="rounded-xl border border-dashed border-slate-300 p-6 text-center text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
                  {t('models.empty_deleted_models')}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Right: Selected Model Schema Builder */}
        <div className="space-y-4 md:col-span-2">
          {selectedModel && activeTab === 'active' ? (
            <>
              <div className="flex flex-col gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="text-base font-bold text-slate-900 dark:text-slate-100">
                      {selectedModel.name}
                    </h3>
                    <button
                      onClick={openEditModelModal}
                      className="rounded p-1 text-slate-400 transition hover:text-emerald-600 dark:hover:text-emerald-400"
                      title={t('models.edit_model')}
                    >
                      <Edit2 className="h-3.5 w-3.5" />
                    </button>
                    <button
                      onClick={handleDeleteModel}
                      className="rounded p-1 text-slate-400 transition hover:text-rose-600 dark:hover:text-rose-400"
                      title={t('models.delete_model')}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  </div>
                  <p className="mt-0.5 font-mono text-xs text-slate-500 dark:text-slate-400">
                    Endpoint:{' '}
                    <code className="text-emerald-600 dark:text-emerald-400">
                      /api/s/{currentSystem.slug}/{selectedModel.slug}
                    </code>
                  </p>
                  {selectedModel.description && (
                    <p className="mt-1 text-xs text-slate-600 dark:text-slate-400">
                      {selectedModel.description}
                    </p>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <Button size="sm" onClick={() => setIsAddFieldOpen(true)}>
                    <Plus className="h-4 w-4" />
                    <span>{t('models.add_field')}</span>
                  </Button>
                </div>
              </div>

              {/* Field Schema Table */}
              <Card className="overflow-hidden p-0">
                <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
                  <thead className="border-b border-slate-200 bg-slate-50 text-xs font-medium uppercase text-slate-500 dark:border-slate-800 dark:bg-slate-950/60 dark:text-slate-400">
                    <tr>
                      <th className="px-5 py-3">{t('models.field_name')}</th>
                      <th className="px-5 py-3">{t('models.field_label')}</th>
                      <th className="px-5 py-3">{t('models.field_type')}</th>
                      <th className="px-5 py-3">{t('models.required')}</th>
                      <th className="px-5 py-3 text-right">{t('common.actions')}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200 text-xs dark:divide-slate-800/60">
                    {fields.map((f) => (
                      <tr
                        key={f.id}
                        className="transition hover:bg-slate-50 dark:hover:bg-slate-800/30"
                      >
                        <td className="px-5 py-3 font-mono font-medium text-emerald-600 dark:text-emerald-400">
                          {f.name}
                        </td>
                        <td className="px-5 py-3 text-slate-800 dark:text-slate-200">{f.label}</td>
                        <td className="px-5 py-3">
                          <div className="flex items-center gap-1.5">
                            <Badge variant={f.field_type === 'relation' ? 'info' : 'purple'}>
                              {f.field_type}
                            </Badge>
                            {f.field_type === 'relation' && f.options?.target_model && (
                              <span className="flex items-center gap-1 font-mono text-[11px] text-sky-600 dark:text-sky-400">
                                <LinkIcon className="h-3 w-3" />
                                {f.options.target_system &&
                                f.options.target_system !== currentSystem.slug
                                  ? `${f.options.target_system}/${f.options.target_model}`
                                  : f.options.target_model}
                                {f.options.display_field && ` (${f.options.display_field})`}
                              </span>
                            )}
                          </div>
                        </td>
                        <td className="px-5 py-3">
                          {f.is_required ? (
                            <span className="font-semibold text-rose-600 dark:text-rose-400">
                              Yes
                            </span>
                          ) : (
                            <span className="text-slate-400 dark:text-slate-500">Optional</span>
                          )}
                        </td>
                        <td className="space-x-2 px-5 py-3 text-right">
                          <button
                            onClick={() => openEditFieldModal(f)}
                            className="rounded p-1 text-slate-400 transition hover:text-emerald-600 dark:hover:text-emerald-400"
                            title={t('models.edit_field')}
                          >
                            <Edit2 className="h-3.5 w-3.5" />
                          </button>
                          <button
                            onClick={() => handleDeleteField(f.id)}
                            className="rounded p-1 text-slate-400 transition hover:text-rose-600 dark:hover:text-rose-400"
                            title={t('models.delete_field')}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </button>
                        </td>
                      </tr>
                    ))}
                    {fields.length === 0 && (
                      <tr>
                        <td
                          colSpan={5}
                          className="py-8 text-center text-slate-500 dark:text-slate-400"
                        >
                          No fields added yet. Add your first field to configure model schema.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </Card>
            </>
          ) : (
            <Card className="py-16 text-center text-slate-400 dark:text-slate-500">
              <Database className="mx-auto mb-2 h-10 w-10 text-slate-300 dark:text-slate-700" />
              <p>Select a data model to view and configure its fields.</p>
            </Card>
          )}
        </div>
      </div>

      {/* Modal: Create Model */}
      <Modal
        isOpen={isCreateModelOpen}
        onClose={() => setIsCreateModelOpen(false)}
        title={t('models.create_model')}
      >
        <form onSubmit={handleCreateModel} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.model_slug')} (e.g. products, articles)
            </label>
            <Input
              required
              placeholder="products"
              value={modelSlug}
              onChange={(e) => setModelSlug(e.target.value.toLowerCase())}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.model_name')}
            </label>
            <Input
              required
              placeholder="Product Catalog"
              value={modelName}
              onChange={(e) => setModelName(e.target.value)}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Description
            </label>
            <Textarea
              rows={2}
              placeholder="Optional description..."
              value={modelDesc}
              onChange={(e) => setModelDesc(e.target.value)}
            />
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsCreateModelOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>

      {/* Modal: Edit Model */}
      <Modal
        isOpen={isEditModelOpen}
        onClose={() => setIsEditModelOpen(false)}
        title={t('models.edit_model')}
      >
        <form onSubmit={handleUpdateModel} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.model_slug')}
            </label>
            <Input
              required
              placeholder="products"
              value={editModelSlug}
              onChange={(e) => setEditModelSlug(e.target.value.toLowerCase())}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.model_name')}
            </label>
            <Input
              required
              placeholder="Product Catalog"
              value={editModelName}
              onChange={(e) => setEditModelName(e.target.value)}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Description
            </label>
            <Textarea
              rows={2}
              placeholder="Optional description..."
              value={editModelDesc}
              onChange={(e) => setEditModelDesc(e.target.value)}
            />
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsEditModelOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>

      {/* Modal: Add Field */}
      <Modal
        isOpen={isAddFieldOpen}
        onClose={() => setIsAddFieldOpen(false)}
        title={t('models.add_field')}
      >
        <form onSubmit={handleAddField} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_name')} (e.g. price, title, user_id)
            </label>
            <Input
              required
              placeholder="price"
              value={fieldName}
              onChange={(e) => setFieldName(e.target.value.toLowerCase())}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_label')}
            </label>
            <Input
              required
              placeholder="Unit Price"
              value={fieldLabel}
              onChange={(e) => setFieldLabel(e.target.value)}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_type')}
            </label>
            <select
              className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
              value={fieldType}
              onChange={(e) => setFieldType(e.target.value)}
            >
              <option value="string">String (Single line text)</option>
              <option value="richtext">Richtext / HTML</option>
              <option value="image">Image (CDN URL)</option>
              <option value="file">File (Asset URL)</option>
              <option value="integer">Integer (Number)</option>
              <option value="number">Float / Decimal</option>
              <option value="boolean">Boolean (True/False)</option>
              <option value="datetime">DateTime</option>
              <option value="array">Array / List</option>
              <option value="relation">Relation (Foreign Model Record ID)</option>
            </select>
          </div>

          {fieldType === 'relation' && (
            <div className="space-y-4">
              <div>
                <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('models.target_system')}
                </label>
                <Input
                  placeholder={currentSystem.slug}
                  value={targetSystem}
                  onChange={(e) => setTargetSystem(e.target.value)}
                />
                <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                  {t('models.target_system_hint')}
                </p>
              </div>

              <div>
                <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('models.target_model')}
                </label>
                <select
                  required
                  className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
                  value={targetModel}
                  onChange={(e) => handleTargetModelChange(e.target.value)}
                >
                  <option value="">{t('models.select_target_model')}</option>
                  {models
                    .filter((m) => m.id !== selectedModel?.id)
                    .map((m) => (
                      <option key={m.id} value={m.slug}>
                        {m.name} ({m.slug})
                      </option>
                    ))}
                </select>
                <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                  {t('models.target_model_hint')}
                </p>
              </div>

              {targetModel && (
                <div>
                  <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                    {t('models.target_display_field')}
                  </label>
                  <select
                    className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
                    value={targetDisplayField}
                    onChange={(e) => setTargetDisplayField(e.target.value)}
                  >
                    <option value="">{t('models.select_target_display_field')}</option>
                    {targetFields.map((tf) => (
                      <option key={tf.id} value={tf.name}>
                        {tf.label} ({tf.name})
                      </option>
                    ))}
                  </select>
                  <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                    {t('models.target_display_field_hint')}
                  </p>
                </div>
              )}
            </div>
          )}

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="is_required_chk"
              checked={isRequired}
              onChange={(e) => setIsRequired(e.target.checked)}
              className="cursor-pointer rounded border-slate-300 bg-white text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-950"
            />
            <label
              htmlFor="is_required_chk"
              className="cursor-pointer text-xs text-slate-700 dark:text-slate-300"
            >
              {t('models.required')}
            </label>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsAddFieldOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>

      {/* Modal: Edit Field */}
      <Modal
        isOpen={isEditFieldOpen}
        onClose={() => setIsEditFieldOpen(false)}
        title={t('models.edit_field')}
      >
        <form onSubmit={handleUpdateField} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_name')}
            </label>
            <Input
              required
              value={editFieldName}
              onChange={(e) => setEditFieldName(e.target.value.toLowerCase())}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_label')}
            </label>
            <Input
              required
              value={editFieldLabel}
              onChange={(e) => setEditFieldLabel(e.target.value)}
            />
          </div>
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('models.field_type')}
            </label>
            <select
              className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
              value={editFieldType}
              onChange={(e) => setEditFieldType(e.target.value)}
            >
              <option value="string">String (Single line text)</option>
              <option value="richtext">Richtext / HTML</option>
              <option value="image">Image (CDN URL)</option>
              <option value="file">File (Asset URL)</option>
              <option value="integer">Integer (Number)</option>
              <option value="number">Float / Decimal</option>
              <option value="boolean">Boolean (True/False)</option>
              <option value="datetime">DateTime</option>
              <option value="array">Array / List</option>
              <option value="relation">Relation (Foreign Model Record ID)</option>
            </select>
          </div>

          {editFieldType === 'relation' && (
            <div className="space-y-4">
              <div>
                <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('models.target_system')}
                </label>
                <Input
                  placeholder={currentSystem.slug}
                  value={editFieldTargetSystem}
                  onChange={(e) => setEditFieldTargetSystem(e.target.value)}
                />
                <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                  {t('models.target_system_hint')}
                </p>
              </div>

              <div>
                <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('models.target_model')}
                </label>
                <select
                  required
                  className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
                  value={editFieldTargetModel}
                  onChange={(e) => handleEditTargetModelChange(e.target.value)}
                >
                  <option value="">{t('models.select_target_model')}</option>
                  {models
                    .filter((m) => m.id !== selectedModel?.id)
                    .map((m) => (
                      <option key={m.id} value={m.slug}>
                        {m.name} ({m.slug})
                      </option>
                    ))}
                </select>
                <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                  {t('models.target_model_hint')}
                </p>
              </div>

              {editFieldTargetModel && (
                <div>
                  <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
                    {t('models.target_display_field')}
                  </label>
                  <select
                    className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
                    value={editTargetDisplayField}
                    onChange={(e) => setEditTargetDisplayField(e.target.value)}
                  >
                    <option value="">{t('models.select_target_display_field')}</option>
                    {editTargetFields.map((tf) => (
                      <option key={tf.id} value={tf.name}>
                        {tf.label} ({tf.name})
                      </option>
                    ))}
                  </select>
                  <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                    {t('models.target_display_field_hint')}
                  </p>
                </div>
              )}
            </div>
          )}

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="is_edit_required_chk"
              checked={editFieldRequired}
              onChange={(e) => setEditFieldRequired(e.target.checked)}
              className="cursor-pointer rounded border-slate-300 bg-white text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-950"
            />
            <label
              htmlFor="is_edit_required_chk"
              className="cursor-pointer text-xs text-slate-700 dark:text-slate-300"
            >
              {t('models.required')}
            </label>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsEditFieldOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>

      {/* Modal: Import Models */}
      <Modal
        isOpen={isImportModalOpen}
        onClose={() => {
          setIsImportModalOpen(false);
          setImportJsonText('');
          setImportParsedModels([]);
          setImportError('');
        }}
        title={t('models.import_modal_title')}
      >
        <div className="space-y-4">
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t('models.import_modal_desc')}
          </p>

          <div>
            <label className="mb-1.5 block text-xs font-semibold text-slate-700 dark:text-slate-300">
              {t('models.select_json_file')}
            </label>
            <input
              type="file"
              accept=".json,application/json"
              onChange={handleFileUpload}
              className="block w-full text-xs text-slate-500 file:mr-3 file:rounded-lg file:border-0 file:bg-emerald-50 file:px-3 file:py-1.5 file:text-xs file:font-semibold file:text-emerald-700 hover:file:bg-emerald-100 dark:file:bg-emerald-950/60 dark:file:text-emerald-300"
            />
          </div>

          <div>
            <label className="mb-1.5 block text-xs font-semibold text-slate-700 dark:text-slate-300">
              {t('models.or_paste_json')}
            </label>
            <Textarea
              rows={6}
              value={importJsonText}
              onChange={(e) => {
                setImportJsonText(e.target.value);
                parseImportJson(e.target.value);
              }}
              placeholder='[ { "slug": "articles", "name": "文章列表", ... } ]'
              className="font-mono text-xs"
            />
          </div>

          {importError && (
            <div className="rounded-lg border border-rose-200 bg-rose-50 p-2.5 text-xs text-rose-700 dark:border-rose-800 dark:bg-rose-950/40 dark:text-rose-300">
              {importError}
            </div>
          )}

          {importParsedModels.length > 0 && !importError && (
            <div className="rounded-lg border border-emerald-200 bg-emerald-50/70 p-3 dark:border-emerald-800/60 dark:bg-emerald-950/40">
              <div className="flex items-center gap-1.5 text-xs font-semibold text-emerald-800 dark:text-emerald-300">
                <Database className="h-4 w-4" />
                <span>{t('models.import_preview', { count: importParsedModels.length })}</span>
              </div>
              <ul className="mt-2 max-h-36 space-y-1 overflow-y-auto text-[11px] text-slate-600 dark:text-slate-300">
                {importParsedModels.map((item, idx) => (
                  <li
                    key={idx}
                    className="flex items-center justify-between border-b border-emerald-100/50 py-0.5 dark:border-emerald-900/40"
                  >
                    <span className="font-medium text-slate-900 dark:text-slate-100">
                      {item.name || item.slug}{' '}
                      <code className="font-mono text-emerald-600 dark:text-emerald-400">
                        /{item.slug}
                      </code>
                    </span>
                    <span className="text-slate-400">
                      {item.fields?.length || 0} {t('models.fields_count')}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                setIsImportModalOpen(false);
                setImportJsonText('');
                setImportParsedModels([]);
                setImportError('');
              }}
            >
              {t('common.cancel')}
            </Button>
            <Button
              type="button"
              disabled={importLoading || importParsedModels.length === 0 || !!importError}
              onClick={handleConfirmImport}
              className="gap-1.5"
            >
              {importLoading && (
                <div className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-white border-t-transparent" />
              )}
              <span>{t('models.confirm_import')}</span>
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
