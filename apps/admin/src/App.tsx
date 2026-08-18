import { useState, useEffect, useCallback } from 'react';
import './locales/i18n';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { DashboardPage } from './pages/DashboardPage';
import { SystemsPage } from './pages/SystemsPage';
import { SubsystemOverviewPage } from './pages/SubsystemOverviewPage';
import { SubsystemApisPage } from './pages/SubsystemApisPage';
import { SubsystemSettingsPage } from './pages/SubsystemSettingsPage';
import { ConfigsPage } from './pages/ConfigsPage';
import { ModelsPage } from './pages/ModelsPage';
import { DataExplorerPage } from './pages/DataExplorerPage';
import { AdminsPage } from './pages/AdminsPage';
import { AuditLogsPage } from './pages/AuditLogsPage';
import { api } from './services/api';
import { AdminProfile, SystemItem } from './types';
import { useAppRouter } from './utils/router';

export function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('foundry_token'));
  const [admin, setAdmin] = useState<AdminProfile | null>(null);
  const [systems, setSystems] = useState<SystemItem[]>([]);
  const [currentSystem, setCurrentSystem] = useState<SystemItem | null>(null);
  const [loading, setLoading] = useState(true);

  const { route, navigatePlatform, navigateSubsystem, updateParams } = useAppRouter();

  const handleLogout = useCallback(() => {
    localStorage.removeItem('foundry_token');
    setToken(null);
    setAdmin(null);
    setSystems([]);
    setCurrentSystem(null);
  }, []);

  const fetchProfileAndSystems = useCallback(async () => {
    try {
      const [profile, sysPaginated] = await Promise.all([
        api.me(),
        api.listSystems({ page: 1, page_size: 100 }),
      ]);
      setAdmin(profile);
      const sysList = sysPaginated.items;
      setSystems(sysList);

      // Resolve active subsystem from current URL or default
      if (route.subsystemSlug) {
        const found = sysList.find((s) => s.slug === route.subsystemSlug);
        if (found) {
          setCurrentSystem(found);
        } else {
          // Attempt to fetch single system by slug
          try {
            const single = await api.getSystemBySlug(route.subsystemSlug);
            setCurrentSystem(single);
          } catch {
            setCurrentSystem(null);
          }
        }
      } else if (sysList.length > 0) {
        setCurrentSystem(sysList[0]);
      } else {
        setCurrentSystem(null);
      }
    } catch (err) {
      console.error(err);
      handleLogout();
    } finally {
      setLoading(false);
    }
  }, [route.subsystemSlug, handleLogout]);

  useEffect(() => {
    if (token) {
      fetchProfileAndSystems();
    } else {
      setLoading(false);
    }
  }, [token, fetchProfileAndSystems]);

  // Sync currentSystem when route.subsystemSlug changes
  useEffect(() => {
    if (route.subsystemSlug && systems.length > 0) {
      const matched = systems.find((s) => s.slug === route.subsystemSlug);
      if (matched && matched.id !== currentSystem?.id) {
        setCurrentSystem(matched);
      }
    }
  }, [route.subsystemSlug, systems, currentSystem]);

  const handleLoginSuccess = (newToken: string, newAdmin: AdminProfile) => {
    localStorage.setItem('foundry_token', newToken);
    setToken(newToken);
    setAdmin(newAdmin);
    fetchProfileAndSystems();
  };

  const handleSelectSystem = (system: SystemItem) => {
    setCurrentSystem(system);
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-50 text-sm text-slate-500 dark:bg-slate-950 dark:text-slate-400">
        Initializing Foundry Control Plane...
      </div>
    );
  }

  if (!token || !admin) {
    return <LoginPage onLoginSuccess={handleLoginSuccess} />;
  }

  return (
    <Layout
      route={route}
      onNavigatePlatform={navigatePlatform}
      onNavigateSubsystem={navigateSubsystem}
      admin={admin}
      systems={systems}
      currentSystem={currentSystem}
      onSelectSystem={handleSelectSystem}
      onLogout={handleLogout}
    >
      {/* 1. Subsystem Mode */}
      {route.mode === 'subsystem' && currentSystem && (
        <>
          {route.subsystemTab === 'overview' && (
            <SubsystemOverviewPage
              currentSystem={currentSystem}
              onNavigate={(tab, params) =>
                navigateSubsystem(currentSystem.slug, tab as any, params)
              }
            />
          )}
          {route.subsystemTab === 'configs' && (
            <ConfigsPage
              currentSystem={currentSystem}
              onNavigate={(tab) => navigateSubsystem(currentSystem.slug, tab as any)}
            />
          )}
          {route.subsystemTab === 'models' && (
            <ModelsPage
              currentSystem={currentSystem}
              queryParams={route.params}
              onUpdateParams={updateParams}
              onNavigate={(tab) => navigateSubsystem(currentSystem.slug, tab as any)}
            />
          )}
          {route.subsystemTab === 'data' && (
            <DataExplorerPage
              currentSystem={currentSystem}
              queryParams={route.params}
              onUpdateParams={updateParams}
              onNavigate={(tab) => navigateSubsystem(currentSystem.slug, tab as any)}
            />
          )}
          {route.subsystemTab === 'apis' && <SubsystemApisPage currentSystem={currentSystem} />}
          {route.subsystemTab === 'audit_logs' && (
            <AuditLogsPage
              currentSystem={currentSystem}
              queryParams={route.params}
              onUpdateParams={updateParams}
            />
          )}
          {route.subsystemTab === 'settings' && (
            <SubsystemSettingsPage
              currentSystem={currentSystem}
              onRefresh={fetchProfileAndSystems}
            />
          )}
        </>
      )}

      {/* 2. Platform Mode */}
      {route.mode === 'platform' && (
        <>
          {route.platformTab === 'dashboard' && (
            <DashboardPage
              admin={admin}
              systems={systems}
              onNavigatePlatform={navigatePlatform}
              onNavigateSubsystem={(slug, tab) =>
                navigateSubsystem(slug, (tab as any) || 'overview')
              }
            />
          )}
          {route.platformTab === 'systems' && (
            <SystemsPage
              admin={admin}
              queryParams={route.params}
              onUpdateParams={updateParams}
              onRefresh={fetchProfileAndSystems}
              onEnterSubsystem={(s) => {
                setCurrentSystem(s);
                navigateSubsystem(s.slug, 'overview');
              }}
            />
          )}
          {route.platformTab === 'admins' && <AdminsPage systems={systems} />}
          {route.platformTab === 'audit_logs' && (
            <AuditLogsPage queryParams={route.params} onUpdateParams={updateParams} />
          )}
        </>
      )}
    </Layout>
  );
}

export default App;
