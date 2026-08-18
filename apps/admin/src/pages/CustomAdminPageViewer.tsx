import React, { useState, useEffect, useRef } from 'react';
import { Sparkles, ExternalLink, RefreshCw, AlertTriangle, ShieldAlert } from 'lucide-react';
import { AdminProfile, CustomAdminPageItem, SystemItem } from '../types';
import { api } from '../services/api';

interface CustomAdminPageViewerProps {
  currentSystem: SystemItem;
  pageKey: string;
  admin: AdminProfile;
}

export function CustomAdminPageViewer({
  currentSystem,
  pageKey,
  admin,
}: CustomAdminPageViewerProps) {
  const [customPages, setCustomPages] = useState<CustomAdminPageItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<{ msg: string; type: string } | null>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    let isMounted = true;
    setLoading(true);
    setError(null);

    api
      .listCustomPages(currentSystem.slug)
      .then((pages) => {
        if (isMounted) {
          setCustomPages(pages);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (isMounted) {
          setError(err.message || 'Failed to load custom admin page registry');
          setLoading(false);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [currentSystem.slug]);

  const targetPage = customPages.find((p) => p.key === pageKey);

  // Send init postMessage to iframe when iframe finishes loading
  const handleIframeLoad = () => {
    if (!iframeRef.current || !targetPage) return;
    const token = localStorage.getItem('foundry_token');
    const isDark = document.documentElement.classList.contains('dark');
    
    iframeRef.current.contentWindow?.postMessage(
      {
        type: 'FOUNDRY_INIT',
        payload: {
          token,
          subsystemSlug: currentSystem.slug,
          admin,
          theme: isDark ? 'dark' : 'light',
        },
      },
      '*',
    );
  };

  // Listen for messages sent from embedded custom iframe page
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (!event.data || typeof event.data !== 'object') return;

      if (event.data.type === 'FOUNDRY_TOAST') {
        const { message, level } = event.data.payload || {};
        setToastMessage({ msg: message || 'Notification from custom page', type: level || 'info' });
        setTimeout(() => setToastMessage(null), 4000);
      }
    };

    window.addEventListener('message', handleMessage);
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  if (loading) {
    return (
      <div className="flex h-96 items-center justify-center text-sm text-slate-500 dark:text-slate-400">
        <RefreshCw className="mr-2 h-5 w-5 animate-spin text-emerald-500" />
        <span>Loading custom subsystem admin page...</span>
      </div>
    );
  }

  if (error || !targetPage) {
    return (
      <div className="rounded-2xl border border-rose-200 bg-rose-50/50 p-8 text-center dark:border-rose-900/40 dark:bg-rose-950/20">
        <AlertTriangle className="mx-auto h-10 w-10 text-rose-500" />
        <h3 className="mt-3 text-base font-bold text-rose-900 dark:text-rose-300">
          {error || `Custom Page "${pageKey}" Not Found`}
        </h3>
        <p className="mt-1 text-xs text-rose-700 dark:text-rose-400">
          No custom admin page registered with key "{pageKey}" for subsystem "{currentSystem.slug}".
        </p>
      </div>
    );
  }

  // Role access enforcement for custom page
  if (targetPage.required_role && admin.role !== 'super_admin' && admin.role !== targetPage.required_role) {
    return (
      <div className="rounded-2xl border border-amber-200 bg-amber-50/50 p-8 text-center dark:border-amber-900/40 dark:bg-amber-950/20">
        <ShieldAlert className="mx-auto h-10 w-10 text-amber-500" />
        <h3 className="mt-3 text-base font-bold text-amber-900 dark:text-amber-300">
          Access Restricted
        </h3>
        <p className="mt-1 text-xs text-amber-700 dark:text-amber-400">
          This custom admin view requires role: <code className="font-bold">{targetPage.required_role}</code>.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Page Header Bar */}
      <div className="flex items-center justify-between rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <div className="flex items-center space-x-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
            <Sparkles className="h-5 w-5" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h1 className="text-base font-bold text-slate-900 dark:text-slate-100">
                {targetPage.title}
              </h1>
              <span className="rounded border border-indigo-200 bg-indigo-50 px-2 py-0.5 font-mono text-[10px] font-semibold text-indigo-700 dark:border-indigo-800/60 dark:bg-indigo-950/80 dark:text-indigo-400">
                {targetPage.type}
              </span>
            </div>
            <p className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
              Integrated Subsystem Admin View · {currentSystem.slug} / {targetPage.key}
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <a
            href={targetPage.entry}
            target="_blank"
            rel="noreferrer"
            className="flex items-center space-x-1.5 rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            <span>Open Standalone</span>
          </a>
        </div>
      </div>

      {/* Toast Notification Banner */}
      {toastMessage && (
        <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-3 text-xs font-medium text-emerald-800 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300">
          🔔 {toastMessage.msg}
        </div>
      )}

      {/* Embedded View */}
      <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <iframe
          ref={iframeRef}
          src={targetPage.entry}
          onLoad={handleIframeLoad}
          className="h-[calc(100vh-220px)] w-full border-none"
          title={targetPage.title}
        />
      </div>
    </div>
  );
}
