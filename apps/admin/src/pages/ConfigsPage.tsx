import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Sliders,
  Plus,
  Save,
  CheckCircle2,
  Image as ImageIcon,
  Calendar,
  Hash,
  Layers,
  AlertTriangle,
} from 'lucide-react';
import { Card, Button, Input, Textarea, Badge, Modal } from '../components/UiWidgets';
import { api } from '../services/api';
import { SystemConfigItem, SystemItem } from '../types';

interface ConfigsPageProps {
  currentSystem: SystemItem | null;
  onNavigate?: (tab: string) => void;
}

export function ConfigsPage({ currentSystem, onNavigate }: ConfigsPageProps) {
  const { t } = useTranslation();
  const [configs, setConfigs] = useState<SystemConfigItem[]>([]);
  const [formValues, setFormValues] = useState<Record<string, any>>({});
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [successMsg, setSuccessMsg] = useState(false);
  const [isAddOpen, setIsAddOpen] = useState(false);

  // New field form state
  const [newKey, setNewKey] = useState('');
  const [newLabel, setNewLabel] = useState('');
  const [newType, setNewType] = useState<string>('string');

  const loadData = async (slug: string) => {
    setLoading(true);
    try {
      const [schema, values] = await Promise.all([
        api.listConfigSchema(slug),
        api.getAggregatedConfigs(slug),
      ]);
      setConfigs(schema);
      setFormValues(values || {});
    } catch (err) {
      console.error(err);
      setConfigs([]);
      setFormValues({});
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (currentSystem?.slug) {
      loadData(currentSystem.slug);
    } else {
      setConfigs([]);
      setFormValues({});
    }
  }, [currentSystem?.slug]);

  const handleSave = async () => {
    if (!currentSystem?.slug) return;
    setSaving(true);
    try {
      await api.updateAggregatedConfigs(currentSystem.slug, formValues);
      setSuccessMsg(true);
      setTimeout(() => setSuccessMsg(false), 3000);
    } catch (err: any) {
      alert(err.message || 'Failed to save configs');
    } finally {
      setSaving(false);
    }
  };

  const handleAddField = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentSystem?.slug) return;
    try {
      await api.upsertConfigSchema(currentSystem.slug, {
        key: newKey,
        label: newLabel,
        value_type: newType,
        options: {},
        sort_order: configs.length + 1,
      });
      setIsAddOpen(false);
      setNewKey('');
      setNewLabel('');
      loadData(currentSystem.slug);
    } catch (err: any) {
      alert(err.message || 'Failed to add config field');
    }
  };

  const updateValue = (key: string, val: any) => {
    setFormValues((prev) => ({ ...prev, [key]: val }));
  };

  const renderWidget = (cfg: SystemConfigItem) => {
    const val = formValues[cfg.key];

    switch (cfg.value_type) {
      case 'image':
        return (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <ImageIcon className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
              <Input
                placeholder="https://example.com/banner.jpg"
                value={val || ''}
                onChange={(e) => updateValue(cfg.key, e.target.value)}
              />
            </div>
            {val && (
              <div className="mt-2 flex h-24 w-48 items-center justify-center overflow-hidden rounded-lg border border-slate-200 bg-slate-100 dark:border-slate-800 dark:bg-slate-950">
                <img src={val} alt="Preview" className="h-full w-full object-cover" />
              </div>
            )}
          </div>
        );

      case 'boolean':
        return (
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={() => updateValue(cfg.key, !val)}
              className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
                val ? 'bg-emerald-600' : 'bg-slate-200 dark:bg-slate-800'
              }`}
            >
              <span
                className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                  val ? 'translate-x-5' : 'translate-x-0'
                }`}
              />
            </button>
            <span className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {val ? 'Enabled (ON)' : 'Disabled (OFF)'}
            </span>
          </div>
        );

      case 'richtext':
        return (
          <Textarea
            rows={4}
            placeholder="<p>Enter formatted HTML or descriptive rules...</p>"
            value={val || ''}
            onChange={(e) => updateValue(cfg.key, e.target.value)}
          />
        );

      case 'integer':
      case 'number':
        return (
          <div className="flex max-w-xs items-center gap-2">
            <Hash className="h-4 w-4 text-slate-400 dark:text-slate-500" />
            <Input
              type="number"
              value={val !== undefined ? val : ''}
              onChange={(e) => updateValue(cfg.key, Number(e.target.value))}
            />
          </div>
        );

      case 'datetime':
        return (
          <div className="flex max-w-sm items-center gap-2">
            <Calendar className="h-4 w-4 text-slate-400 dark:text-slate-500" />
            <Input
              type="datetime-local"
              value={val || ''}
              onChange={(e) => updateValue(cfg.key, e.target.value)}
            />
          </div>
        );

      case 'array': {
        const arr = Array.isArray(val) ? val : [];
        return (
          <div className="space-y-2">
            {arr.map((item: string, idx: number) => (
              <div key={idx} className="flex items-center gap-2">
                <Input
                  value={item}
                  onChange={(e) => {
                    const copy = [...arr];
                    copy[idx] = e.target.value;
                    updateValue(cfg.key, copy);
                  }}
                />
                <button
                  type="button"
                  onClick={() => {
                    const copy = arr.filter((_, i) => i !== idx);
                    updateValue(cfg.key, copy);
                  }}
                  className="rounded-lg bg-slate-100 px-2.5 py-2 text-xs text-rose-600 transition hover:bg-slate-200 dark:bg-slate-800 dark:text-rose-400 dark:hover:bg-slate-700"
                >
                  ✕
                </button>
              </div>
            ))}
            <Button
              type="button"
              size="sm"
              variant="secondary"
              onClick={() => updateValue(cfg.key, [...arr, ''])}
            >
              + Add Item
            </Button>
          </div>
        );
      }

      default:
        return <Input value={val || ''} onChange={(e) => updateValue(cfg.key, e.target.value)} />;
    }
  };

  // If no subsystem is selected or available
  if (!currentSystem) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('configs.title')}
            </h1>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('configs.desc')}</p>
          </div>
          <div className="flex items-center gap-3">
            <Button variant="secondary" disabled title={t('configs.no_system_desc')}>
              <Plus className="h-4 w-4" />
              <span>{t('configs.add_field')}</span>
            </Button>
            <Button disabled title={t('configs.no_system_desc')}>
              <Save className="h-4 w-4" />
              <span>{t('configs.save_all')}</span>
            </Button>
          </div>
        </div>

        <Card className="mx-auto max-w-2xl px-6 py-16 text-center">
          <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl border border-amber-200 bg-amber-50 text-amber-600 dark:border-amber-800/60 dark:bg-amber-950/60 dark:text-amber-400">
            <AlertTriangle className="h-7 w-7" />
          </div>
          <h2 className="mb-2 text-lg font-bold text-slate-900 dark:text-slate-100">
            {t('configs.no_system_title')}
          </h2>
          <p className="mx-auto mb-6 max-w-md text-sm text-slate-500 dark:text-slate-400">
            {t('configs.no_system_desc')}
          </p>
          {onNavigate && (
            <Button onClick={() => onNavigate('systems')} className="gap-2">
              <Layers className="h-4 w-4" />
              <span>{t('configs.go_to_systems')}</span>
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
              {t('configs.title')}
            </h1>
            <Badge variant="success">/{currentSystem.slug}</Badge>
          </div>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('configs.desc')}</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="secondary" onClick={() => setIsAddOpen(true)}>
            <Plus className="h-4 w-4" />
            <span>{t('configs.add_field')}</span>
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            <Save className="h-4 w-4" />
            <span>{saving ? t('common.loading') : t('configs.save_all')}</span>
          </Button>
        </div>
      </div>

      {successMsg && (
        <div className="flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-xs text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
          <CheckCircle2 className="h-4 w-4" />
          <span>{t('configs.saved_success')}</span>
        </div>
      )}

      {configs.length === 0 && !loading && (
        <Card className="py-12 text-center text-slate-500 dark:text-slate-400">
          <Sliders className="mx-auto mb-3 h-10 w-10 text-slate-400 dark:text-slate-600" />
          <p className="text-sm">{t('configs.empty')}</p>
          <Button size="sm" variant="secondary" className="mt-4" onClick={() => setIsAddOpen(true)}>
            + Define First Setting
          </Button>
        </Card>
      )}

      {/* Visual Form Cards (No raw JSON) */}
      <div className="space-y-4">
        {configs.map((cfg) => (
          <Card key={cfg.id} className="space-y-2.5">
            <div className="flex items-center justify-between">
              <div>
                <span className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                  {cfg.label}
                </span>
                <span className="ml-2 font-mono text-xs text-slate-400 dark:text-slate-500">
                  ({cfg.key})
                </span>
              </div>
              <Badge variant="purple">{cfg.value_type}</Badge>
            </div>
            <div>{renderWidget(cfg)}</div>
          </Card>
        ))}
      </div>

      {/* Modal to add config property definition */}
      <Modal isOpen={isAddOpen} onClose={() => setIsAddOpen(false)} title={t('configs.add_field')}>
        <form onSubmit={handleAddField} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Configuration Key (e.g. banner_url, start_time, rules)
            </label>
            <Input
              required
              placeholder="banner_url"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Display Label
            </label>
            <Input
              required
              placeholder="Campaign Banner Image"
              value={newLabel}
              onChange={(e) => setNewLabel(e.target.value)}
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              Visual Widget Type
            </label>
            <select
              className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
              value={newType}
              onChange={(e) => setNewType(e.target.value)}
            >
              <option value="string">String (Single line input)</option>
              <option value="richtext">Richtext (HTML / Rules editor)</option>
              <option value="image">Image (URL & Preview uploader)</option>
              <option value="boolean">Boolean (Toggle switch)</option>
              <option value="integer">Integer (Counter)</option>
              <option value="number">Number / Decimal</option>
              <option value="datetime">DateTime Picker</option>
              <option value="array">Array (Dynamic Repeater list)</option>
            </select>
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsAddOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
