import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Mail, KeyRound, ShieldCheck, CheckCircle2, AlertCircle, RefreshCw } from 'lucide-react';
import { Modal, Button, Input, Badge } from './UiWidgets';
import { api } from '../services/api';
import { AdminProfile } from '../types';

interface ProfileBindEmailModalProps {
  isOpen: boolean;
  onClose: () => void;
  admin: AdminProfile;
  onAdminUpdated: (updated: AdminProfile) => void;
  onOpenChangePassword?: () => void;
}

export function ProfileBindEmailModal({
  isOpen,
  onClose,
  admin,
  onAdminUpdated,
  onOpenChangePassword,
}: ProfileBindEmailModalProps) {
  const { t } = useTranslation();
  const [email, setEmail] = useState('');
  const [code, setCode] = useState('');
  const [sendingCode, setSendingCode] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [cooldown, setCooldown] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Reset form when modal opens
  useEffect(() => {
    if (isOpen) {
      setEmail('');
      setCode('');
      setError(null);
      setSuccessMsg(null);
    }
  }, [isOpen]);

  // Cooldown countdown timer
  useEffect(() => {
    if (cooldown <= 0) return;
    const timer = setInterval(() => {
      setCooldown((prev) => prev - 1);
    }, 1000);
    return () => clearInterval(timer);
  }, [cooldown]);

  const handleSendCode = async () => {
    if (!email || !email.includes('@')) {
      setError('请输入有效的邮箱地址 (Please enter a valid email address)');
      return;
    }
    setError(null);
    setSuccessMsg(null);
    setSendingCode(true);
    try {
      const res = await api.sendBindEmailCode(email.trim());
      setSuccessMsg(res.message || '绑定验证码已发送至您的邮箱，5分钟内有效');
      setCooldown(60);
    } catch (err: any) {
      setError(err.message || '发送验证码失败');
    } finally {
      setSendingCode(false);
    }
  };

  const handleVerifyBind = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email || !email.includes('@')) {
      setError('请输入有效的邮箱地址');
      return;
    }
    if (!code || code.trim().length === 0) {
      setError('请输入邮箱验证码');
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      const updated = await api.verifyBindEmail({
        email: email.trim(),
        code: code.trim(),
      });
      setSuccessMsg('邮箱绑定成功！已更新至您的管理员账户。');
      onAdminUpdated(updated);
      setEmail('');
      setCode('');
      setTimeout(() => {
        onClose();
      }, 1500);
    } catch (err: any) {
      setError(err.message || '邮箱验证绑定失败');
    } finally {
      setSubmitting(false);
    }
  };

  const getRoleLabel = (role: AdminProfile['role']) => {
    switch (role) {
      case 'super_admin':
        return t('admins.super_admin');
      case 'admin':
        return t('admins.normal_admin');
      case 'topic_admin':
        return t('admins.topic_admin');
      default:
        return role;
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="个人中心与邮箱绑定">
      <div className="space-y-5">
        {/* Current Account Profile Banner */}
        <div className="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900/60">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-500 font-bold text-white shadow-md shadow-emerald-500/20">
                {admin.username[0]?.toUpperCase()}
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-slate-900 dark:text-slate-100">
                    {admin.username}
                  </span>
                  <Badge variant="purple">{getRoleLabel(admin.role)}</Badge>
                </div>
                <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
                  管理员 ID: <code className="font-mono text-[11px]">{admin.id}</code>
                </div>
              </div>
            </div>
          </div>

          <div className="mt-3 flex items-center justify-between border-t border-slate-200/80 pt-3 text-xs dark:border-slate-800/80">
            <span className="text-slate-500 dark:text-slate-400">当前绑定邮箱：</span>
            {admin.email ? (
              <span className="flex items-center gap-1.5 font-medium text-emerald-600 dark:text-emerald-400">
                <CheckCircle2 className="h-3.5 w-3.5" />
                <span>{admin.email}</span>
                <span className="py-0.2 rounded bg-emerald-100 px-1.5 text-[10px] text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300">
                  已验证
                </span>
              </span>
            ) : (
              <span className="flex items-center gap-1.5 text-amber-600 dark:text-amber-400">
                <AlertCircle className="h-3.5 w-3.5" />
                <span>暂未绑定邮箱（无法使用邮箱登录）</span>
              </span>
            )}
          </div>

          <div className="mt-2.5 flex items-center justify-between border-t border-slate-200/80 pt-2.5 text-xs dark:border-slate-800/80">
            <span className="text-slate-500 dark:text-slate-400">账号登录密码：</span>
            <div className="flex items-center gap-2">
              {admin.is_initial_password ? (
                <span className="text-[11px] font-medium text-amber-600 dark:text-amber-400">
                  初始默认密码
                </span>
              ) : (
                <span className="text-[11px] text-slate-500 dark:text-slate-400">
                  已设置自定义密码
                </span>
              )}
              {onOpenChangePassword && (
                <button
                  type="button"
                  onClick={() => {
                    onClose();
                    onOpenChangePassword();
                  }}
                  className="rounded border border-slate-200 bg-white px-2 py-0.5 text-[11px] font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-emerald-600 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700 dark:hover:text-emerald-400"
                >
                  {t('auth.change_password')}
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Feedback Messages */}
        {error && (
          <div className="flex items-center gap-2 rounded-lg border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800/60 dark:bg-rose-950/60 dark:text-rose-300">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {successMsg && (
          <div className="flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-xs text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
            <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400" />
            <span>{successMsg}</span>
          </div>
        )}

        {/* Bind / Change Email Form */}
        <form onSubmit={handleVerifyBind} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {admin.email ? '更换绑定的新邮箱 (New Email)' : '绑定邮箱地址 (Email Address)'}
            </label>
            <div className="relative">
              <Mail className="absolute left-3.5 top-2.5 h-4 w-4 text-slate-400" />
              <Input
                type="email"
                required
                placeholder="your_email@company.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="pl-10"
              />
            </div>
            <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
              * 绑定后该邮箱可用于接收登录验证码及安全通知。
            </p>
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              邮箱验证码 (Verification Code)
            </label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <KeyRound className="absolute left-3.5 top-2.5 h-4 w-4 text-slate-400" />
                <Input
                  type="text"
                  maxLength={6}
                  required
                  placeholder="6 位验证码"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  className="pl-10 font-mono font-semibold tracking-widest"
                />
              </div>
              <button
                type="button"
                disabled={sendingCode || cooldown > 0 || !email}
                onClick={handleSendCode}
                className="shrink-0 rounded-lg border border-slate-200 bg-slate-50 px-3.5 py-2 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-50 dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
              >
                {sendingCode ? (
                  <span className="flex items-center gap-1">
                    <RefreshCw className="h-3 w-3 animate-spin" />
                    <span>发送中...</span>
                  </span>
                ) : cooldown > 0 ? (
                  `${cooldown}s`
                ) : (
                  '获取验证码'
                )}
              </button>
            </div>
          </div>

          <div className="flex items-center justify-end gap-2 border-t border-slate-200 pt-4 dark:border-slate-800">
            <Button type="button" variant="ghost" onClick={onClose}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={submitting || !email || !code}>
              <ShieldCheck className="mr-1 h-4 w-4" />
              <span>{submitting ? '验证中...' : '验证并绑定'}</span>
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
