import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Database,
  Sliders,
  TableProperties,
  FileClock,
  ExternalLink,
  Code2,
  Settings,
  Copy,
  Check,
  PlusCircle,
  ArrowRight,
} from 'lucide-react';
import { Card, Badge, Button } from '../components/UiWidgets';
import { api } from '../services/api';
import { SystemItem, SystemStats } from '../types';

interface SubsystemOverviewPageProps {
  currentSystem: SystemItem;
  onNavigate: (tab: string, params?: Record<string, any>) => void;
}

export function SubsystemOverviewPage({ currentSystem, onNavigate }: SubsystemOverviewPageProps) {
  const { t } = useTranslation();
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [copiedCurl, setCopiedCurl] = useState(false);
  const [copiedFetch, setCopiedFetch] = useState(false);

  useEffect(() => {
    let mounted = true;
    api
      .getSystemStats(currentSystem.slug)
      .then((res) => {
        if (mounted) setStats(res);
      })
      .catch(() => {
        if (mounted) {
          setStats({
            id: currentSystem.id,
            slug: currentSystem.slug,
            name: currentSystem.name,
            description: currentSystem.description,
            status: currentSystem.status,
            created_at: currentSystem.created_at,
            models_count: currentSystem.models_count || 0,
            configs_count: currentSystem.configs_count || 0,
            records_count: currentSystem.records_count || 0,
            audit_logs_count: 0,
          });
        }
      });
    return () => {
      mounted = false;
    };
  }, [currentSystem]);

  const curlExample = `curl -X GET "http://localhost:8080/api/v1/s/${currentSystem.slug}/configs" \\
  -H "Accept: application/json"`;

  const fetchExample = `// Fetch topic configurations
const res = await fetch('/api/v1/s/${currentSystem.slug}/configs');
const { data } = await res.json();
console.log('Configs:', data);`;

  const copyToClipboard = (text: string, type: 'curl' | 'fetch') => {
    navigator.clipboard.writeText(text);
    if (type === 'curl') {
      setCopiedCurl(true);
      setTimeout(() => setCopiedCurl(false), 2000);
    } else {
      setCopiedFetch(true);
      setTimeout(() => setCopiedFetch(false), 2000);
    }
  };

  const statCards = [
    {
      title: t('subsystem.stats_models'),
      value: stats ? stats.models_count : (currentSystem.models_count ?? 0),
      desc: 'Dynamic Zero-DDL Models',
      icon: Database,
      tab: 'models',
      actionText: 'Manage Models',
    },
    {
      title: t('subsystem.stats_configs'),
      value: stats ? stats.configs_count : (currentSystem.configs_count ?? 0),
      desc: 'Visual Single-Row Properties',
      icon: Sliders,
      tab: 'configs',
      actionText: 'Configure Settings',
    },
    {
      title: t('subsystem.stats_records'),
      value: stats ? stats.records_count : (currentSystem.records_count ?? 0),
      desc: 'Structured Model Records',
      icon: TableProperties,
      tab: 'data',
      actionText: 'Open Explorer',
    },
    {
      title: t('subsystem.stats_audits'),
      value: stats ? stats.audit_logs_count : 0,
      desc: 'Mutation Write Trail',
      icon: FileClock,
      tab: 'audit_logs',
      actionText: 'View Audit Trail',
    },
  ];

  return (
    <div className="space-y-8">
      {/* Subsystem Banner */}
      <div className="relative overflow-hidden rounded-2xl border border-slate-200 bg-gradient-to-r from-emerald-50 via-teal-50/40 to-slate-50 p-6 shadow-sm dark:border-slate-800 dark:from-slate-900 dark:via-emerald-950/30 dark:to-slate-900">
        <div className="relative z-10 flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <div className="mb-2 flex items-center gap-2">
              <Badge variant="purple">Subsystem Realm</Badge>
              <Badge variant={currentSystem.status === 1 ? 'success' : 'default'}>
                {currentSystem.status === 1 ? t('systems.active') : t('systems.disabled')}
              </Badge>
              <span className="font-mono text-xs text-slate-500 dark:text-slate-400">
                ID: {currentSystem.id}
              </span>
            </div>
            <h1 className="text-2xl font-bold text-slate-900 dark:text-slate-100">
              {currentSystem.name}
            </h1>
            <p className="mt-1 max-w-2xl text-xs text-slate-600 dark:text-slate-400">
              {currentSystem.description || 'Independent sub-system workspace on Foundry Platform.'}
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2.5">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => onNavigate('settings')}
              className="gap-1.5"
            >
              <Settings className="h-4 w-4" />
              <span>{t('nav.sub_settings')}</span>
            </Button>
            <Button size="sm" onClick={() => onNavigate('models')} className="gap-1.5">
              <PlusCircle className="h-4 w-4" />
              <span>{t('models.create_model')}</span>
            </Button>
          </div>
        </div>
      </div>

      {/* KPI Stats Grid */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
        {statCards.map((st, i) => {
          const Icon = st.icon;
          return (
            <Card
              key={i}
              className="group cursor-pointer transition hover:border-slate-300 dark:hover:border-slate-700"
            >
              <div onClick={() => onNavigate(st.tab)} className="space-y-3">
                <div className="flex items-start justify-between">
                  <div>
                    <div className="text-xs font-medium text-slate-500 dark:text-slate-400">
                      {st.title}
                    </div>
                    <div className="mt-1 text-2xl font-bold text-slate-900 dark:text-slate-100">
                      {st.value}
                    </div>
                  </div>
                  <div className="rounded-xl bg-slate-100 p-2.5 text-slate-600 transition group-hover:bg-emerald-50 group-hover:text-emerald-600 dark:bg-slate-800 dark:text-slate-300 dark:group-hover:bg-emerald-950/40 dark:group-hover:text-emerald-400">
                    <Icon className="h-5 w-5" />
                  </div>
                </div>
                <div className="flex items-center justify-between border-t border-slate-100 pt-2 text-[11px] text-slate-500 dark:border-slate-800/80">
                  <span>{st.desc}</span>
                  <span className="flex items-center gap-0.5 font-medium text-emerald-600 group-hover:underline dark:text-emerald-400">
                    {st.actionText}
                    <ArrowRight className="h-3 w-3" />
                  </span>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Subsystem API Integration Guide */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Left: Quick Endpoints & Docs */}
        <Card className="space-y-4">
          <div className="flex items-center justify-between border-b border-slate-100 pb-3 dark:border-slate-800">
            <div className="flex items-center gap-2">
              <Code2 className="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
              <h3 className="text-sm font-bold text-slate-900 dark:text-slate-100">
                REST API Integration Endpoints
              </h3>
            </div>
            <button
              onClick={() => onNavigate('apis')}
              className="text-xs font-medium text-emerald-600 hover:underline dark:text-emerald-400"
            >
              View Full Catalog
            </button>
          </div>

          <div className="space-y-2.5 text-xs">
            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3 dark:border-slate-800 dark:bg-slate-950">
              <div className="mb-1 font-semibold text-slate-700 dark:text-slate-300">
                Base Subsystem Route Prefix:
              </div>
              <code className="block font-mono text-emerald-600 dark:text-emerald-400">
                /api/v1/s/{currentSystem.slug}
              </code>
            </div>

            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3 dark:border-slate-800 dark:bg-slate-950">
              <div className="mb-1 font-semibold text-slate-700 dark:text-slate-300">
                Topic Configs Aggregation:
              </div>
              <code className="block font-mono text-slate-600 dark:text-slate-400">
                GET /api/v1/s/{currentSystem.slug}/configs
              </code>
            </div>

            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3 dark:border-slate-800 dark:bg-slate-950">
              <div className="mb-1 font-semibold text-slate-700 dark:text-slate-300">
                Auto-CRUD Dynamic Models:
              </div>
              <code className="block font-mono text-slate-600 dark:text-slate-400">
                GET /api/v1/s/{currentSystem.slug}/&#123;model_slug&#125;
              </code>
            </div>
          </div>

          <div className="pt-2">
            <a
              href="/api/v1/health"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 text-xs font-medium text-emerald-600 hover:underline dark:text-emerald-400"
            >
              <ExternalLink className="h-3.5 w-3.5" />
              <span>Interactive OpenAPI Specification & Swagger Docs</span>
            </a>
          </div>
        </Card>

        {/* Right: cURL & Fetch Examples */}
        <Card className="space-y-4">
          <div className="flex items-center justify-between border-b border-slate-100 pb-3 dark:border-slate-800">
            <h3 className="text-sm font-bold text-slate-900 dark:text-slate-100">
              Developer Quick Snippets
            </h3>
            <span className="font-mono text-[11px] text-slate-400">HTTP REST</span>
          </div>

          <div className="space-y-3 text-xs">
            <div>
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-medium text-slate-700 dark:text-slate-300">cURL Example</span>
                <button
                  onClick={() => copyToClipboard(curlExample, 'curl')}
                  className="flex items-center gap-1 text-[11px] text-slate-500 hover:text-emerald-600 dark:hover:text-emerald-400"
                >
                  {copiedCurl ? (
                    <>
                      <Check className="h-3 w-3 text-emerald-600" />
                      <span>{t('common.copied')}</span>
                    </>
                  ) : (
                    <>
                      <Copy className="h-3 w-3" />
                      <span>{t('common.copy')}</span>
                    </>
                  )}
                </button>
              </div>
              <pre className="overflow-x-auto rounded-lg border border-slate-200 bg-slate-900 p-3 font-mono text-[11px] text-slate-200 dark:border-slate-800">
                {curlExample}
              </pre>
            </div>

            <div>
              <div className="mb-1.5 flex items-center justify-between">
                <span className="font-medium text-slate-700 dark:text-slate-300">
                  TypeScript Fetch
                </span>
                <button
                  onClick={() => copyToClipboard(fetchExample, 'fetch')}
                  className="flex items-center gap-1 text-[11px] text-slate-500 hover:text-emerald-600 dark:hover:text-emerald-400"
                >
                  {copiedFetch ? (
                    <>
                      <Check className="h-3 w-3 text-emerald-600" />
                      <span>{t('common.copied')}</span>
                    </>
                  ) : (
                    <>
                      <Copy className="h-3 w-3" />
                      <span>{t('common.copy')}</span>
                    </>
                  )}
                </button>
              </div>
              <pre className="overflow-x-auto rounded-lg border border-slate-200 bg-slate-900 p-3 font-mono text-[11px] text-slate-200 dark:border-slate-800">
                {fetchExample}
              </pre>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
