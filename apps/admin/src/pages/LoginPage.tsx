import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  ShieldCheck,
  ArrowRight,
  Lock,
  User,
  Globe,
  Mail,
  KeyRound,
  CheckCircle2,
} from 'lucide-react';
import { Button, Input } from '../components/UiWidgets';
import { ThemeToggle } from '../components/ThemeToggle';
import { api } from '../services/api';
import { AdminProfile, PublicAuthSettings } from '../types';

interface LoginPageProps {
  onLoginSuccess: (token: string, admin: AdminProfile) => void;
}

export function LoginPage({ onLoginSuccess }: LoginPageProps) {
  const { t, i18n } = useTranslation();
  const [authSettings, setAuthSettings] = useState<PublicAuthSettings>({
    enable_password_login: true,
    enable_email_login: false,
  });
  const [activeTab, setActiveTab] = useState<'password' | 'email'>('password');

  // Password login form state
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  // Email login form state
  const [email, setEmail] = useState('');
  const [code, setCode] = useState('');
  const [sendingCode, setSendingCode] = useState(false);
  const [cooldown, setCooldown] = useState(0);
  const [codeSuccessMsg, setCodeSuccessMsg] = useState<string | null>(null);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Fetch public auth configurations
    api
      .getPublicAuthSettings()
      .then((settings) => {
        setAuthSettings(settings);
        if (!settings.enable_password_login && settings.enable_email_login) {
          setActiveTab('email');
        } else {
          setActiveTab('password');
        }
        if (settings.show_default_admin_hint) {
          setUsername((prev) => prev || 'admin');
          setPassword((prev) => prev || 'admin123456');
        }
      })
      .catch((err) => {
        console.warn('Failed to load public auth settings, fallback to defaults:', err);
      });
  }, []);

  // Cooldown countdown timer
  useEffect(() => {
    if (cooldown <= 0) return;
    const timer = setInterval(() => {
      setCooldown((prev) => prev - 1);
    }, 1000);
    return () => clearInterval(timer);
  }, [cooldown]);

  const toggleLanguage = () => {
    const nextLang = i18n.language.startsWith('zh') ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(nextLang);
  };

  const handlePasswordSubmit = async (e: React.FormEvent) => {
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

  const handleSendCode = async () => {
    if (!email || !email.includes('@')) {
      setError('请输入有效的邮箱地址 (Please enter a valid email address)');
      return;
    }
    setError(null);
    setCodeSuccessMsg(null);
    setSendingCode(true);
    try {
      const res = await api.sendLoginEmailCode(email);
      setCodeSuccessMsg(res.message || '验证码已发送至您的邮箱，5分钟内有效');
      setCooldown(60);
    } catch (err: any) {
      setError(err.message || '发送验证码失败');
    } finally {
      setSendingCode(false);
    }
  };

  const handleEmailSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!code || code.trim().length === 0) {
      setError('请输入邮箱验证码 (Please enter the verification code)');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await api.loginWithEmail({ email, code: code.trim() });
      onLoginSuccess(res.token, res.admin);
    } catch (err: any) {
      setError(err.message || '邮箱验证码登录失败');
    } finally {
      setLoading(false);
    }
  };

  const showBothTabs = authSettings.enable_password_login && authSettings.enable_email_login;

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
          {/* Method Tabs if both are enabled */}
          {showBothTabs && (
            <div className="mb-5 flex rounded-xl bg-slate-100 p-1 dark:bg-slate-800/80">
              <button
                type="button"
                onClick={() => {
                  setActiveTab('password');
                  setError(null);
                  setCodeSuccessMsg(null);
                }}
                className={`flex flex-1 items-center justify-center gap-1.5 rounded-lg py-2 text-xs font-semibold transition ${
                  activeTab === 'password'
                    ? 'bg-white text-emerald-700 shadow-sm dark:bg-slate-700 dark:text-emerald-400'
                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200'
                }`}
              >
                <User className="h-3.5 w-3.5" />
                <span>账号密码登录</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setActiveTab('email');
                  setError(null);
                  setCodeSuccessMsg(null);
                }}
                className={`flex flex-1 items-center justify-center gap-1.5 rounded-lg py-2 text-xs font-semibold transition ${
                  activeTab === 'email'
                    ? 'bg-white text-emerald-700 shadow-sm dark:bg-slate-700 dark:text-emerald-400'
                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200'
                }`}
              >
                <Mail className="h-3.5 w-3.5" />
                <span>邮箱验证码登录</span>
              </button>
            </div>
          )}

          {error && (
            <div className="mb-5 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
              {error}
            </div>
          )}

          {codeSuccessMsg && (
            <div className="mb-5 flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-xs text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
              <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400" />
              <span>{codeSuccessMsg}</span>
            </div>
          )}

          {/* Form 1: Password Login */}
          {activeTab === 'password' && authSettings.enable_password_login && (
            <form onSubmit={handlePasswordSubmit} className="space-y-4">
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
          )}

          {/* Form 2: Email Verification Code Login */}
          {activeTab === 'email' && authSettings.enable_email_login && (
            <form onSubmit={handleEmailSubmit} className="space-y-4">
              <div>
                <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  管理员绑定邮箱 (Admin Email)
                </label>
                <div className="relative">
                  <Mail className="absolute left-3.5 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
                  <Input
                    type="email"
                    required
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="pl-10"
                    placeholder="admin@foundry.local"
                  />
                </div>
              </div>

              <div>
                <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                  邮箱验证码 (Verification Code)
                </label>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <KeyRound className="absolute left-3.5 top-3 h-4 w-4 text-slate-400 dark:text-slate-500" />
                    <Input
                      type="text"
                      maxLength={6}
                      required
                      value={code}
                      onChange={(e) => setCode(e.target.value)}
                      className="pl-10 font-mono font-semibold tracking-widest"
                      placeholder="6 位数字"
                    />
                  </div>
                  <button
                    type="button"
                    disabled={sendingCode || cooldown > 0 || !email}
                    onClick={handleSendCode}
                    className="shrink-0 rounded-lg border border-slate-200 bg-slate-50 px-3.5 py-2 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-50 dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
                  >
                    {sendingCode ? '发送中...' : cooldown > 0 ? `${cooldown}s` : '获取验证码'}
                  </button>
                </div>
              </div>

              <Button type="submit" disabled={loading} className="mt-2 w-full" size="lg">
                <span>{loading ? t('common.loading') : '验证码登录 (Sign In)'}</span>
                <ArrowRight className="ml-1 h-4 w-4" />
              </Button>
            </form>
          )}

          {/* Footer note */}
          <div className="mt-6 border-t border-slate-100 pt-4 text-center dark:border-slate-800/80">
            {!authSettings.enable_password_login ? (
              <p className="text-xs text-amber-600 dark:text-amber-400">
                ⚠️ 当前系统已关闭账号密码登录，请使用已绑定的有效管理员邮箱接收验证码登录。
              </p>
            ) : authSettings.show_default_admin_hint ? (
              <p className="text-xs text-slate-500 dark:text-slate-400">
                Default Super Admin:{' '}
                <button
                  type="button"
                  onClick={() => {
                    setUsername('admin');
                    setPassword('admin123456');
                  }}
                  className="font-mono font-medium text-emerald-600 underline-offset-2 hover:underline dark:text-emerald-400"
                  title="点击填入"
                >
                  admin
                </button>{' '}
                /{' '}
                <button
                  type="button"
                  onClick={() => {
                    setUsername('admin');
                    setPassword('admin123456');
                  }}
                  className="font-mono font-medium text-emerald-600 underline-offset-2 hover:underline dark:text-emerald-400"
                  title="点击填入"
                >
                  admin123456
                </button>
              </p>
            ) : null}
          </div>
        </div>
      </div>

      <div className="text-center text-xs text-slate-400 dark:text-slate-600">
        Foundry Control Plane &copy; 2026
      </div>
    </div>
  );
}
