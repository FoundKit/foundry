import React, { useState, useEffect } from 'react';
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
  ArrowLeft,
  Compass,
  Code2,
  Settings,
  Sparkles,
} from 'lucide-react';
import { AdminProfile, CustomAdminPageItem, RouteState, SystemItem } from '../types';
import { api } from '../services/api';
import { ThemeToggle } from './ThemeToggle';

interface LayoutProps {
  route: RouteState;
  onNavigatePlatform: (tab: RouteState['platformTab'], params?: Record<string, any>) => void;
  onNavigateSubsystem: (
    slug: string,
    tab?: RouteState['subsystemTab'],
    params?: Record<string, any>,
    replace?: boolean,
    customPageKey?: string,
  ) => void;
  admin: AdminProfile;
  systems: SystemItem[];
  currentSystem: SystemItem | null;
  onSelectSystem: (system: SystemItem) => void;
  onLogout: () => void;
  children: React.ReactNode;
}

export function Layout({
  route,
  onNavigatePlatform,
  onNavigateSubsystem,
  admin,
  systems,
  currentSystem,
  onSelectSystem,
  onLogout,
  children,
}: LayoutProps) {
  const { t, i18n } = useTranslation();
  const [customPages, setCustomPages] = useState<CustomAdminPageItem[]>([]);

  useEffect(() => {
    if (currentSystem?.slug) {
      api.listCustomPages(currentSystem.slug).then(setCustomPages).catch(() => setCustomPages([]));
    } else {
      setCustomPages([]);
    }
  }, [currentSystem?.slug]);

  const toggleLanguage = () => {
    const nextLang = i18n.language.startsWith('zh') ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(nextLang);
  };


  const isSubsystemMode = route.mode === 'subsystem' && currentSystem !== null;

  // Platform navigation items with strict role gating
  const platformNavItems = [
    { id: 'dashboard' as const, label: t('nav.dashboard'), icon: LayoutDashboard },
    { id: 'systems' as const, label: t('nav.systems'), icon: Layers },
    { id: 'audit_logs' as const, label: t('nav.audit_logs'), icon: FileClock, hideForTopicAdmin: true },
    { id: 'admins' as const, label: t('nav.admins'), icon: ShieldCheck, superOnly: true },
  ];

  // Subsystem navigation items
  const subsystemNavItems = [
    { id: 'overview' as const, label: t('nav.sub_overview'), icon: Compass },
    { id: 'configs' as const, label: t('nav.configs'), icon: Sliders },
    { id: 'models' as const, label: t('nav.models'), icon: Database },
    { id: 'data' as const, label: t('nav.data_explorer'), icon: TableProperties },
    { id: 'apis' as const, label: t('nav.sub_apis'), icon: Code2 },
    { id: 'audit_logs' as const, label: t('nav.sub_audit'), icon: FileClock },
    { id: 'settings' as const, label: t('nav.sub_settings'), icon: Settings },
  ];

  const getRoleDisplayName = (role: AdminProfile['role']) => {
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
    <div className="flex min-h-screen flex-col bg-slate-50 text-slate-900 transition-colors duration-150 dark:bg-slate-950 dark:text-slate-100">
      {/* Top Navbar */}
      <header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-slate-200 bg-white/80 px-6 backdrop-blur dark:border-slate-800 dark:bg-slate-900/80">
        <div className="flex items-center gap-4 sm:gap-6">
          {/* Logo / Brand */}
          <div
            onClick={() => onNavigatePlatform('dashboard')}
            className="flex cursor-pointer items-center gap-3"
          >
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-500 font-bold text-white shadow-lg shadow-emerald-500/20">
              F
            </div>
            <div className="hidden sm:block">
              <div className="flex items-center gap-2">
                <span className="font-bold tracking-tight text-slate-900 dark:text-slate-100">
                  FOUNDRY
                </span>
                <span className="rounded border border-emerald-200 bg-emerald-50 px-1.5 py-0.5 font-mono text-[10px] font-medium text-emerald-700 dark:border-emerald-800/60 dark:bg-emerald-950/80 dark:text-emerald-400">
                  v0.1.0
                </span>
              </div>
              <div className="text-[10px] text-slate-500 dark:text-slate-400">
                {isSubsystemMode ? t('app.subsystem_hub') : t('app.platform_hub')}
              </div>
            </div>
          </div>

          <div className="h-6 w-px bg-slate-200 dark:bg-slate-800" />

          {/* Subsystem Mode Navigation Breadcrumb */}
          {isSubsystemMode && (
            <div className="flex items-center gap-3">
              <button
                onClick={() => onNavigatePlatform('systems')}
                className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-white"
              >
                <ArrowLeft className="h-3.5 w-3.5" />
                <span>{t('app.back_to_platform')}</span>
              </button>

              <div className="flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50/80 px-3 py-1.5 text-xs text-emerald-800 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300">
                <span className="text-[10px] text-emerald-600 dark:text-emerald-400">
                  Subsystem:
                </span>
                <span className="font-semibold">{currentSystem.name}</span>
                <span className="font-mono text-[10px] opacity-70">/{currentSystem.slug}</span>
              </div>
            </div>
          )}
        </div>

        {/* Right Tools */}
        <div className="flex items-center gap-2.5">
          <ThemeToggle />

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

          {/* User Profile */}
          <div className="flex items-center gap-2 pl-1">
            <div className="flex h-8 w-8 items-center justify-center rounded-full border border-slate-200 bg-slate-100 text-xs font-bold text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200">
              {admin.username[0]?.toUpperCase()}
            </div>
            <div className="hidden text-left sm:block">
              <div className="text-xs font-semibold text-slate-800 dark:text-slate-200">
                {admin.username}
              </div>
              <div className="text-[10px] text-slate-500 dark:text-slate-400">
                {getRoleDisplayName(admin.role)}
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
        <aside className="w-64 space-y-4 border-r border-slate-200 bg-white/60 p-3.5 dark:border-slate-800 dark:bg-slate-900/40">
          {isSubsystemMode ? (
            <>
              {/* Subsystem Realm Indicator */}
              <div className="rounded-xl border border-emerald-200/80 bg-emerald-50/60 p-3 dark:border-emerald-800/40 dark:bg-emerald-950/40">
                <div className="text-[10px] font-bold uppercase tracking-wider text-emerald-700 dark:text-emerald-400">
                  {t('nav.subsystem_section')}
                </div>
                <div className="mt-1 truncate text-xs font-bold text-slate-900 dark:text-slate-100">
                  {currentSystem.name}
                </div>
                <div className="mt-0.5 font-mono text-[10px] text-emerald-600 dark:text-emerald-400">
                  /{currentSystem.slug}
                </div>
              </div>

              {/* Subsystem Navigation */}
              <div className="space-y-1">
                {subsystemNavItems.map((item) => {
                  const Icon = item.icon;
                  const isActive = route.subsystemTab === item.id;
                  return (
                    <button
                      key={item.id}
                      onClick={() => onNavigateSubsystem(currentSystem.slug, item.id)}
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
              </div>

              {/* Subsystem Custom Admin Pages Section */}
              {customPages.length > 0 && (
                <div className="border-t border-slate-200 pt-3 dark:border-slate-800">
                  <div className="px-2 pb-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 flex items-center justify-between">
                    <span>✨ 自定义后台 / Custom Pages</span>
                    <span className="rounded bg-indigo-100 px-1 py-0.2 text-[9px] text-indigo-700 dark:bg-indigo-950 dark:text-indigo-300 font-mono">
                      {customPages.length}
                    </span>
                  </div>
                  <div className="space-y-1">
                    {customPages.map((page) => {
                      const isActive =
                        route.subsystemTab === 'custom' && route.customPageKey === page.key;
                      return (
                        <button
                          key={page.key}
                          onClick={() =>
                            onNavigateSubsystem(
                              currentSystem.slug,
                              'custom',
                              undefined,
                              false,
                              page.key,
                            )
                          }
                          className={`flex w-full items-center gap-3 rounded-xl px-3.5 py-2 text-xs font-medium transition ${
                            isActive
                              ? 'border border-indigo-200/80 bg-indigo-50 text-indigo-700 shadow-sm dark:border-indigo-500/20 dark:bg-indigo-600/15 dark:text-indigo-300'
                              : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900 dark:text-slate-400 dark:hover:bg-slate-900 dark:hover:text-slate-200'
                          }`}
                        >
                          <Sparkles
                            className={`h-3.5 w-3.5 ${
                              isActive ? 'text-indigo-600 dark:text-indigo-400' : 'text-slate-400'
                            }`}
                          />
                          <span className="truncate">{page.title}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Back to Platform Central button */}
              <div className="border-t border-slate-200 pt-3 dark:border-slate-800">
                <button
                  onClick={() => onNavigatePlatform('systems')}
                  className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-xs font-medium text-slate-500 transition hover:bg-slate-100 hover:text-slate-900 dark:text-slate-400 dark:hover:bg-slate-900 dark:hover:text-slate-200"
                >
                  <ArrowLeft className="h-3.5 w-3.5" />
                  <span>{t('app.back_to_platform')}</span>
                </button>
              </div>

            </>
          ) : (
            <>
              {/* Platform Control Plane Indicator */}
              <div className="px-2 py-1 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
                {t('nav.platform_section')}
              </div>

              {/* Platform Navigation */}
              <div className="space-y-1">
                {platformNavItems.map((item) => {
                  if (item.superOnly && admin.role !== 'super_admin') return null;
                  if (item.hideForTopicAdmin && admin.role === 'topic_admin') return null;
                  const Icon = item.icon;
                  const isActive = route.platformTab === item.id;
                  return (
                    <button
                      key={item.id}
                      onClick={() => onNavigatePlatform(item.id)}
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
              </div>
            </>
          )}
        </aside>

        {/* Content View */}
        <main className="flex-1 overflow-y-auto bg-slate-50 p-8 dark:bg-slate-950">
          <div className="mx-auto max-w-7xl">{children}</div>
        </main>
      </div>
    </div>
  );
}
