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
  ArrowLeft,
  Compass,
  Code2,
  Settings,
} from 'lucide-react';
import { AdminProfile, RouteState, SystemItem } from '../types';
import { ThemeToggle } from './ThemeToggle';

interface LayoutProps {
  route: RouteState;
  onNavigatePlatform: (tab: RouteState['platformTab'], params?: Record<string, any>) => void;
  onNavigateSubsystem: (
    slug: string,
    tab?: RouteState['subsystemTab'],
    params?: Record<string, any>,
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

  const toggleLanguage = () => {
    const nextLang = i18n.language.startsWith('zh') ? 'en-US' : 'zh-CN';
    i18n.changeLanguage(nextLang);
  };

  const isSubsystemMode = route.mode === 'subsystem' && currentSystem !== null;

  // Platform navigation items
  const platformNavItems = [
    { id: 'dashboard' as const, label: t('nav.dashboard'), icon: LayoutDashboard },
    { id: 'systems' as const, label: t('nav.systems'), icon: Layers },
    { id: 'audit_logs' as const, label: t('nav.audit_logs'), icon: FileClock },
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

          {/* Mode Switcher / Breadcrumbs */}
          {isSubsystemMode ? (
            <div className="flex items-center gap-3">
              <button
                onClick={() => onNavigatePlatform('systems')}
                className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-white"
              >
                <ArrowLeft className="h-3.5 w-3.5" />
                <span>{t('app.back_to_platform')}</span>
              </button>

              {/* Subsystem Switcher Dropdown */}
              <div className="group relative">
                <button className="flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50/80 px-3 py-1.5 text-xs text-emerald-800 shadow-sm transition hover:bg-emerald-100 dark:border-emerald-800/60 dark:bg-emerald-950/60 dark:text-emerald-300 dark:hover:bg-emerald-900/60">
                  <span className="text-[10px] text-emerald-600 dark:text-emerald-400">
                    Subsystem:
                  </span>
                  <span className="font-semibold">{currentSystem.name}</span>
                  <span className="font-mono text-[10px] opacity-70">/{currentSystem.slug}</span>
                  <ChevronDown className="h-3.5 w-3.5" />
                </button>

                <div className="animate-in fade-in-50 zoom-in-95 absolute left-0 top-full z-50 mt-1.5 hidden w-72 rounded-xl border border-slate-200 bg-white p-1.5 shadow-xl duration-100 group-hover:block dark:border-slate-800 dark:bg-slate-900">
                  <div className="mb-1 border-b border-slate-100 px-3 py-1.5 text-[11px] font-semibold text-slate-400 dark:border-slate-800 dark:text-slate-500">
                    SWITCH ACTIVE SUBSYSTEM
                  </div>
                  <div className="max-h-60 space-y-0.5 overflow-y-auto">
                    {systems.map((sys) => (
                      <button
                        key={sys.id}
                        onClick={() => onNavigateSubsystem(sys.slug, route.subsystemTab)}
                        className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-xs transition ${
                          currentSystem?.id === sys.id
                            ? 'bg-emerald-50 font-medium text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-400'
                            : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
                        }`}
                      >
                        <span className="truncate">{sys.name}</span>
                        <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500">
                          /{sys.slug}
                        </span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ) : (
            /* Sub-System Switcher Dropdown in Platform Mode */
            <div className="group relative">
              <button className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-100 px-3 py-1.5 text-xs text-slate-700 shadow-sm transition hover:bg-slate-200 dark:border-slate-700/80 dark:bg-slate-800/80 dark:text-slate-200 dark:shadow-none dark:hover:bg-slate-700">
                <span className="text-[10px] text-slate-500 dark:text-slate-400">Quick Jump:</span>
                <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                  {currentSystem ? currentSystem.name : t('app.all_systems')}
                </span>
                <ChevronDown className="h-3.5 w-3.5 text-slate-400" />
              </button>

              <div className="animate-in fade-in-50 zoom-in-95 absolute left-0 top-full z-50 mt-1.5 hidden w-72 rounded-xl border border-slate-200 bg-white p-1.5 shadow-xl duration-100 group-hover:block dark:border-slate-800 dark:bg-slate-900">
                <div className="mb-1 border-b border-slate-100 px-3 py-1.5 text-[11px] font-semibold text-slate-400 dark:border-slate-800 dark:text-slate-500">
                  OPEN SUBSYSTEM CONSOLE
                </div>

                {systems.length > 0 ? (
                  <>
                    <div className="max-h-60 space-y-0.5 overflow-y-auto">
                      {systems.map((sys) => (
                        <button
                          key={sys.id}
                          onClick={() => {
                            onSelectSystem(sys);
                            onNavigateSubsystem(sys.slug, 'overview');
                          }}
                          className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-xs transition hover:bg-emerald-50 hover:text-emerald-700 dark:hover:bg-emerald-950/50 dark:hover:text-emerald-400"
                        >
                          <span className="truncate">{sys.name}</span>
                          <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500">
                            /{sys.slug}
                          </span>
                        </button>
                      ))}
                    </div>
                    <div className="mt-1 border-t border-slate-100 pt-1 dark:border-slate-800">
                      <button
                        onClick={() => onNavigatePlatform('systems')}
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
                      onClick={() => onNavigatePlatform('systems')}
                      className="flex w-full items-center justify-center gap-1 rounded-lg bg-emerald-600 px-3 py-1.5 text-center text-xs font-medium text-white transition hover:bg-emerald-500"
                    >
                      <PlusCircle className="h-3.5 w-3.5" />
                      <span>{t('systems.create')}</span>
                    </button>
                  </div>
                )}
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

              {/* Active Sub-Systems Quick Jump List */}
              {systems.length > 0 && (
                <div className="space-y-2 border-t border-slate-200 pt-4 dark:border-slate-800">
                  <div className="flex items-center justify-between px-2 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
                    <span>{t('nav.systems')}</span>
                    <span className="font-mono">{systems.length}</span>
                  </div>
                  <div className="max-h-48 space-y-1 overflow-y-auto">
                    {systems.slice(0, 5).map((sys) => (
                      <button
                        key={sys.id}
                        onClick={() => {
                          onSelectSystem(sys);
                          onNavigateSubsystem(sys.slug, 'overview');
                        }}
                        className="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-xs text-slate-600 transition hover:bg-slate-100 hover:text-emerald-600 dark:text-slate-400 dark:hover:bg-slate-900 dark:hover:text-emerald-400"
                      >
                        <span className="truncate">{sys.name}</span>
                        <span className="font-mono text-[10px] text-slate-400">/{sys.slug}</span>
                      </button>
                    ))}
                  </div>
                </div>
              )}
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
