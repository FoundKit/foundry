import { useState, useEffect } from 'react';
import './locales/i18n';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { DashboardPage } from './pages/DashboardPage';
import { SystemsPage } from './pages/SystemsPage';
import { ConfigsPage } from './pages/ConfigsPage';
import { ModelsPage } from './pages/ModelsPage';
import { DataExplorerPage } from './pages/DataExplorerPage';
import { AdminsPage } from './pages/AdminsPage';
import { AuditLogsPage } from './pages/AuditLogsPage';
import { api } from './services/api';
import { AdminProfile, SystemItem } from './types';

export function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('foundry_token'));
  const [admin, setAdmin] = useState<AdminProfile | null>(null);
  const [currentTab, setCurrentTab] = useState<string>('dashboard');
  const [systems, setSystems] = useState<SystemItem[]>([]);
  const [currentSystem, setCurrentSystem] = useState<SystemItem | null>(null);
  const [loading, setLoading] = useState(true);

  const handleLogout = () => {
    localStorage.removeItem('foundry_token');
    setToken(null);
    setAdmin(null);
    setSystems([]);
    setCurrentSystem(null);
  };

  const fetchProfileAndSystems = async () => {
    try {
      const [profile, sysList] = await Promise.all([
        api.me(),
        api.listSystems(),
      ]);
      setAdmin(profile);
      setSystems(sysList);
      if (sysList.length > 0 && !currentSystem) {
        setCurrentSystem(sysList[0]);
      }
    } catch (err) {
      console.error(err);
      handleLogout();
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (token) {
      fetchProfileAndSystems();
    } else {
      setLoading(false);
    }
  }, [token]);

  const handleLoginSuccess = (newToken: string, newAdmin: AdminProfile) => {
    localStorage.setItem('foundry_token', newToken);
    setToken(newToken);
    setAdmin(newAdmin);
    fetchProfileAndSystems();
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-slate-950 flex items-center justify-center text-slate-400 text-sm">
        Initializing Foundry Control Plane...
      </div>
    );
  }

  if (!token || !admin) {
    return <LoginPage onLoginSuccess={handleLoginSuccess} />;
  }

  return (
    <Layout
      currentTab={currentTab}
      onTabChange={setCurrentTab}
      admin={admin}
      systems={systems}
      currentSystem={currentSystem}
      onSelectSystem={setCurrentSystem}
      onLogout={handleLogout}
    >
      {currentTab === 'dashboard' && (
        <DashboardPage
          admin={admin}
          systems={systems}
          currentSystem={currentSystem}
          onNavigate={setCurrentTab}
        />
      )}
      {currentTab === 'systems' && (
        <SystemsPage
          admin={admin}
          systems={systems}
          onRefresh={fetchProfileAndSystems}
          onSelectSystem={(s) => {
            setCurrentSystem(s);
            setCurrentTab('configs');
          }}
        />
      )}
      {currentTab === 'configs' && <ConfigsPage currentSystem={currentSystem} />}
      {currentTab === 'models' && <ModelsPage currentSystem={currentSystem} />}
      {currentTab === 'data_explorer' && <DataExplorerPage currentSystem={currentSystem} />}
      {currentTab === 'admins' && <AdminsPage systems={systems} />}
      {currentTab === 'audit_logs' && <AuditLogsPage currentSystem={currentSystem} />}
    </Layout>
  );
}

export default App;
