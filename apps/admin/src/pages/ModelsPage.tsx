import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Database, Plus, Layers, AlertTriangle } from 'lucide-react';
import { Card, Button, Input, Textarea, Badge, Modal } from '../components/UiWidgets';
import { api } from '../services/api';
import { ModelFieldItem, ModelItem, SystemItem } from '../types';

interface ModelsPageProps {
  currentSystem: SystemItem | null;
  onNavigate?: (tab: string) => void;
}

export function ModelsPage({ currentSystem, onNavigate }: ModelsPageProps) {
  const { t } = useTranslation();
  const [models, setModels] = useState<ModelItem[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelItem | null>(null);
  const [fields, setFields] = useState<ModelFieldItem[]>([]);
  const [loading, setLoading] = useState(false);

  // Create Model Modal
  const [isCreateModelOpen, setIsCreateModelOpen] = useState(false);
  const [modelSlug, setModelSlug] = useState('');
  const [modelName, setModelName] = useState('');
  const [modelDesc, setModelDesc] = useState('');

  // Add Field Modal
  const [isAddFieldOpen, setIsAddFieldOpen] = useState(false);
  const [fieldName, setFieldName] = useState('');
  const [fieldLabel, setFieldLabel] = useState('');
  const [fieldType, setFieldType] = useState('string');
  const [isRequired, setIsRequired] = useState(false);

  const loadModels = async (slug: string) => {
    setLoading(true);
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
    } finally {
      setLoading(false);
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
    } else {
      setModels([]);
      setSelectedModel(null);
      setFields([]);
    }
  }, [currentSystem?.slug]);

  useEffect(() => {
    if (currentSystem?.slug && selectedModel) {
      loadFields(currentSystem.slug, selectedModel.id);
    } else {
      setFields([]);
    }
  }, [currentSystem?.slug, selectedModel]);

  const handleCreateModel = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug) {
      alert(t('models.no_system_desc'));
      return;
    }
    try {
      const created = await api.createModel(currentSystem.slug, {
        slug: modelSlug,
        name: modelName,
        description: modelDesc,
      });
      setIsCreateModelOpen(false);
      setModelSlug('');
      setModelName('');
      setModelDesc('');
      await loadModels(currentSystem.slug);
      setSelectedModel(created);
    } catch (err: any) {
      alert(err.message || 'Failed to create model');
    }
  };

  const handleAddField = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug || !selectedModel) return;
    try {
      await api.addModelField(currentSystem.slug, selectedModel.id, {
        name: fieldName,
        label: fieldLabel,
        field_type: fieldType,
        is_required: isRequired,
        sort_order: fields.length + 1,
      });
      setIsAddFieldOpen(false);
      setFieldName('');
      setFieldLabel('');
      loadFields(currentSystem.slug, selectedModel.id);
    } catch (err: any) {
      alert(err.message || 'Failed to add field');
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
        <Button onClick={() => setIsCreateModelOpen(true)}>
          <Plus className="h-4 w-4" />
          <span>{t('models.create_model')}</span>
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
        {/* Left: Models List */}
        <div className="space-y-2">
          <div className="px-1 text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            Data Models ({models.length})
          </div>
          {models.map((m) => {
            const isSelected = selectedModel?.id === m.id;
            return (
              <Card
                key={m.id}
                className={`cursor-pointer p-4 transition ${
                  isSelected
                    ? 'border-emerald-500 bg-emerald-50/40 shadow-md shadow-emerald-500/5 ring-1 ring-emerald-500/30 dark:bg-slate-900/90'
                    : 'bg-white hover:border-slate-300 dark:bg-slate-900/40 dark:hover:border-slate-700'
                }`}
              >
                <div onClick={() => setSelectedModel(m)}>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-semibold text-slate-900 dark:text-slate-100">
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
              </Card>
            );
          })}
          {models.length === 0 && !loading && (
            <div className="rounded-xl border border-dashed border-slate-300 p-6 text-center text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
              {t('models.empty_models')}
            </div>
          )}
        </div>

        {/* Right: Selected Model Schema Builder */}
        <div className="space-y-4 md:col-span-2">
          {selectedModel ? (
            <>
              <div className="flex items-center justify-between rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900">
                <div>
                  <h3 className="text-base font-bold text-slate-900 dark:text-slate-100">
                    {selectedModel.name} Schema
                  </h3>
                  <p className="mt-0.5 font-mono text-xs text-slate-500 dark:text-slate-400">
                    Endpoint:{' '}
                    <code className="text-emerald-600 dark:text-emerald-400">
                      /api/v1/s/{currentSystem.slug}/{selectedModel.slug}
                    </code>
                  </p>
                </div>
                <Button size="sm" onClick={() => setIsAddFieldOpen(true)}>
                  <Plus className="h-4 w-4" />
                  <span>{t('models.add_field')}</span>
                </Button>
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
                          <Badge variant="purple">{f.field_type}</Badge>
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
                      </tr>
                    ))}
                    {fields.length === 0 && (
                      <tr>
                        <td
                          colSpan={4}
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

      {/* Modal: Add Field */}
      <Modal
        isOpen={isAddFieldOpen}
        onClose={() => setIsAddFieldOpen(false)}
        title={t('models.add_field')}
      >
        <form onSubmit={handleAddField} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Field Key (e.g. price, title, cover_image)
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
              Field Display Label
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
              Field Type
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
              <option value="relation">Relation (Foreign Model Record)</option>
            </select>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="is_required_chk"
              checked={isRequired}
              onChange={(e) => setIsRequired(e.target.checked)}
              className="rounded border-slate-300 bg-white text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-950"
            />
            <label htmlFor="is_required_chk" className="text-xs text-slate-700 dark:text-slate-300">
              Required field
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
    </div>
  );
}
