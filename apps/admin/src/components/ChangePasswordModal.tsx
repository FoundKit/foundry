import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock, KeyRound, CheckCircle2, AlertCircle, AlertTriangle } from 'lucide-react';
import { Modal, Button, Input } from './UiWidgets';
import { api } from '../services/api';
import { AdminProfile } from '../types';

interface ChangePasswordModalProps {
  isOpen: boolean;
  onClose: () => void;
  admin: AdminProfile;
  onAdminUpdated: (updated: AdminProfile) => void;
  isInitialPrompt?: boolean;
}

export function ChangePasswordModal({
  isOpen,
  onClose,
  admin,
  onAdminUpdated,
  isInitialPrompt = false,
}: ChangePasswordModalProps) {
  const { t } = useTranslation();
  const [oldPassword, setOldPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Initialize or reset form when modal opens
  useEffect(() => {
    if (isOpen) {
      setOldPassword(admin.is_initial_password ? 'admin123456' : '');
      setNewPassword('');
      setConfirmPassword('');
      setError(null);
      setSuccessMsg(null);
    }
  }, [isOpen, admin.is_initial_password]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccessMsg(null);

    if (!oldPassword) {
      setError(t('auth.old_password') + '不能为空');
      return;
    }

    if (newPassword.length < 6) {
      setError(t('auth.password_too_short'));
      return;
    }

    if (newPassword === oldPassword) {
      setError(t('auth.password_cannot_be_same'));
      return;
    }

    if (newPassword !== confirmPassword) {
      setError(t('auth.passwords_do_not_match'));
      return;
    }

    setSubmitting(true);
    try {
      const updated = await api.changePassword({
        old_password: oldPassword,
        new_password: newPassword,
      });
      setSuccessMsg(t('auth.password_changed_success'));
      onAdminUpdated(updated);
      setTimeout(() => {
        onClose();
      }, 1200);
    } catch (err: any) {
      setError(err.message || '修改密码失败，请检查当前密码是否正确');
    } finally {
      setSubmitting(false);
    }
  };

  const modalTitle = isInitialPrompt
    ? t('auth.initial_password_prompt_title')
    : t('auth.change_password');

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={modalTitle}>
      <div className="space-y-4">
        {/* Security Warning Notice when using default password */}
        {isInitialPrompt && (
          <div className="rounded-xl border border-amber-300 bg-amber-50 p-3.5 text-xs text-amber-900 dark:border-amber-800/80 dark:bg-amber-950/60 dark:text-amber-200">
            <div className="flex items-start gap-2.5">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
              <div>
                <p className="font-semibold">{t('auth.initial_password_prompt_title')}</p>
                <p className="mt-1 leading-relaxed opacity-90">
                  {t('auth.initial_password_prompt_desc')}
                </p>
              </div>
            </div>
          </div>
        )}

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

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('auth.old_password')}
            </label>
            <div className="relative">
              <Lock className="absolute left-3.5 top-2.5 h-4 w-4 text-slate-400" />
              <Input
                type="password"
                required
                placeholder="••••••••"
                value={oldPassword}
                onChange={(e) => setOldPassword(e.target.value)}
                className="pl-10 font-mono"
              />
            </div>
            {admin.is_initial_password && (
              <p className="mt-1 text-[11px] text-amber-600 dark:text-amber-400">
                * 当前为系统初始默认密码（已自动为您填入）
              </p>
            )}
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('auth.new_password')}
            </label>
            <div className="relative">
              <KeyRound className="absolute left-3.5 top-2.5 h-4 w-4 text-slate-400" />
              <Input
                type="password"
                required
                placeholder="至少 6 位新密码"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                className="pl-10 font-mono"
              />
            </div>
            <p className="mt-1 text-[11px] text-slate-500 dark:text-slate-400">
              * 建议包含字母、数字或特殊字符，长度不少于 6 位
            </p>
          </div>

          <div>
            <label className="mb-1 block text-xs font-medium text-slate-700 dark:text-slate-300">
              {t('auth.confirm_new_password')}
            </label>
            <div className="relative">
              <KeyRound className="absolute left-3.5 top-2.5 h-4 w-4 text-slate-400" />
              <Input
                type="password"
                required
                placeholder="请再次输入新密码"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="pl-10 font-mono"
              />
            </div>
          </div>

          <div className="flex items-center justify-end gap-2 border-t border-slate-200 pt-4 dark:border-slate-800">
            <Button type="button" variant="ghost" onClick={onClose}>
              {isInitialPrompt ? '稍后修改' : t('common.cancel')}
            </Button>
            <Button type="submit" disabled={submitting || !oldPassword || !newPassword}>
              <span>{submitting ? t('common.loading') : t('common.confirm')}</span>
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
