import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Layers, AlertCircle, PlusCircle } from 'lucide-react';
import { Card, Button, Input, Textarea, Badge, Modal } from '../components/UiWidgets';
import { api } from '../services/api';
import { AdminProfile, SystemItem } from '../types';

interface SystemsPageProps {
  admin: AdminProfile;
  systems: SystemItem[];
  onRefresh: () => void;
  onSelectSystem: (system: SystemItem) => void;
}

export function SystemsPage({
  admin,
  systems,
  onRefresh,
  onSelectSystem,
}: SystemsPageProps) {
  const { t } = useTranslation();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [slug, setSlug] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await api.createSystem({ slug, name, description });
      setIsCreateOpen(false);
      setSlug('');
      setName('');
      setDescription('');
      onRefresh();
    } catch (err: any) {
      setError(err.message || 'Failed to create sub-system');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">{t('systems.title')}</h1>
          <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">{t('systems.desc')}</p>
        </div>
        {admin.role === 'super_admin' && (
          <Button onClick={() => setIsCreateOpen(true)} className="gap-2">
            <Plus className="w-4 h-4" />
            <span>{t('systems.create')}</span>
          </Button>
        )}
      </div>

      {/* Systems Table */}
      <Card className="p-0 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
            <thead className="bg-slate-50 dark:bg-slate-950/60 text-xs uppercase font-medium text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-800">
              <tr>
                <th className="px-6 py-4">{t('systems.name')}</th>
                <th className="px-6 py-4">{t('systems.slug')}</th>
                <th className="px-6 py-4">{t('systems.description')}</th>
                <th className="px-6 py-4">{t('systems.status')}</th>
                <th className="px-6 py-4 text-right">{t('systems.actions')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 dark:divide-slate-800/60">
              {systems.map((sys) => (
                <tr key={sys.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/30 transition">
                  <td className="px-6 py-4 font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2.5">
                    <Layers className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                    <span>{sys.name}</span>
                  </td>
                  <td className="px-6 py-4 font-mono text-xs text-emerald-600 dark:text-emerald-400">
                    /{sys.slug}
                  </td>
                  <td className="px-6 py-4 text-xs text-slate-500 dark:text-slate-400 max-w-md truncate">
                    {sys.description || '-'}
                  </td>
                  <td className="px-6 py-4">
                    <Badge variant={sys.status === 1 ? 'success' : 'default'}>
                      {sys.status === 1 ? t('systems.active') : t('systems.disabled')}
                    </Badge>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => onSelectSystem(sys)}
                    >
                      Manage
                    </Button>
                  </td>
                </tr>
              ))}
              {systems.length === 0 && (
                <tr>
                  <td colSpan={5} className="text-center py-16 px-4">
                    <Layers className="w-10 h-10 mx-auto text-slate-400 dark:text-slate-600 mb-3" />
                    <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100 mb-1">
                      {t('systems.empty_title')}
                    </h3>
                    <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mx-auto mb-4">
                      {t('systems.empty_desc')}
                    </p>
                    {admin.role === 'super_admin' && (
                      <Button onClick={() => setIsCreateOpen(true)} className="gap-2 mx-auto">
                        <PlusCircle className="w-4 h-4" />
                        <span>{t('systems.create')}</span>
                      </Button>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Create System Modal */}
      <Modal
        isOpen={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
        title={t('systems.create')}
      >
        {error && (
          <div className="mb-4 p-3 rounded-lg bg-rose-50 dark:bg-rose-950/60 border border-rose-200 dark:border-rose-800/60 text-xs text-rose-700 dark:text-rose-300 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}
        <form onSubmit={handleCreate} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('systems.slug')} (Immutable identifier)
            </label>
            <Input
              required
              placeholder="carnival_2026"
              value={slug}
              onChange={(e) => setSlug(e.target.value.toLowerCase())}
            />
            <span className="text-[10px] text-slate-500 dark:text-slate-400 mt-1 block">
              Alphanumeric with underscores/hyphens (e.g. carnival_2026, vip_mall)
            </span>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('systems.name')}
            </label>
            <Input
              required
              placeholder="Marketing Carnival 2026"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('systems.description')}
            </label>
            <Textarea
              rows={3}
              placeholder="Short description of this sub-system..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => setIsCreateOpen(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? t('common.loading') : t('common.confirm')}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
