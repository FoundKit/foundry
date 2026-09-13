import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Save, CheckCircle2, AlertCircle } from 'lucide-react';
import { Card, Button, Input, Textarea, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import { SystemItem } from '../types';

interface SubsystemSettingsPageProps {
  currentSystem: SystemItem;
  onRefresh: () => void;
}

export function SubsystemSettingsPage({ currentSystem, onRefresh }: SubsystemSettingsPageProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(currentSystem.name);
  const [description, setDescription] = useState(currentSystem.description || '');
  const [status, setStatus] = useState(currentSystem.status);
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await api.updateSystem(currentSystem.id, {
        name,
        description: description || undefined,
        status,
      });
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
      onRefresh();
    } catch (err: any) {
      setError(err.message || 'Failed to update settings');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-3xl space-y-6">
      <div>
        <div className="flex items-center gap-2">
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
            {t('subsystem.settings_title')}
          </h1>
          <Badge variant="success">/{currentSystem.slug}</Badge>
        </div>
        <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
          {t('subsystem.settings_desc')}
        </p>
      </div>

      {success && (
        <div className="flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-xs text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
          <CheckCircle2 className="h-4 w-4" />
          <span>{t('subsystem.settings_saved')}</span>
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      <Card>
        <form onSubmit={handleSave} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              System ID (UUID)
            </label>
            <Input value={currentSystem.id} disabled className="font-mono text-xs opacity-70" />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              System Slug (Immutable Identifier)
            </label>
            <Input value={currentSystem.slug} disabled className="font-mono text-xs opacity-70" />
            <span className="mt-1 block text-[10px] text-slate-400">
              Unique slug cannot be changed once created to avoid breaking API routes.
            </span>
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.name')}
            </label>
            <Input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Display Name"
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.description')}
            </label>
            <Textarea
              rows={3}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Short description of this sub-system..."
            />
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('systems.status')}
            </label>
            <select
              value={status}
              onChange={(e) => setStatus(Number(e.target.value))}
              className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
            >
              <option value={1}>{t('systems.active')}</option>
              <option value={0}>{t('systems.disabled')}</option>
            </select>
          </div>

          <div className="flex justify-end pt-2">
            <Button type="submit" disabled={loading} className="gap-1.5">
              <Save className="h-4 w-4" />
              <span>{loading ? t('common.loading') : t('subsystem.save_settings')}</span>
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}
