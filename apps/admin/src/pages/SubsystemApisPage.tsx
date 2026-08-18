import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Code2, ExternalLink, Copy, Check } from 'lucide-react';
import { Card, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import { ModelItem, SystemItem } from '../types';

interface SubsystemApisPageProps {
  currentSystem: SystemItem;
}

export function SubsystemApisPage({ currentSystem }: SubsystemApisPageProps) {
  const { t } = useTranslation();
  const [models, setModels] = useState<ModelItem[]>([]);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  useEffect(() => {
    api
      .listModels(currentSystem.slug)
      .then(setModels)
      .catch(() => setModels([]));
  }, [currentSystem.slug]);

  const copyUrl = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  const getMethodBadge = (method: string) => {
    switch (method) {
      case 'GET':
        return <Badge variant="info">GET</Badge>;
      case 'POST':
        return <Badge variant="success">POST</Badge>;
      case 'PUT':
        return <Badge variant="purple">PUT</Badge>;
      case 'PATCH':
        return <Badge variant="warning">PATCH</Badge>;
      case 'DELETE':
        return <Badge variant="danger">DELETE</Badge>;
      default:
        return <Badge variant="default">{method}</Badge>;
    }
  };

  // Pre-configured system endpoints
  const configEndpoints = [
    {
      method: 'GET',
      path: `/api/v1/s/${currentSystem.slug}/configs`,
      desc: 'Retrieve aggregated key-value JSON configurations for this sub-system',
      auth: 'Public / Scoped',
    },
    {
      method: 'PUT',
      path: `/api/v1/s/${currentSystem.slug}/configs`,
      desc: 'Batch update sub-system configuration values (Zero-DDL single-row properties)',
      auth: 'Admin RBAC',
    },
    {
      method: 'GET',
      path: `/api/v1/admin/s/${currentSystem.slug}/configs/schema`,
      desc: 'Retrieve metadata definitions and UI widget configurations for all properties',
      auth: 'Admin RBAC',
    },
  ];

  // Custom demo endpoint if carnival_demo
  const customEndpoints =
    currentSystem.slug === 'carnival_demo'
      ? [
          {
            method: 'POST',
            path: `/api/v1/s/carnival_demo/participate`,
            desc: 'Code-first custom controller endpoint for Carnival activity participation and prize drawing',
            auth: 'Public',
          },
        ]
      : [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('subsystem.apis_title')}
            </h1>
            <Badge variant="success">/{currentSystem.slug}</Badge>
          </div>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
            {t('subsystem.apis_desc')}
          </p>
        </div>

        <a
          href="/api/v1/health"
          target="_blank"
          rel="noreferrer"
          className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-3 py-2 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-200 dark:hover:bg-slate-800"
        >
          <ExternalLink className="h-3.5 w-3.5" />
          <span>OpenAPI Docs</span>
        </a>
      </div>

      {/* 1. Subsystem Configs Endpoints */}
      <div className="space-y-3">
        <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
          <Code2 className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
          <span>Topic Configuration REST Endpoints</span>
        </div>

        <div className="space-y-2">
          {configEndpoints.map((ep, idx) => (
            <Card
              key={idx}
              className="p-4 transition hover:border-slate-300 dark:hover:border-slate-700"
            >
              <div className="flex flex-col justify-between gap-2 sm:flex-row sm:items-center">
                <div className="flex items-center gap-3">
                  {getMethodBadge(ep.method)}
                  <span className="font-mono text-xs font-semibold text-slate-900 dark:text-slate-100">
                    {ep.path}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="purple">{ep.auth}</Badge>
                  <button
                    onClick={() => copyUrl(ep.path, `cfg_${idx}`)}
                    className="rounded p-1 text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400"
                    title={t('common.copy')}
                  >
                    {copiedKey === `cfg_${idx}` ? (
                      <Check className="h-4 w-4 text-emerald-600" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </button>
                </div>
              </div>
              <p className="mt-2 text-xs text-slate-500 dark:text-slate-400">{ep.desc}</p>
            </Card>
          ))}
        </div>
      </div>

      {/* 2. Custom Code-First Endpoints (if any) */}
      {customEndpoints.length > 0 && (
        <div className="space-y-3">
          <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
            <Code2 className="h-4 w-4 text-purple-600 dark:text-purple-400" />
            <span>
              Code-First Custom Rust Controller Endpoints (`systems/src/{currentSystem.slug}/`)
            </span>
          </div>

          <div className="space-y-2">
            {customEndpoints.map((ep, idx) => (
              <Card key={idx} className="border-purple-200/70 p-4 dark:border-purple-900/40">
                <div className="flex flex-col justify-between gap-2 sm:flex-row sm:items-center">
                  <div className="flex items-center gap-3">
                    {getMethodBadge(ep.method)}
                    <span className="font-mono text-xs font-semibold text-slate-900 dark:text-slate-100">
                      {ep.path}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="purple">Native Rust Route</Badge>
                    <button
                      onClick={() => copyUrl(ep.path, `custom_${idx}`)}
                      className="rounded p-1 text-slate-400 hover:text-emerald-600 dark:hover:text-emerald-400"
                    >
                      {copiedKey === `custom_${idx}` ? (
                        <Check className="h-4 w-4 text-emerald-600" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </button>
                  </div>
                </div>
                <p className="mt-2 text-xs text-slate-500 dark:text-slate-400">{ep.desc}</p>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* 3. Auto-CRUD Dynamic Models Endpoints */}
      <div className="space-y-3">
        <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
          <Code2 className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
          <span>Dynamic Auto-CRUD Data Model Endpoints ({models.length} models defined)</span>
        </div>

        {models.map((m) => {
          const listPath = `/api/v1/s/${currentSystem.slug}/${m.slug}`;
          const itemPath = `/api/v1/s/${currentSystem.slug}/${m.slug}/:id`;
          return (
            <Card key={m.id} className="space-y-3 p-4">
              <div className="flex items-center justify-between border-b border-slate-100 pb-2 dark:border-slate-800">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-bold text-slate-900 dark:text-slate-100">
                    {m.name}
                  </span>
                  <span className="font-mono text-xs text-emerald-600 dark:text-emerald-400">
                    /{m.slug}
                  </span>
                </div>
                <Badge variant="info">Zero-DDL Auto-CRUD</Badge>
              </div>

              <div className="grid grid-cols-1 gap-2 text-xs md:grid-cols-2">
                <div className="flex items-center justify-between rounded-lg border border-slate-100 bg-slate-50 p-2.5 dark:border-slate-800 dark:bg-slate-950">
                  <div className="flex items-center gap-2 font-mono">
                    <Badge variant="info">GET</Badge>
                    <span className="truncate text-slate-700 dark:text-slate-300">{listPath}</span>
                  </div>
                  <span className="text-[11px] text-slate-400">List & filter</span>
                </div>

                <div className="flex items-center justify-between rounded-lg border border-slate-100 bg-slate-50 p-2.5 dark:border-slate-800 dark:bg-slate-950">
                  <div className="flex items-center gap-2 font-mono">
                    <Badge variant="success">POST</Badge>
                    <span className="truncate text-slate-700 dark:text-slate-300">{listPath}</span>
                  </div>
                  <span className="text-[11px] text-slate-400">Create record</span>
                </div>

                <div className="flex items-center justify-between rounded-lg border border-slate-100 bg-slate-50 p-2.5 dark:border-slate-800 dark:bg-slate-950">
                  <div className="flex items-center gap-2 font-mono">
                    <Badge variant="info">GET</Badge>
                    <span className="truncate text-slate-700 dark:text-slate-300">{itemPath}</span>
                  </div>
                  <span className="text-[11px] text-slate-400">Get by ID</span>
                </div>

                <div className="flex items-center justify-between rounded-lg border border-slate-100 bg-slate-50 p-2.5 dark:border-slate-800 dark:bg-slate-950">
                  <div className="flex items-center gap-2 font-mono">
                    <Badge variant="danger">DELETE</Badge>
                    <span className="truncate text-slate-700 dark:text-slate-300">{itemPath}</span>
                  </div>
                  <span className="text-[11px] text-slate-400">Soft delete</span>
                </div>
              </div>
            </Card>
          );
        })}

        {models.length === 0 && (
          <Card className="py-8 text-center text-xs text-slate-500 dark:text-slate-400">
            No dynamic models created yet in this sub-system.
          </Card>
        )}
      </div>
    </div>
  );
}
