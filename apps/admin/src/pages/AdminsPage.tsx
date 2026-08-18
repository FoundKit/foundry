import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { ShieldCheck, UserPlus, AlertCircle } from 'lucide-react';
import { Card, Button, Input, Modal, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import { AdminProfile, SystemItem } from '../types';

interface AdminsPageProps {
  systems: SystemItem[];
}

export function AdminsPage({ systems }: AdminsPageProps) {
  const { t } = useTranslation();
  const [admins, setAdmins] = useState<AdminProfile[]>([]);
  const [loading, setLoading] = useState(false);

  // Create Admin Modal
  const [isOpen, setIsOpen] = useState(false);
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [role, setRole] = useState<'super_admin' | 'admin'>('admin');
  const [selectedSystems, setSelectedSystems] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const loadAdmins = async () => {
    setLoading(true);
    try {
      const list = await api.listAdmins();
      setAdmins(list);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAdmins();
  }, []);

  const handleCreateAdmin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (role === 'admin' && systems.length === 0) {
      setError(t('admins.no_systems_warning'));
      return;
    }
    try {
      const allowed = role === 'super_admin' ? ['*'] : selectedSystems;
      await api.createAdmin({
        username,
        email: email || undefined,
        password,
        role,
        allowed_systems: allowed,
      });
      setIsOpen(false);
      setUsername('');
      setEmail('');
      setPassword('');
      setSelectedSystems([]);
      loadAdmins();
    } catch (err: any) {
      setError(err.message || 'Failed to create admin');
    }
  };

  const toggleSystemSelection = (slug: string) => {
    setSelectedSystems((prev) =>
      prev.includes(slug) ? prev.filter((s) => s !== slug) : [...prev, slug]
    );
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">{t('admins.title')}</h1>
          <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">{t('admins.desc')}</p>
        </div>
        <Button onClick={() => {
          setError(null);
          setIsOpen(true);
        }}>
          <UserPlus className="w-4 h-4" />
          <span>{t('admins.create_admin')}</span>
        </Button>
      </div>

      <Card className="p-0 overflow-hidden">
        <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
          <thead className="bg-slate-50 dark:bg-slate-950/60 text-xs uppercase font-medium text-slate-500 dark:text-slate-400 border-b border-slate-200 dark:border-slate-800">
            <tr>
              <th className="px-6 py-4">{t('auth.username')}</th>
              <th className="px-6 py-4">Email</th>
              <th className="px-6 py-4">{t('admins.role')}</th>
              <th className="px-6 py-4">{t('admins.allowed_systems')}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-200 dark:divide-slate-800/60 text-xs">
            {admins.map((adm) => (
              <tr key={adm.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/30 transition">
                <td className="px-6 py-4 font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                  <ShieldCheck className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                  <span>{adm.username}</span>
                </td>
                <td className="px-6 py-4 text-slate-500 dark:text-slate-400">{adm.email || '-'}</td>
                <td className="px-6 py-4">
                  <Badge variant={adm.role === 'super_admin' ? 'purple' : 'info'}>
                    {adm.role === 'super_admin' ? t('admins.super_admin') : t('admins.normal_admin')}
                  </Badge>
                </td>
                <td className="px-6 py-4">
                  {adm.role === 'super_admin' ? (
                    <Badge variant="success">{t('admins.all_topics_wildcard')}</Badge>
                  ) : (
                    <div className="flex flex-wrap gap-1.5">
                      {adm.allowed_systems.map((s, idx) => (
                        <span
                          key={idx}
                          className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 text-[10px] font-mono border border-slate-200 dark:border-slate-700"
                        >
                          /{s}
                        </span>
                      ))}
                    </div>
                  )}
                </td>
              </tr>
            ))}
            {admins.length === 0 && (
              <tr>
                <td colSpan={4} className="text-center py-8 text-slate-500 dark:text-slate-400">
                  {loading ? t('common.loading') : 'No administrators registered.'}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </Card>

      {/* Modal: Create Admin */}
      <Modal
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        title={t('admins.create_admin')}
      >
        {error && (
          <div className="mb-4 p-3 rounded-lg bg-rose-50 dark:bg-rose-950/60 border border-rose-200 dark:border-rose-800/60 text-xs text-rose-700 dark:text-rose-300 flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}
        <form onSubmit={handleCreateAdmin} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('auth.username')}
            </label>
            <Input
              required
              placeholder="operator_zhang"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              Email (Optional)
            </label>
            <Input
              type="email"
              placeholder="operator@company.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('auth.password')}
            </label>
            <Input
              type="password"
              required
              placeholder="••••••••"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
              {t('admins.role')}
            </label>
            <select
              className="w-full bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-800 rounded-lg px-3.5 py-2 text-sm text-slate-900 dark:text-slate-100 focus:outline-none focus:border-emerald-500"
              value={role}
              onChange={(e) => setRole(e.target.value as any)}
            >
              <option value="admin">{t('admins.normal_admin')} (Topic Scoped)</option>
              <option value="super_admin">{t('admins.super_admin')} (Platform Wildcard)</option>
            </select>
          </div>

          {role === 'admin' && (
            <div>
              <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1.5">
                Assign Allowed Sub-Systems
              </label>
              {systems.length > 0 ? (
                <div className="space-y-1.5 max-h-40 overflow-y-auto p-2 bg-slate-50 dark:bg-slate-950 rounded-lg border border-slate-200 dark:border-slate-800">
                  {systems.map((s) => {
                    const isChecked = selectedSystems.includes(s.slug);
                    return (
                      <label
                        key={s.id}
                        className="flex items-center gap-2 text-xs text-slate-700 dark:text-slate-300 cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-900 p-1.5 rounded transition"
                      >
                        <input
                          type="checkbox"
                          checked={isChecked}
                          onChange={() => toggleSystemSelection(s.slug)}
                          className="rounded bg-white dark:bg-slate-950 border-slate-300 dark:border-slate-800 text-emerald-600 focus:ring-emerald-500"
                        />
                        <span>{s.name}</span>
                        <span className="font-mono text-slate-400 dark:text-slate-500 text-[10px]">/{s.slug}</span>
                      </label>
                    );
                  })}
                </div>
              ) : (
                <div className="p-3 bg-amber-50 dark:bg-amber-950/40 border border-amber-200 dark:border-amber-800/40 rounded-lg text-xs text-amber-700 dark:text-amber-300">
                  {t('admins.no_systems_warning')}
                </div>
              )}
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={() => setIsOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.confirm')}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
