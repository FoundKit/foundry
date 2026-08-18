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
    <div className="min-h-screen bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100 flex flex-col transition-colors duration-150">
      {/* Top Navbar */}
      <header className="h-16 border-b border-slate-200 dark:border-slate-800 bg-white/80 dark:bg-slate-900/80 backdrop-blur px-6 flex items-center justify-between sticky top-0 z-40">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-500 flex items-center justify-center font-bold text-white shadow-lg shadow-emerald-500/20">
              F
            </div>
            <div>
              <span className="font-bold text-slate-900 dark:text-slate-100 tracking-tight">FOUNDRY</span>
              <span className="ml-2 text-xs font-medium px-2 py-0.5 rounded bg-emerald-50 dark:bg-emerald-950/80 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/60">
                v0.1.0
              </span>
            </div>
          </div>

          {/* Sub-System Switcher Dropdown */}
          <div className="relative group">
            <button className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-100 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 text-slate-700 dark:text-slate-200 text-sm hover:bg-slate-200 dark:hover:bg-slate-700 transition shadow-sm dark:shadow-none">
              <span className="text-xs text-slate-500 dark:text-slate-400">System:</span>
              <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                {currentSystem
                  ? currentSystem.name
                  : systems.length === 0
                  ? t('app.no_systems')
                  : t('app.all_systems')}
              </span>
              <ChevronDown className="w-4 h-4 text-slate-400 dark:text-slate-400" />
            </button>

            <div className="absolute left-0 top-full mt-1.5 w-64 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl shadow-xl p-1.5 hidden group-hover:block z-50 animate-in fade-in-50 zoom-in-95 duration-100">
              <div className="px-3 py-1.5 text-xs font-semibold text-slate-400 dark:text-slate-500 border-b border-slate-100 dark:border-slate-800 mb-1">
                SWITCH ACTIVE SYSTEM
              </div>

              {systems.length > 0 ? (
                <>
                  <div className="max-h-60 overflow-y-auto space-y-0.5">
                    {systems.map((sys) => (
                      <button
                        key={sys.id}
                        onClick={() => onSelectSystem(sys)}
                        className={`w-full text-left px-3 py-2 rounded-lg text-sm transition flex items-center justify-between ${
                          currentSystem?.id === sys.id
                            ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-400 font-medium'
                            : 'text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'
                        }`}
                      >
                        <span className="truncate">{sys.name}</span>
                        <span className="text-xs font-mono text-slate-400 dark:text-slate-500">/{sys.slug}</span>
                      </button>
                    ))}
                  </div>
                  <div className="border-t border-slate-100 dark:border-slate-800 mt-1 pt-1">
                    <button
                      onClick={() => onTabChange('systems')}
                      className="w-full text-left px-3 py-1.5 rounded-lg text-xs text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40 flex items-center gap-1.5 transition"
                    >
                      <Layers className="w-3.5 h-3.5" />
                      <span>{t('app.manage_systems')}</span>
                    </button>
                  </div>
                </>
              ) : (
                <div className="p-3 text-center">
                  <p className="text-xs text-slate-500 dark:text-slate-400 mb-2">{t('app.no_systems')}</p>
                  <button
                    onClick={() => onTabChange('systems')}
                    className="w-full text-center px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-600 text-white hover:bg-emerald-500 transition flex items-center justify-center gap-1"
                  >
                    <PlusCircle className="w-3.5 h-3.5" />
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
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-slate-700 hover:text-slate-900 bg-white dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 dark:text-slate-300 dark:hover:text-white border border-slate-200 dark:border-slate-800 transition shadow-sm dark:shadow-none"
          >
            <Globe className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />
            <span>{i18n.language.startsWith('zh') ? '中文' : 'EN'}</span>
          </button>

          <a
            href="/api/v1/health"
            target="_blank"
            rel="noreferrer"
            className="hidden sm:flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs text-slate-600 hover:text-slate-900 bg-white dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 dark:text-slate-400 dark:hover:text-slate-200 border border-slate-200 dark:border-slate-800 transition shadow-sm dark:shadow-none"
          >
            <ExternalLink className="w-3.5 h-3.5" />
            <span>API Docs</span>
          </a>

          <div className="h-4 w-px bg-slate-200 dark:bg-slate-800 mx-1" />

          {/* User profile */}
          <div className="flex items-center gap-2 pl-1">
            <div className="w-8 h-8 rounded-full bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 flex items-center justify-center text-xs font-bold text-slate-700 dark:text-slate-200">
              {admin.username[0]?.toUpperCase()}
            </div>
            <div className="hidden sm:block text-left">
              <div className="text-xs font-semibold text-slate-800 dark:text-slate-200">{admin.username}</div>
              <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                {admin.role === 'super_admin' ? 'Super Admin' : 'Topic Admin'}
              </div>
            </div>
            <button
              onClick={onLogout}
              title={t('app.logout')}
              className="p-1.5 rounded-lg text-slate-400 hover:text-rose-500 dark:hover:text-rose-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition ml-1"
            >
              <LogOut className="w-4 h-4" />
            </button>
          </div>
        </div>
      </header>

      {/* Main Container */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Navigation Sidebar */}
        <aside className="w-60 border-r border-slate-200 dark:border-slate-800 bg-white/60 dark:bg-slate-900/40 p-3 space-y-1">
          {navItems.map((item) => {
            if (item.superOnly && admin.role !== 'super_admin') return null;
            const Icon = item.icon;
            const isActive = currentTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onTabChange(item.id)}
                className={`w-full flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition ${
                  isActive
                    ? 'bg-emerald-50 text-emerald-700 border border-emerald-200/80 shadow-sm dark:bg-emerald-600/15 dark:text-emerald-400 dark:border-emerald-500/20 dark:shadow-none'
                    : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-900'
                }`}
              >
                <Icon className={`w-4 h-4 ${isActive ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400 dark:text-slate-400'}`} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </aside>

        {/* Content View */}
        <main className="flex-1 overflow-y-auto p-8 bg-slate-50 dark:bg-slate-950">
          <div className="max-w-7xl mx-auto">{children}</div>
        </main>
      </div>
    </div>
  );
}
