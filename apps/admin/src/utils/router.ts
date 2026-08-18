import { useState, useEffect, useCallback } from 'react';
import { RouteState } from '../types';

export function parseRoute(pathname: string, search: string): RouteState {
  const searchParams = new URLSearchParams(search);
  const params: Record<string, string> = {};
  searchParams.forEach((value, key) => {
    if (value !== '') {
      params[key] = value;
    }
  });

  // Normalize path
  const cleanPath = pathname.replace(/\/+$/, '') || '/';

  // Subsystem routes: /admin/s/:slug/*
  const subsystemMatch = cleanPath.match(/^\/admin\/s\/([^/]+)(?:\/(.*))?$/);
  if (subsystemMatch) {
    const slug = subsystemMatch[1];
    const subPath = subsystemMatch[2] || 'overview';

    let subsystemTab: RouteState['subsystemTab'];
    if (subPath === 'configs') subsystemTab = 'configs';
    else if (subPath === 'models') subsystemTab = 'models';
    else if (subPath === 'data' || subPath === 'data_explorer') subsystemTab = 'data';
    else if (subPath === 'apis') subsystemTab = 'apis';
    else if (subPath === 'audit-logs' || subPath === 'audit_logs') subsystemTab = 'audit_logs';
    else if (subPath === 'settings') subsystemTab = 'settings';
    else subsystemTab = 'overview';

    return {
      mode: 'subsystem',
      platformTab: 'systems',
      subsystemSlug: slug,
      subsystemTab,
      params,
    };
  }

  // Platform routes: /admin/:tab or /:tab
  let platformTab: RouteState['platformTab'];
  if (cleanPath === '/admin/systems' || cleanPath === '/systems') {
    platformTab = 'systems';
  } else if (cleanPath === '/admin/admins' || cleanPath === '/admins') {
    platformTab = 'admins';
  } else if (
    cleanPath === '/admin/audit-logs' ||
    cleanPath === '/admin/audit_logs' ||
    cleanPath === '/audit-logs'
  ) {
    platformTab = 'audit_logs';
  } else {
    platformTab = 'dashboard';
  }

  return {
    mode: 'platform',
    platformTab,
    subsystemSlug: null,
    subsystemTab: 'overview',
    params,
  };
}

export function buildRouteUrl(state: {
  mode?: 'platform' | 'subsystem';
  platformTab?: RouteState['platformTab'];
  subsystemSlug?: string | null;
  subsystemTab?: RouteState['subsystemTab'];
  params?: Record<string, any>;
}): string {
  const path =
    state.mode === 'subsystem' && state.subsystemSlug
      ? `/admin/s/${state.subsystemSlug}/${state.subsystemTab || 'overview'}`
      : `/admin/${state.platformTab || 'dashboard'}`;

  const queryParams = new URLSearchParams();
  if (state.params) {
    Object.entries(state.params).forEach(([k, v]) => {
      if (v !== undefined && v !== null && v !== '') {
        queryParams.set(k, String(v));
      }
    });
  }

  const qs = queryParams.toString();
  return qs ? `${path}?${qs}` : path;
}

export function useAppRouter() {
  const [route, setRoute] = useState<RouteState>(() =>
    parseRoute(window.location.pathname, window.location.search),
  );

  useEffect(() => {
    const handlePopState = () => {
      setRoute(parseRoute(window.location.pathname, window.location.search));
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const navigatePlatform = useCallback(
    (tab: RouteState['platformTab'], params?: Record<string, any>, replace = false) => {
      const newUrl = buildRouteUrl({
        mode: 'platform',
        platformTab: tab,
        subsystemSlug: null,
        params,
      });
      if (replace) {
        window.history.replaceState(null, '', newUrl);
      } else {
        window.history.pushState(null, '', newUrl);
      }
      setRoute(parseRoute(window.location.pathname, window.location.search));
    },
    [],
  );

  const navigateSubsystem = useCallback(
    (
      slug: string,
      tab: RouteState['subsystemTab'] = 'overview',
      params?: Record<string, any>,
      replace = false,
    ) => {
      const newUrl = buildRouteUrl({
        mode: 'subsystem',
        subsystemSlug: slug,
        subsystemTab: tab,
        params,
      });
      if (replace) {
        window.history.replaceState(null, '', newUrl);
      } else {
        window.history.pushState(null, '', newUrl);
      }
      setRoute(parseRoute(window.location.pathname, window.location.search));
    },
    [],
  );

  const updateParams = useCallback((newParams: Record<string, any>, replace = true) => {
    setRoute((prev) => {
      const mergedParams: Record<string, any> = { ...prev.params };
      Object.entries(newParams).forEach(([k, v]) => {
        if (v === undefined || v === null || v === '') {
          delete mergedParams[k];
        } else {
          mergedParams[k] = v;
        }
      });

      const newUrl = buildRouteUrl({
        mode: prev.mode,
        platformTab: prev.platformTab,
        subsystemSlug: prev.subsystemSlug,
        subsystemTab: prev.subsystemTab,
        params: mergedParams,
      });

      if (replace) {
        window.history.replaceState(null, '', newUrl);
      } else {
        window.history.pushState(null, '', newUrl);
      }

      return parseRoute(window.location.pathname, window.location.search);
    });
  }, []);

  return {
    route,
    navigatePlatform,
    navigateSubsystem,
    updateParams,
  };
}
