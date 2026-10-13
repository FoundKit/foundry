import { useState, useEffect } from 'react';
import {
  Mail,
  Lock,
  Save,
  AlertTriangle,
  CheckCircle2,
  AlertCircle,
  Eye,
  Server,
  RefreshCw,
} from 'lucide-react';
import { Card, Button, Input } from '../components/UiWidgets';
import { api } from '../services/api';
import { PlatformLoginSettings } from '../types';

export function PlatformSettingsPage() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const [settings, setSettings] = useState<PlatformLoginSettings>({
    enable_password_login: true,
    enable_email_login: false,
    login_email_subject: '【Foundry】登录验证码',
    login_email_template:
      '您的登录验证码是：{code}，5分钟内有效。请勿将验证码泄露给他人。如非本人操作请忽略。',
  });

  const loadSettings = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.getPlatformLoginSettings();
      setSettings(data);
    } catch (err: any) {
      setError(err.message || '加载平台配置失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSettings();
  }, []);

  // Validation rules
  const isBothDisabled = !settings.enable_password_login && !settings.enable_email_login;
  const hasCodePlaceholder = settings.login_email_template.includes('{code}');
  const isSubjectEmpty = settings.login_email_subject.trim().length === 0;
  const canSave = !isBothDisabled && hasCodePlaceholder && !isSubjectEmpty;

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isBothDisabled) {
      setError('必须至少保留一种登录方式（账号密码登录或邮箱验证码登录）！');
      return;
    }
    if (!hasCodePlaceholder) {
      setError('登录邮件内容必须包含验证码占位符 {code}！');
      return;
    }
    if (isSubjectEmpty) {
      setError('登录邮件主题不能为空！');
      return;
    }

    setSaving(true);
    setError(null);
    setSuccessMsg(null);
    try {
      const updated = await api.updatePlatformLoginSettings(settings);
      setSettings(updated);
      setSuccessMsg('平台登录设置保存成功！所有变更已即时生效。');
    } catch (err: any) {
      setError(err.message || '保存设置失败');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center text-sm text-slate-500">
        <RefreshCw className="mr-2 h-4 w-4 animate-spin text-emerald-600" />
        <span>加载平台设置中...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Page Title */}
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              平台与登录安全设置
            </h1>
            <span className="rounded-md border border-purple-200 bg-purple-50 px-2 py-0.5 text-xs font-semibold text-purple-700 dark:border-purple-800 dark:bg-purple-950/80 dark:text-purple-300">
              超级管理员专属
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
            自定义平台登录方式、配置邮箱验证码登录及邮件内容模板，保障系统访问控制安全。
          </p>
        </div>
        <Button onClick={handleSave} disabled={saving || !canSave}>
          <Save className="h-4 w-4" />
          <span>{saving ? '保存中...' : '保存全局设置'}</span>
        </Button>
      </div>

      {/* Feedback Messages */}
      {error && (
        <div className="flex items-center gap-2 rounded-xl border border-rose-200 bg-rose-50 p-4 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {successMsg && (
        <div className="flex items-center gap-2 rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-xs text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
          <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400" />
          <span>{successMsg}</span>
        </div>
      )}

      <form onSubmit={handleSave} className="space-y-6">
        {/* Card 1: Login Methods Configuration */}
        <Card className="p-6">
          <div className="flex items-center gap-2.5 border-b border-slate-200 pb-4 dark:border-slate-800">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-400">
              <Lock className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                登录认证方式 (Authentication Methods)
              </h2>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                系统创建后默认仅开启账号密码登录。支持关闭账号密码登录或同时开启两种方式。
              </p>
            </div>
          </div>

          <div className="mt-5 space-y-4">
            {/* Switch 1: Password Login */}
            <div className="flex items-start justify-between rounded-xl border border-slate-200 bg-slate-50/60 p-4 dark:border-slate-800 dark:bg-slate-900/40">
              <div className="space-y-1 pr-4">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                    开启账号密码登录
                  </span>
                  <span className="py-0.2 rounded bg-slate-200/80 px-1.5 text-[10px] text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                    默认开启
                  </span>
                </div>
                <p className="text-xs text-slate-500 dark:text-slate-400">
                  允许管理员使用用户名和密码进行后台认证登录。
                </p>
              </div>
              <label className="relative inline-flex cursor-pointer items-center">
                <input
                  type="checkbox"
                  checked={settings.enable_password_login}
                  onChange={(e) =>
                    setSettings({ ...settings, enable_password_login: e.target.checked })
                  }
                  className="peer sr-only"
                />
                <div className="peer h-6 w-11 rounded-full bg-slate-200 after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-slate-300 after:bg-white after:transition-all after:content-[''] peer-checked:bg-emerald-600 peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none dark:bg-slate-700" />
              </label>
            </div>

            {/* Switch 2: Email Verification Code Login */}
            <div className="flex items-start justify-between rounded-xl border border-slate-200 bg-slate-50/60 p-4 dark:border-slate-800 dark:bg-slate-900/40">
              <div className="space-y-1 pr-4">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                    开启邮箱验证码登录
                  </span>
                  <span className="py-0.2 rounded bg-emerald-100 px-1.5 text-[10px] text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300">
                    高安全性
                  </span>
                </div>
                <p className="text-xs text-slate-500 dark:text-slate-400">
                  允许管理员输入绑定的有效邮箱接收 6 位验证码免密码安全登录。
                </p>
              </div>
              <label className="relative inline-flex cursor-pointer items-center">
                <input
                  type="checkbox"
                  checked={settings.enable_email_login}
                  onChange={(e) =>
                    setSettings({ ...settings, enable_email_login: e.target.checked })
                  }
                  className="peer sr-only"
                />
                <div className="peer h-6 w-11 rounded-full bg-slate-200 after:absolute after:left-[2px] after:top-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-slate-300 after:bg-white after:transition-all after:content-[''] peer-checked:bg-emerald-600 peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none dark:bg-slate-700" />
              </label>
            </div>

            {/* Rule 1 Warning: Both Disabled */}
            {isBothDisabled && (
              <div className="flex items-start gap-2.5 rounded-xl border border-rose-300 bg-rose-50 p-4 text-xs text-rose-800 dark:border-rose-800/80 dark:bg-rose-950/60 dark:text-rose-200">
                <AlertTriangle className="h-4 w-4 shrink-0 text-rose-600 dark:text-rose-400" />
                <div>
                  <div className="font-semibold">禁止同时关闭所有登录方式！</div>
                  <div className="mt-0.5">
                    系统必须至少保留一种可用登录方式（账号密码登录或邮箱验证码登录），否则管理员将无法登录后台。
                  </div>
                </div>
              </div>
            )}

            {/* Rule 2 Notice: Password Disabled Warning */}
            {!settings.enable_password_login && settings.enable_email_login && (
              <div className="flex items-start gap-2.5 rounded-xl border border-amber-300 bg-amber-50 p-4 text-xs text-amber-800 dark:border-amber-800/80 dark:bg-amber-950/60 dark:text-amber-200">
                <AlertTriangle className="h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
                <div>
                  <div className="font-semibold">⚠️ 账号密码登录已关闭提示</div>
                  <div className="mt-0.5">
                    保存后，所有管理员登录将完全依赖邮箱验证码。请务必确认相关管理员账号均已在后台完成邮箱绑定与有效性验证，避免出现无法登录的情况。
                  </div>
                </div>
              </div>
            )}

            {/* Both Enabled Tip */}
            {settings.enable_password_login && settings.enable_email_login && (
              <div className="flex items-start gap-2.5 rounded-xl border border-blue-200 bg-blue-50/70 p-3.5 text-xs text-blue-800 dark:border-blue-900/60 dark:bg-blue-950/40 dark:text-blue-300">
                <CheckCircle2 className="h-4 w-4 shrink-0 text-blue-600 dark:text-blue-400" />
                <span>
                  当前已同时开启两种登录方式。管理员可在后台登录页面自由切换“账号密码登录”或“邮箱验证码登录”。
                </span>
              </div>
            )}
          </div>
        </Card>

        {/* Card 2: Custom Login Email Template */}
        <Card className="p-6">
          <div className="flex items-center gap-2.5 border-b border-slate-200 pb-4 dark:border-slate-800">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-50 text-indigo-600 dark:bg-indigo-950 dark:text-indigo-400">
              <Mail className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                自定义登录邮件内容 (Email Template)
              </h2>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                自定义发送给管理员的验证码邮件主题与正文，邮件内容必须包含验证码占位符{' '}
                <code className="font-mono font-bold text-indigo-600 dark:text-indigo-400">
                  {'{code}'}
                </code>
                。
              </p>
            </div>
          </div>

          <div className="mt-5 space-y-4">
            <div>
              <label className="mb-1.5 block text-xs font-medium text-slate-700 dark:text-slate-300">
                登录邮件主题 (Subject)
              </label>
              <Input
                required
                value={settings.login_email_subject}
                onChange={(e) => setSettings({ ...settings, login_email_subject: e.target.value })}
                placeholder="【Foundry】登录验证码"
              />
            </div>

            <div>
              <div className="mb-1.5 flex items-center justify-between">
                <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
                  登录邮件内容正文 (Email Content Body)
                </label>
                {hasCodePlaceholder ? (
                  <span className="flex items-center gap-1 font-mono text-[11px] font-semibold text-emerald-600 dark:text-emerald-400">
                    <CheckCircle2 className="h-3.5 w-3.5" />
                    <span>已包含占位符 {'{code}'}</span>
                  </span>
                ) : (
                  <span className="flex items-center gap-1 font-mono text-[11px] font-semibold text-rose-600 dark:text-rose-400">
                    <AlertCircle className="h-3.5 w-3.5" />
                    <span>必须包含占位符 {'{code}'}</span>
                  </span>
                )}
              </div>
              <textarea
                rows={4}
                required
                value={settings.login_email_template}
                onChange={(e) => setSettings({ ...settings, login_email_template: e.target.value })}
                className={`w-full rounded-xl border p-3 font-sans text-xs focus:outline-none dark:bg-slate-950 dark:text-slate-100 ${
                  hasCodePlaceholder
                    ? 'border-slate-300 focus:border-emerald-500 dark:border-slate-800'
                    : 'border-rose-400 focus:border-rose-500 dark:border-rose-800'
                }`}
                placeholder="您的登录验证码是：{code}，5分钟内有效。"
              />
              <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
                提示：系统在向管理员发送邮件时，会自动将正文中的{' '}
                <code className="font-mono font-bold text-indigo-600 dark:text-indigo-400">
                  {'{code}'}
                </code>{' '}
                替换为 6 位数字验证码（如 849201）。
              </p>
            </div>

            {/* Live Email Preview Box */}
            <div className="mt-4 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900/60">
              <div className="mb-2.5 flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                <Eye className="h-3.5 w-3.5" />
                <span>实时效果预览 (Inbox Live Preview)</span>
              </div>
              <div className="rounded-lg border border-slate-200/80 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-950">
                <div className="border-b border-slate-100 pb-2.5 text-xs dark:border-slate-800/80">
                  <div className="text-[11px] text-slate-400">
                    发件人:{' '}
                    <span className="text-slate-600 dark:text-slate-300">
                      Foundry &lt;no-reply@foundry.local&gt;
                    </span>
                  </div>
                  <div className="text-[11px] text-slate-400">
                    收件人:{' '}
                    <span className="text-slate-600 dark:text-slate-300">admin@company.com</span>
                  </div>
                  <div className="mt-1.5 text-xs font-bold text-slate-900 dark:text-slate-100">
                    {settings.login_email_subject || '（未设置主题）'}
                  </div>
                </div>
                <div className="mt-3 whitespace-pre-wrap font-sans text-xs text-slate-700 dark:text-slate-300">
                  {settings.login_email_template.replace('{code}', '849201')}
                </div>
              </div>
            </div>
          </div>
        </Card>

        {/* Card 3: Email Service Dispatcher Info */}
        <Card className="p-6">
          <div className="flex items-center gap-2.5 border-b border-slate-200 pb-4 dark:border-slate-800">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-teal-50 text-teal-600 dark:bg-teal-950 dark:text-teal-400">
              <Server className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                邮件传输通道服务说明 (SMTP Gateway Integration)
              </h2>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                Foundry 原生支持 SMTP 邮件中继网关与本地开发调试模拟模式。
              </p>
            </div>
          </div>

          <div className="mt-4 space-y-2 text-xs text-slate-600 dark:text-slate-400">
            <p>
              • <strong>生产环境邮件发送：</strong>在服务端环境变量中配置{' '}
              <code className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
                SMTP_HOST
              </code>
              、
              <code className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
                SMTP_PORT
              </code>
              （默认 587）、
              <code className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
                SMTP_USER
              </code>
              、
              <code className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400">
                SMTP_PASS
              </code>
              ，系统将自动通过加密 SMTP 协议实时发送邮件。
            </p>
            <p>
              • <strong>开发/测试模式：</strong>当未配置 SMTP_HOST 时，系统将自动进入 Dev
              模式，将验证码与邮件正文安全打印到服务端日志中，方便开发联调。
            </p>
          </div>
        </Card>

        {/* Action Button */}
        <div className="flex justify-end gap-3 pt-2">
          <Button type="submit" disabled={saving || !canSave} size="lg">
            <Save className="h-4 w-4" />
            <span>{saving ? '保存中...' : '保存全局设置'}</span>
          </Button>
        </div>
      </form>
    </div>
  );
}
