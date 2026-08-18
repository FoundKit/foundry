import { useTranslation } from 'react-i18next';
import { Layers, Database, Sliders, FileClock, ArrowUpRight, PlusCircle } from 'lucide-react';
import { Card, Badge, Button } from '../components/UiWidgets';
import { AdminProfile, SystemItem } from '../types';

interface DashboardPageProps {
  admin: AdminProfile;
  systems: SystemItem[];
  currentSystem: SystemItem | null;
  onNavigate: (tab: string) => void;
}

export function DashboardPage({ admin, systems, currentSystem, onNavigate }: DashboardPageProps) {
  const { t } = useTranslation();

  const stats = [
    {
      title: t('nav.systems'),
      value: systems.length,
      desc: 'Active Sub-Systems',
      icon: Layers,
      tab: 'systems',
    },
    {
      title: t('nav.configs'),
      value: currentSystem ? 'Configured' : systems.length === 0 ? 'N/A' : 'Global',
      desc: 'Visual Zero-DDL Props',
      icon: Sliders,
      tab: 'configs',
    },
    {
      title: t('nav.models'),
      value: systems.length > 0 ? 'Auto-CRUD' : 'N/A',
      desc: 'Dynamic Data Collections',
      icon: Database,
      tab: 'models',
    },
    {
      title: t('nav.audit_logs'),
      value: 'Non-GET',
      desc: 'Real-time Write Trail',
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
            <Badge variant="success">Foundry Engine Ready</Badge>
            <span className="text-xs text-slate-500 dark:text-slate-400">
              REST-First & OpenAPI-Native
            </span>
          </div>
          <h1 className="text-2xl font-bold text-slate-900 dark:text-slate-100">
            Welcome back, {admin.username}
          </h1>
          <p className="mt-1 max-w-2xl text-sm text-slate-600 dark:text-slate-400">
            Currently managing{' '}
            <strong className="font-semibold text-emerald-600 dark:text-emerald-400">
              {currentSystem
                ? currentSystem.name
                : systems.length === 0
                  ? 'Platform Initialized'
                  : 'All Platforms'}
            </strong>
            . Build dynamic data models, configure topic properties, or extend with code modules.
          </p>
        </div>
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat, i) => {
          const Icon = stat.icon;
          return (
            <Card
              key={i}
              className="group cursor-pointer transition hover:border-slate-300 dark:hover:border-slate-700"
            >
              <div
                onClick={() => onNavigate(stat.tab)}
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

      {/* Active Systems Overview */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
            Registered Sub-Systems
          </h2>
          <button
            onClick={() => onNavigate('systems')}
            className="flex items-center gap-1 text-xs font-medium text-emerald-600 hover:underline dark:text-emerald-400"
          >
            <span>View All</span>
            <ArrowUpRight className="h-3.5 w-3.5" />
          </button>
        </div>

        {systems.length > 0 ? (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {systems.map((sys) => (
              <Card
                key={sys.id}
                className="transition hover:border-slate-300 dark:hover:border-slate-700"
              >
                <div className="mb-3 flex items-start justify-between">
                  <div className="text-base font-semibold text-slate-900 dark:text-slate-100">
                    {sys.name}
                  </div>
                  <Badge variant={sys.status === 1 ? 'success' : 'default'}>
                    {sys.status === 1 ? 'Active' : 'Disabled'}
                  </Badge>
                </div>
                <p className="mb-4 line-clamp-2 text-xs text-slate-600 dark:text-slate-400">
                  {sys.description || 'No description provided.'}
                </p>
                <div className="flex items-center justify-between border-t border-slate-100 pt-3 text-xs text-slate-500 dark:border-slate-800/80">
                  <span className="font-mono font-medium text-emerald-600 dark:text-emerald-400">
                    /{sys.slug}
                  </span>
                  <span>
                    REST API:{' '}
                    <code className="font-mono text-slate-600 dark:text-slate-400">
                      /api/v1/s/{sys.slug}
                    </code>
                  </span>
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
            <Button onClick={() => onNavigate('systems')} className="mx-auto gap-2">
              <PlusCircle className="h-4 w-4" />
              <span>{t('systems.create')}</span>
            </Button>
          </Card>
        )}
      </div>
    </div>
  );
}
