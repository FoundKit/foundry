import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ShieldCheck, ArrowRight, Lock, User, Globe } from 'lucide-react';
import { Button, Input } from '../components/UiWidgets';
import { ThemeToggle } from '../components/ThemeToggle';
import { api } from '../services/api';
import { AdminProfile } from '../types';

interface LoginPageProps {
  onLoginSuccess: (token: string, admin: AdminProfile) => void;
}

export function LoginPage({ onLoginSuccess }: LoginPageProps) {
  const { t, i18n } = useTranslation();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('admin123456');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleLanguage = () => {
    const nextLang = i18n.language.startsWith('zh') ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(nextLang);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const res = await api.login({ username, password });
      onLoginSuccess(res.token, res.admin);
    } catch (err: any) {
      setError(err.message || t('auth.login_failed'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col justify-between bg-slate-50 p-6 transition-colors duration-150 dark:bg-slate-950">
      {/* Top right language & theme toggle */}
      <div className="flex justify-end gap-2">
        <ThemeToggle />
        <button
          onClick={toggleLanguage}
          className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-300 dark:shadow-none dark:hover:bg-slate-800 dark:hover:text-white"
        >
          <Globe className="h-3.5 w-3.5 text-slate-500 dark:text-slate-400" />
          <span>{i18n.language.startsWith('zh') ? '中文' : 'EN'}</span>
        </button>
      </div>

      <div className="mx-auto my-auto w-full max-w-md">
        <div className="mb-8 text-center">
          <div className="mb-4 inline-flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-tr from-emerald-600 to-teal-500 shadow-xl shadow-emerald-500/20">
            <ShieldCheck className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-slate-900 dark:text-slate-100">
            {t('auth.login_title')}
          </h1>
          <p className="mt-2 text-sm text-slate-500 dark:text-slate-400">
            {t('auth.login_subtitle')}
          </p>
        </div>

        <div className="rounded-2xl border border-slate-200 bg-white p-6 shadow-xl dark:border-slate-800 dark:bg-slate-900 dark:shadow-2xl">
          {error && (
            <div className="mb-5 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
              {error}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('auth.username')}
              </label>
              <div className="relative">
                <User className="absolute left-3.5 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
                <Input
                  type="text"
                  required
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="pl-10"
                  placeholder="admin"
                />
              </div>
            </div>

            <div>
              <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                {t('auth.password')}
              </label>
              <div className="relative">
                <Lock className="absolute left-3.5 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
                <Input
                  type="password"
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="pl-10"
                  placeholder="••••••••"
                />
              </div>
            </div>

            <Button type="submit" disabled={loading} className="mt-2 w-full" size="lg">
              <span>{loading ? t('common.loading') : t('auth.sign_in')}</span>
              <ArrowRight className="ml-1 h-4 w-4" />
            </Button>
          </form>

          <div className="mt-6 border-t border-slate-100 pt-4 text-center dark:border-slate-800/80">
            <p className="text-xs text-slate-500 dark:text-slate-400">
              Default Super Admin:{' '}
              <code className="font-mono font-medium text-emerald-600 dark:text-emerald-400">
                admin
              </code>{' '}
              /{' '}
              <code className="font-mono font-medium text-emerald-600 dark:text-emerald-400">
                admin123456
              </code>
            </p>
          </div>
        </div>
      </div>

      <div className="text-center text-xs text-slate-400 dark:text-slate-600">
        Foundry Control Plane &copy; 2026
      </div>
    </div>
  );
}
