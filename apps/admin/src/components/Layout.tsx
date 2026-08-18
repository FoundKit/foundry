import React from 'react';
import { useTranslation } from 'react-i18next';
import {
  Layers,
  Sliders,
  Database,
  TableProperties,
  ShieldCheck,
  FileClock,
  LogOut,
  Globe,
  LayoutDashboard,
  ExternalLink,
  ChevronDown,
  PlusCircle,
} from 'lucide-react';
import { AdminProfile, SystemItem } from '../types';
import { ThemeToggle } from './ThemeToggle';

interface LayoutProps {
  currentTab: string;
  onTabChange: (tab: string) => void;
  admin: AdminProfile;
  systems: SystemItem[];
  currentSystem: SystemItem | null;
  onSelectSystem: (system: SystemItem) => void;
  onLogout: () => void;
  children: React.ReactNode;
}

export function Layout({
  currentTab,
  onTabChange,
  admin,
  systems,
  currentSystem,
  onSelectSystem,
  onLogout,
  children,
}: LayoutProps) {
  const { t, i18n } = useTranslation();

  const toggleLanguage = () => {
    const nextLang = i18n.language.startsWith('zh') ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(nextLang);
  };

  const navItems = [
    { id: 'dashboard', label: t('nav.dashboard'), icon: LayoutDashboard },
    { id: 'systems', label: t('nav.systems'), icon: Layers },
    { id: 'configs', label: t('nav.configs'), icon: Sliders },
    { id: 'models', label: t('nav.models'), icon: Database },
    { id: 'data_explorer', label: t('nav.data_explorer'), icon: TableProperties },
    { id: 'admins', label: t('nav.admins'), icon: ShieldCheck, superOnly: true },
    { id: 'audit_logs', label: t('nav.audit_logs'), icon: FileClock },
  ];

  return (
    <div className="flex min-h-screen flex-col bg-slate-50 text-slate-900 transition-colors duration-150 dark:bg-slate-950 dark:text-slate-100">
      {/* Top Navbar */}
      <header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-slate-200 bg-white/80 px-6 backdrop-blur dark:border-slate-800 dark:bg-slate-900/80">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-500 font-bold text-white shadow-lg shadow-emerald-500/20">
              F
            </div>
            <div>
              <span className="font-bold tracking-tight text-slate-900 dark:text-slate-100">
                FOUNDRY
              </span>
              <span className="ml-2 rounded border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/80 dark:text-emerald-400">
                v0.1.0
              </span>
            </div>
          </div>

          {/* Sub-System Switcher Dropdown */}
          <div className="group relative">
            <button className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-100 px-3 py-1.5 text-sm text-slate-700 shadow-sm transition hover:bg-slate-200 dark:border-slate-700/80 dark:bg-slate-800/80 dark:text-slate-200 dark:shadow-none dark:hover:bg-slate-700">
              <span className="text-xs text-slate-500 dark:text-slate-400">System:</span>
              <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                {currentSystem
                  ? currentSystem.name
                  : systems.length === 0
                    ? t('app.no_systems')
                    : t('app.all_systems')}
              </span>
              <ChevronDown className="h-4 w-4 text-slate-400 dark:text-slate-400" />
            </button>

            <div className="animate-in fade-in-50 zoom-in-95 absolute left-0 top-full z-50 mt-1.5 hidden w-64 rounded-xl border border-slate-200 bg-white p-1.5 shadow-xl duration-100 group-hover:block dark:border-slate-800 dark:bg-slate-900">
              <div className="mb-1 border-b border-slate-100 px-3 py-1.5 text-xs font-semibold text-slate-400 dark:border-slate-800 dark:text-slate-500">
                SWITCH ACTIVE SYSTEM
              </div>

              {systems.length > 0 ? (
                <>
                  <div className="max-h-60 space-y-0.5 overflow-y-auto">
                    {systems.map((sys) => (
                      <button
                        key={sys.id}
                        onClick={() => onSelectSystem(sys)}
                        className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition ${
                          currentSystem?.id === sys.id
                            ? 'bg-emerald-50 font-medium text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-400'
                            : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
                        }`}
                      >
                        <span className="truncate">{sys.name}</span>
                        <span className="font-mono text-xs text-slate-400 dark:text-slate-500">
                          /{sys.slug}
                        </span>
                      </button>
                    ))}
                  </div>
                  <div className="mt-1 border-t border-slate-100 pt-1 dark:border-slate-800">
                    <button
                      onClick={() => onTabChange('systems')}
                      className="flex w-full items-center gap-1.5 rounded-lg px-3 py-1.5 text-left text-xs text-emerald-600 transition hover:bg-emerald-50 dark:text-emerald-400 dark:hover:bg-emerald-950/40"
                    >
                      <Layers className="h-3.5 w-3.5" />
                      <span>{t('app.manage_systems')}</span>
                    </button>
                  </div>
                </>
              ) : (
                <div className="p-3 text-center">
                  <p className="mb-2 text-xs text-slate-500 dark:text-slate-400">
                    {t('app.no_systems')}
                  </p>
                  <button
                    onClick={() => onTabChange('systems')}
                    className="flex w-full items-center justify-center gap-1 rounded-lg bg-emerald-600 px-3 py-1.5 text-center text-xs font-medium text-white transition hover:bg-emerald-500"
                  >
                    <PlusCircle className="h-3.5 w-3.5" />
                    <span>{t('systems.create')}</span>
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Right tools */}
        <div className="flex items-center gap-2.5">
          {/* Theme switcher */}
          <ThemeToggle />

          {/* Language Switcher */}
          <button
            onClick={toggleLanguage}
            className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-300 dark:shadow-none dark:hover:bg-slate-800 dark:hover:text-white"
          >
            <Globe className="h-3.5 w-3.5 text-slate-500 dark:text-slate-400" />
            <span>{i18n.language.startsWith('zh') ? '中文' : 'EN'}</span>
          </button>

          <a
            href="/api/v1/health"
            target="_blank"
            rel="noreferrer"
            className="hidden items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs text-slate-600 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-400 dark:shadow-none dark:hover:bg-slate-800 dark:hover:text-slate-200 sm:flex"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            <span>API Docs</span>
          </a>

          <div className="mx-1 h-4 w-px bg-slate-200 dark:bg-slate-800" />

          {/* User profile */}
          <div className="flex items-center gap-2 pl-1">
            <div className="flex h-8 w-8 items-center justify-center rounded-full border border-slate-200 bg-slate-100 text-xs font-bold text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200">
              {admin.username[0]?.toUpperCase()}
            </div>
            <div className="hidden text-left sm:block">
              <div className="text-xs font-semibold text-slate-800 dark:text-slate-200">
                {admin.username}
              </div>
              <div className="font-mono text-[10px] text-slate-500 dark:text-slate-400">
                {admin.role === 'super_admin' ? 'Super Admin' : 'Topic Admin'}
              </div>
            </div>
            <button
              onClick={onLogout}
              title={t('app.logout')}
              className="ml-1 rounded-lg p-1.5 text-slate-400 transition hover:bg-slate-100 hover:text-rose-500 dark:hover:bg-slate-800 dark:hover:text-rose-400"
            >
              <LogOut className="h-4 w-4" />
            </button>
          </div>
        </div>
      </header>

      {/* Main Container */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Navigation Sidebar */}
        <aside className="w-60 space-y-1 border-r border-slate-200 bg-white/60 p-3 dark:border-slate-800 dark:bg-slate-900/40">
          {navItems.map((item) => {
            if (item.superOnly && admin.role !== 'super_admin') return null;
            const Icon = item.icon;
            const isActive = currentTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onTabChange(item.id)}
                className={`flex w-full items-center gap-3 rounded-xl px-3.5 py-2.5 text-sm font-medium transition ${
                  isActive
                    ? 'border border-emerald-200/80 bg-emerald-50 text-emerald-700 shadow-sm dark:border-emerald-500/20 dark:bg-emerald-600/15 dark:text-emerald-400 dark:shadow-none'
                    : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-400 dark:hover:bg-slate-900 dark:hover:text-slate-200'
                }`}
              >
                <Icon
                  className={`h-4 w-4 ${isActive ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400 dark:text-slate-400'}`}
                />
                <span>{item.label}</span>
              </button>
            );
          })}
        </aside>

        {/* Content View */}
        <main className="flex-1 overflow-y-auto bg-slate-50 p-8 dark:bg-slate-950">
          <div className="mx-auto max-w-7xl">{children}</div>
        </main>
      </div>
    </div>
  );
}
