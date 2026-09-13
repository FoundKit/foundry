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
  const [role, setRole] = useState<'super_admin' | 'admin' | 'topic_admin'>('topic_admin');
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

    if (role === 'topic_admin') {
      if (systems.length === 0) {
        setError(t('admins.no_systems_warning'));
        return;
      }
      if (selectedSystems.length === 0) {
        setError('请至少为专题管理员选择一个授权专题 (Please select at least one sub-system)');
        return;
      }
    }

    try {
      const allowed = role === 'topic_admin' ? selectedSystems : ['*'];
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
      setRole('topic_admin');
      setSelectedSystems([]);
      loadAdmins();
    } catch (err: any) {
      setError(err.message || 'Failed to create admin');
    }
  };

  const toggleSystemSelection = (slug: string) => {
    setSelectedSystems((prev) =>
      prev.includes(slug) ? prev.filter((s) => s !== slug) : [...prev, slug],
    );
  };

  const getRoleBadgeVariant = (r: AdminProfile['role']) => {
    switch (r) {
      case 'super_admin':
        return 'purple';
      case 'admin':
        return 'info';
      case 'topic_admin':
        return 'success';
      default:
        return 'default';
    }
  };

  const getRoleLabel = (r: AdminProfile['role']) => {
    switch (r) {
      case 'super_admin':
        return t('admins.super_admin');
      case 'admin':
        return t('admins.normal_admin');
      case 'topic_admin':
        return t('admins.topic_admin');
      default:
        return r;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
            {t('admins.title')}
          </h1>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t('admins.desc')}</p>
        </div>
        <Button
          onClick={() => {
            setError(null);
            setIsOpen(true);
          }}
        >
          <UserPlus className="h-4 w-4" />
          <span>{t('admins.create_admin')}</span>
        </Button>
      </div>

      <Card className="overflow-hidden p-0">
        <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
          <thead className="border-b border-slate-200 bg-slate-50 text-xs font-medium uppercase text-slate-500 dark:border-slate-800 dark:bg-slate-950/60 dark:text-slate-400">
            <tr>
              <th className="px-6 py-4">{t('auth.username')}</th>
              <th className="px-6 py-4">Email</th>
              <th className="px-6 py-4">{t('admins.role')}</th>
              <th className="px-6 py-4">{t('admins.allowed_systems')}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-200 text-xs dark:divide-slate-800/60">
            {admins.map((adm) => (
              <tr key={adm.id} className="transition hover:bg-slate-50 dark:hover:bg-slate-800/30">
                <td className="flex items-center gap-2 px-6 py-4 font-semibold text-slate-900 dark:text-slate-100">
                  <ShieldCheck className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
                  <span>{adm.username}</span>
                </td>
                <td className="px-6 py-4 text-slate-500 dark:text-slate-400">{adm.email || '-'}</td>
                <td className="px-6 py-4">
                  <Badge variant={getRoleBadgeVariant(adm.role) as any}>
                    {getRoleLabel(adm.role)}
                  </Badge>
                </td>
                <td className="px-6 py-4">
                  {adm.role === 'super_admin' || adm.role === 'admin' ? (
                    <Badge variant="success">{t('admins.all_topics_wildcard')}</Badge>
                  ) : (
                    <div className="flex flex-wrap gap-1.5">
                      {adm.allowed_systems && adm.allowed_systems.length > 0 ? (
                        adm.allowed_systems.map((s, idx) => (
                          <span
                            key={idx}
                            className="rounded border border-emerald-200 bg-emerald-50 px-2 py-0.5 font-mono text-[10px] text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950/60 dark:text-emerald-300"
                          >
                            /{s}
                          </span>
                        ))
                      ) : (
                        <span className="text-slate-400">无授权专题 (None)</span>
                      )}
                    </div>
                  )}
                </td>
              </tr>
            ))}
            {admins.length === 0 && (
              <tr>
                <td colSpan={4} className="py-8 text-center text-slate-500 dark:text-slate-400">
                  {loading ? t('common.loading') : 'No administrators registered.'}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </Card>

      {/* Modal: Create Admin */}
      <Modal isOpen={isOpen} onClose={() => setIsOpen(false)} title={t('admins.create_admin')}>
        {error && (
          <div className="mb-4 flex items-center gap-2 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}
        <form onSubmit={handleCreateAdmin} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
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
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
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
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
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
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('admins.role')}
            </label>
            <select
              className="w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 focus:border-emerald-500 focus:outline-none dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100"
              value={role}
              onChange={(e) => setRole(e.target.value as any)}
            >
              <option value="topic_admin">{t('admins.topic_admin')}</option>
              <option value="admin">{t('admins.normal_admin')}</option>
              <option value="super_admin">{t('admins.super_admin')}</option>
            </select>
          </div>

          {role === 'topic_admin' && (
            <div>
              <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                指派授权子系统/专题 (Assign Allowed Sub-Systems)
              </label>
              {systems.length > 0 ? (
                <div className="max-h-40 space-y-1.5 overflow-y-auto rounded-lg border border-slate-200 bg-slate-50 p-2 dark:border-slate-800 dark:bg-slate-950">
                  {systems.map((s) => {
                    const isChecked = selectedSystems.includes(s.slug);
                    return (
                      <label
                        key={s.id}
                        className="flex cursor-pointer items-center gap-2 rounded p-1.5 text-xs text-slate-700 transition hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-900"
                      >
                        <input
                          type="checkbox"
                          checked={isChecked}
                          onChange={() => toggleSystemSelection(s.slug)}
                          className="rounded border-slate-300 bg-white text-emerald-600 focus:ring-emerald-500 dark:border-slate-800 dark:bg-slate-950"
                        />
                        <span>{s.name}</span>
                        <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500">
                          /{s.slug}
                        </span>
                      </label>
                    );
                  })}
                </div>
              ) : (
                <div className="rounded-lg border border-amber-200 bg-amber-50 p-3 text-xs text-amber-700 dark:border-amber-800/40 dark:bg-amber-950/40 dark:text-amber-300">
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
