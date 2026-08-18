import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Eye } from 'lucide-react';
import { Card, Button, Modal, Badge } from '../components/UiWidgets';
import { api } from '../services/api';
import { AuditLogItem, SystemItem } from '../types';

interface AuditLogsPageProps {
  currentSystem: SystemItem | null;
}

export function AuditLogsPage({ currentSystem }: AuditLogsPageProps) {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<AuditLogItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [page] = useState(1);
  const [total, setTotal] = useState(0);

  // Inspector Modal state
  const [inspectLog, setInspectLog] = useState<AuditLogItem | null>(null);
  const [activeTab, setActiveTab] = useState<'headers' | 'query' | 'body'>('body');

  const loadLogs = async () => {
    setLoading(true);
    try {
      const res = await api.listAuditLogs({
        page,
        system_slug: currentSystem?.slug,
      });
      setLogs(res.items);
      setTotal(res.pagination.total);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadLogs();
  }, [currentSystem, page]);

  const getMethodBadge = (method: string) => {
    switch (method.toUpperCase()) {
      case 'POST':
        return <Badge variant="success">POST</Badge>;
      case 'PUT':
        return <Badge variant="info">PUT</Badge>;
      case 'PATCH':
        return <Badge variant="warning">PATCH</Badge>;
      case 'DELETE':
        return <Badge variant="danger">DELETE</Badge>;
      default:
        return <Badge variant="default">{method}</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
            {t('audit.title')}
          </h1>
          <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
            {t('audit.desc')} ({total} records)
          </p>
        </div>
        <Button variant="secondary" onClick={loadLogs} size="sm">
          Refresh Trail
        </Button>
      </div>

      <Card className="overflow-hidden p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-700 dark:text-slate-300">
            <thead className="border-b border-slate-200 bg-slate-50 text-xs font-medium uppercase text-slate-500 dark:border-slate-800 dark:bg-slate-950/60 dark:text-slate-400">
              <tr>
                <th className="px-5 py-3.5">{t('audit.time')}</th>
                <th className="px-5 py-3.5">{t('audit.operator')}</th>
                <th className="px-5 py-3.5">{t('audit.action')}</th>
                <th className="px-5 py-3.5">{t('audit.method')}</th>
                <th className="px-5 py-3.5">{t('audit.path')}</th>
                <th className="px-5 py-3.5">{t('audit.system')}</th>
                <th className="px-5 py-3.5">{t('audit.status')}</th>
                <th className="px-5 py-3.5">{t('audit.duration')}</th>
                <th className="px-5 py-3.5 text-right">Inspect</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 text-xs dark:divide-slate-800/60">
              {logs.map((log) => (
                <tr
                  key={log.id}
                  className="transition hover:bg-slate-50 dark:hover:bg-slate-800/30"
                >
                  <td className="whitespace-nowrap px-5 py-3.5 font-mono text-[11px] text-slate-500 dark:text-slate-400">
                    {new Date(log.created_at).toLocaleString()}
                  </td>
                  <td className="px-5 py-3.5 font-semibold text-slate-800 dark:text-slate-200">
                    {log.admin_username || 'Anonymous'}
                  </td>
                  <td className="px-5 py-3.5">
                    <span className="font-medium text-emerald-700 dark:text-emerald-300">
                      {log.action_name || 'State Mutation'}
                    </span>
                  </td>
                  <td className="px-5 py-3.5">{getMethodBadge(log.method)}</td>
                  <td className="max-w-xs truncate px-5 py-3.5 font-mono text-xs text-slate-700 dark:text-slate-300">
                    {log.path}
                  </td>
                  <td className="px-5 py-3.5 font-mono text-xs text-slate-500 dark:text-slate-400">
                    {log.system_slug ? `/${log.system_slug}` : 'Global'}
                  </td>
                  <td className="px-5 py-3.5">
                    <span
                      className={`font-mono font-semibold ${
                        (log.status_code || 200) < 400
                          ? 'text-emerald-600 dark:text-emerald-400'
                          : 'text-rose-600 dark:text-rose-400'
                      }`}
                    >
                      {log.status_code || 200}
                    </span>
                  </td>
                  <td className="px-5 py-3.5 font-mono text-slate-500 dark:text-slate-400">
                    {log.duration_ms !== undefined ? `${log.duration_ms}ms` : '-'}
                  </td>
                  <td className="px-5 py-3.5 text-right">
                    <button
                      onClick={() => setInspectLog(log)}
                      className="rounded-lg p-1.5 text-slate-400 transition hover:bg-slate-100 hover:text-emerald-600 dark:hover:bg-slate-800 dark:hover:text-emerald-400"
                      title="Inspect Raw Request"
                    >
                      <Eye className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
              {logs.length === 0 && (
                <tr>
                  <td colSpan={9} className="py-12 text-center text-slate-500 dark:text-slate-400">
                    {loading ? t('common.loading') : 'No write audit logs recorded yet.'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Raw Request Inspector Modal */}
      <Modal isOpen={!!inspectLog} onClose={() => setInspectLog(null)} title={t('audit.inspector')}>
        {inspectLog && (
          <div className="space-y-4">
            <div className="flex items-center justify-between border-b border-slate-200 pb-3 text-xs dark:border-slate-800">
              <div className="flex items-center gap-2">
                {getMethodBadge(inspectLog.method)}
                <span className="font-mono text-slate-800 dark:text-slate-200">
                  {inspectLog.path}
                </span>
              </div>
              <span className="font-mono text-slate-500 dark:text-slate-400">
                IP: {inspectLog.ip_address || '127.0.0.1'}
              </span>
            </div>

            {/* Discrete Tabs */}
            <div className="flex gap-2 border-b border-slate-200 dark:border-slate-800">
              <button
                onClick={() => setActiveTab('body')}
                className={`border-b-2 px-3 py-2 text-xs font-medium transition ${
                  activeTab === 'body'
                    ? 'border-emerald-600 text-emerald-700 dark:border-emerald-500 dark:text-emerald-400'
                    : 'border-transparent text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
                }`}
              >
                {t('audit.tab_body')}
              </button>
              <button
                onClick={() => setActiveTab('query')}
                className={`border-b-2 px-3 py-2 text-xs font-medium transition ${
                  activeTab === 'query'
                    ? 'border-emerald-600 text-emerald-700 dark:border-emerald-500 dark:text-emerald-400'
                    : 'border-transparent text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
                }`}
              >
                {t('audit.tab_query')}
              </button>
              <button
                onClick={() => setActiveTab('headers')}
                className={`border-b-2 px-3 py-2 text-xs font-medium transition ${
                  activeTab === 'headers'
                    ? 'border-emerald-600 text-emerald-700 dark:border-emerald-500 dark:text-emerald-400'
                    : 'border-transparent text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
                }`}
              >
                {t('audit.tab_headers')}
              </button>
            </div>

            {/* Content Display */}
            <div className="max-h-72 overflow-auto rounded-xl border border-slate-200 bg-slate-100 p-4 font-mono text-xs text-slate-800 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-300">
              {activeTab === 'body' && (
                <pre className="whitespace-pre-wrap">
                  {inspectLog.body_params || '(Empty Request Body)'}
                </pre>
              )}
              {activeTab === 'query' && (
                <pre className="whitespace-pre-wrap">
                  {inspectLog.query_params || '(Empty Query String)'}
                </pre>
              )}
              {activeTab === 'headers' && (
                <pre className="whitespace-pre-wrap">
                  {JSON.stringify(inspectLog.headers, null, 2)}
                </pre>
              )}
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
