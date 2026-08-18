import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Layers,
  Database,
  ShieldCheck,
  FileClock,
  ArrowUpRight,
  PlusCircle,
  Search,
  ArrowRight,
  Compass,
} from 'lucide-react';
import { Card, Badge, Button, Input } from '../components/UiWidgets';
import { api } from '../services/api';
import { AdminProfile, PlatformSummary, RouteState, SystemItem } from '../types';

interface DashboardPageProps {
  admin: AdminProfile;
  systems: SystemItem[];
  onNavigatePlatform: (tab: RouteState['platformTab'], params?: Record<string, any>) => void;
  onNavigateSubsystem: (slug: string, tab?: string) => void;
}

export function DashboardPage({
  admin,
  systems,
  onNavigatePlatform,
  onNavigateSubsystem,
}: DashboardPageProps) {
  const { t } = useTranslation();
  const [summary, setSummary] = useState<PlatformSummary | null>(null);
  const [quickSearch, setQuickSearch] = useState('');

  useEffect(() => {
    if (admin.role === 'super_admin') {
      api
        .getPlatformSummary()
        .then(setSummary)
        .catch(() => setSummary(null));
    }
  }, [admin.role]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (quickSearch.trim()) {
      onNavigatePlatform('systems', { keyword: quickSearch.trim() });
    } else {
      onNavigatePlatform('systems');
    }
  };

  const statCards: Array<{
    title: string;
    value: string | number;
    desc: string;
    icon: any;
    tab: RouteState['platformTab'];
    superOnly?: boolean;
  }> = [
    {
      title: t('nav.systems'),
      value: summary ? summary.total_systems : systems.length,
      desc: `${summary ? summary.active_systems : systems.filter((s) => s.status === 1).length} Active Realms`,
      icon: Layers,
      tab: 'systems',
    },
    {
      title: t('nav.models'),
      value: summary ? summary.total_models : 'Zero-DDL',
      desc: 'Platform Dynamic Collections',
      icon: Database,
      tab: 'systems',
    },
    {
      title: t('nav.admins'),
      value: summary
        ? summary.total_admins
        : admin.role === 'super_admin'
          ? 'Super Admin'
          : 'Topic Admin',
      desc: 'Hierarchical IAM & RBAC',
      icon: ShieldCheck,
      tab: 'admins',
      superOnly: true,
    },
    {
      title: t('nav.audit_logs'),
      value: summary ? summary.total_audit_logs : 'Non-GET',
      desc: 'Immutable Write Trail',
      icon: FileClock,
      tab: 'audit_logs',
    },
  ];

  return (
    <div className="space-y-8">
      {/* Header Banner */}
      <div className="relative overflow-hidden rounded-2xl border border-slate-200 bg-gradient-to-r from-emerald-50 via-slate-50 to-slate-100 p-6 shadow-sm dark:border-slate-800 dark:from-slate-900 dark:via-slate-900 dark:to-emerald-950/40">
        <div className="relative z-10">
          <div className="mb-2 flex items-center gap-2">
            <Badge variant="success">Foundry Control Plane</Badge>
            <span className="text-xs text-slate-500 dark:text-slate-400">
              REST-First · Zero-DDL · Subsystem Autonomy
            </span>
          </div>
          <h1 className="text-2xl font-bold text-slate-900 dark:text-slate-100">
            Welcome back, {admin.username}
          </h1>
          <p className="mt-1 max-w-2xl text-sm text-slate-600 dark:text-slate-400">
            Foundry Platform Central Hub. Manage all independent sub-systems, audit write
            operations, delegate topic-scoped RBAC permissions, or enter any sub-system workspace.
          </p>

          {/* Quick Search Bar */}
          <form onSubmit={handleSearchSubmit} className="mt-4 flex max-w-lg items-center gap-2">
            <Input
              placeholder="Quick search sub-systems by name, slug, or ID..."
              value={quickSearch}
              onChange={(e) => setQuickSearch(e.target.value)}
              className="bg-white/90 shadow-sm dark:bg-slate-950/90"
            />
            <Button type="submit" size="sm" className="shrink-0 gap-1.5">
              <Search className="h-4 w-4" />
              <span>Search</span>
            </Button>
          </form>
        </div>
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        {statCards.map((stat, i) => {
          if (stat.superOnly && admin.role !== 'super_admin') return null;
          const Icon = stat.icon;
          return (
            <Card
              key={i}
              className="group cursor-pointer transition hover:border-slate-300 dark:hover:border-slate-700"
            >
              <div
                onClick={() => onNavigatePlatform(stat.tab)}
                className="flex items-start justify-between"
              >
                <div>
                  <div className="mb-1 text-xs font-medium text-slate-500 dark:text-slate-400">
                    {stat.title}
                  </div>
                  <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">
                    {stat.value}
                  </div>
                  <div className="mt-1 text-xs text-slate-500 dark:text-slate-500">{stat.desc}</div>
                </div>
                <div className="rounded-xl bg-slate-100 p-2.5 text-slate-600 transition group-hover:bg-emerald-50 group-hover:text-emerald-600 dark:bg-slate-800 dark:text-slate-300 dark:group-hover:bg-emerald-950/40 dark:group-hover:text-emerald-400">
                  <Icon className="h-5 w-5" />
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Active Subsystems Showcase */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
              Independent Subsystems
            </h2>
            <p className="text-xs text-slate-500 dark:text-slate-400">
              Click &quot;Enter Workspace&quot; to open the dedicated Subsystem Console for models,
              configs, and records.
            </p>
          </div>
          <button
            onClick={() => onNavigatePlatform('systems')}
            className="flex items-center gap-1 text-xs font-medium text-emerald-600 hover:underline dark:text-emerald-400"
          >
            <span>View All Subsystems</span>
            <ArrowUpRight className="h-3.5 w-3.5" />
          </button>
        </div>

        {systems.length > 0 ? (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {systems.map((sys) => (
              <Card
                key={sys.id}
                className="flex flex-col justify-between transition hover:border-slate-300 dark:hover:border-slate-700"
              >
                <div>
                  <div className="mb-2 flex items-start justify-between">
                    <div>
                      <div className="text-base font-semibold text-slate-900 dark:text-slate-100">
                        {sys.name}
                      </div>
                      <div className="font-mono text-xs font-medium text-emerald-600 dark:text-emerald-400">
                        /{sys.slug}
                      </div>
                    </div>
                    <Badge variant={sys.status === 1 ? 'success' : 'default'}>
                      {sys.status === 1 ? t('systems.active') : t('systems.disabled')}
                    </Badge>
                  </div>

                  <p className="mb-4 line-clamp-2 text-xs text-slate-600 dark:text-slate-400">
                    {sys.description ||
                      'Independent sub-system with dynamic schema models and isolated topic configs.'}
                  </p>

                  <div className="mb-4 flex items-center gap-3 text-[11px] text-slate-500">
                    <span>
                      <strong>{sys.models_count ?? 0}</strong> models
                    </span>
                    <span>•</span>
                    <span>
                      <strong>{sys.configs_count ?? 0}</strong> configs
                    </span>
                    <span>•</span>
                    <span>
                      <strong>{sys.records_count ?? 0}</strong> records
                    </span>
                  </div>
                </div>

                <div className="flex items-center justify-between border-t border-slate-100 pt-3 dark:border-slate-800/80">
                  <span className="font-mono text-[10px] text-slate-400">/api/v1/s/{sys.slug}</span>
                  <Button
                    size="sm"
                    onClick={() => onNavigateSubsystem(sys.slug, 'overview')}
                    className="gap-1 text-xs"
                  >
                    <Compass className="h-3.5 w-3.5" />
                    <span>Workspace</span>
                    <ArrowRight className="h-3 w-3" />
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        ) : (
          <Card className="border-dashed px-4 py-12 text-center">
            <Layers className="mx-auto mb-3 h-10 w-10 text-slate-400 dark:text-slate-600" />
            <h3 className="mb-1 text-base font-semibold text-slate-900 dark:text-slate-100">
              {t('systems.empty_title')}
            </h3>
            <p className="mx-auto mb-4 max-w-sm text-xs text-slate-500 dark:text-slate-400">
              {t('systems.empty_desc')}
            </p>
            <Button onClick={() => onNavigatePlatform('systems')} className="mx-auto gap-2">
              <PlusCircle className="h-4 w-4" />
              <span>{t('systems.create')}</span>
            </Button>
          </Card>
        )}
      </div>
    </div>
  );
}
