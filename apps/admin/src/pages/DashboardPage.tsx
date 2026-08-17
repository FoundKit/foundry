import { useTranslation } from 'react-i18next';
import {
  Layers,
  Database,
  Sliders,
  FileClock,
  ArrowUpRight,
} from 'lucide-react';
import { Card, Badge } from '../components/UiWidgets';
import { AdminProfile, SystemItem } from '../types';

interface DashboardPageProps {
  admin: AdminProfile;
  systems: SystemItem[];
  currentSystem: SystemItem | null;
  onNavigate: (tab: string) => void;
}

export function DashboardPage({
  admin,
  systems,
  currentSystem,
  onNavigate,
}: DashboardPageProps) {
  const { t } = useTranslation();

  const stats = [
    {
      title: t('nav.systems'),
      value: systems.length,
      desc: 'Active Sub-Systems',
      icon: Layers,
      tab: 'systems',
      color: 'emerald',
    },
    {
      title: t('nav.configs'),
      value: currentSystem ? 'Configured' : 'Global',
      desc: 'Visual Zero-DDL Props',
      icon: Sliders,
      tab: 'configs',
      color: 'teal',
    },
    {
      title: t('nav.models'),
      value: 'Auto-CRUD',
      desc: 'Dynamic Data Collections',
      icon: Database,
      tab: 'models',
      color: 'sky',
    },
    {
      title: t('nav.audit_logs'),
      value: 'Non-GET',
      desc: 'Real-time Write Trail',
      icon: FileClock,
      tab: 'audit_logs',
      color: 'indigo',
    },
  ];

  return (
    <div className="space-y-8">
      {/* Header Banner */}
      <div className="bg-gradient-to-r from-slate-900 via-slate-900 to-emerald-950/40 border border-slate-800 rounded-2xl p-6 relative overflow-hidden">
        <div className="relative z-10">
          <div className="flex items-center gap-2 mb-2">
            <Badge variant="success">Foundry Engine Ready</Badge>
            <span className="text-xs text-slate-400">REST-First & OpenAPI-Native</span>
          </div>
          <h1 className="text-2xl font-bold text-slate-100">
            Welcome back, {admin.username}
          </h1>
          <p className="text-sm text-slate-400 mt-1 max-w-2xl">
            Currently managing{' '}
            <strong className="text-emerald-400">
              {currentSystem ? currentSystem.name : 'All Platforms'}
            </strong>
            . Build dynamic data models, configure topic properties, or extend with code modules.
          </p>
        </div>
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {stats.map((stat, i) => {
          const Icon = stat.icon;
          return (
            <Card
              key={i}
              className="cursor-pointer hover:border-slate-700 transition group"
            >
              <div
                onClick={() => onNavigate(stat.tab)}
                className="flex items-start justify-between"
              >
                <div>
                  <div className="text-xs font-medium text-slate-400 mb-1">{stat.title}</div>
                  <div className="text-2xl font-bold text-slate-100">{stat.value}</div>
                  <div className="text-xs text-slate-500 mt-1">{stat.desc}</div>
                </div>
                <div className="p-2.5 rounded-xl bg-slate-800 text-slate-300 group-hover:text-emerald-400 group-hover:bg-emerald-950/40 transition">
                  <Icon className="w-5 h-5" />
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Active Systems Overview */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-100">Registered Sub-Systems</h2>
          <button
            onClick={() => onNavigate('systems')}
            className="text-xs text-emerald-400 hover:underline flex items-center gap-1"
          >
            <span>View All</span>
            <ArrowUpRight className="w-3.5 h-3.5" />
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {systems.map((sys) => (
            <Card key={sys.id} className="hover:border-slate-700 transition">
              <div className="flex items-start justify-between mb-3">
                <div className="font-semibold text-slate-100 text-base">{sys.name}</div>
                <Badge variant={sys.status === 1 ? 'success' : 'default'}>
                  {sys.status === 1 ? 'Active' : 'Disabled'}
                </Badge>
              </div>
              <p className="text-xs text-slate-400 mb-4 line-clamp-2">
                {sys.description || 'No description provided.'}
              </p>
              <div className="flex items-center justify-between text-xs text-slate-500 border-t border-slate-800/80 pt-3">
                <span className="font-mono text-emerald-400">/{sys.slug}</span>
                <span>REST API: <code className="text-slate-400 font-mono">/api/v1/s/{sys.slug}</code></span>
              </div>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}
