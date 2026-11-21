import { useLayoutEffect, useState } from "react";
import { Sidebar, type View } from "./components/Sidebar";
import { StorageStatus } from "./components/StorageStatus";
import { Library } from "./features/library/Library";
import { useCatalogPreferences } from "./features/library/useCatalogPreferences";
import { BackupsPanel } from "./features/settings/BackupsPanel";
import { useAutomaticBackup } from "./features/settings/useAutomaticBackup";
import { LocaleContext, locales } from "./locales/locale";
import "./styles/app.css";

export default function App() {
  const [revision, setRevision] = useState(0);
  return <Workspace key={revision} restored={revision > 0} onRestored={() => setRevision((value) => value + 1)} />;
}

function Workspace({ restored, onRestored }: { restored: boolean; onRestored: () => void }) {
  const [view, setView] = useState<View>(restored ? "settings" : "library");
  const [activeSeries, setActiveSeries] = useState<number | null>(null);
  const settings = useCatalogPreferences();
  const automatic = useAutomaticBackup();
  function navigate(next: View) {
    setView(next);
    setActiveSeries(null);
  }
  const theme = settings.preferences?.theme ?? "dark";
  const language = settings.preferences?.language ?? "es";
  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.lang = language;
  }, [theme, language]);
  return (
    <LocaleContext.Provider value={locales[language]}>
    <div className="app" data-cover-size={settings.preferences?.coverSize ?? "medium"} onDragOver={(event) => {
      if (event.dataTransfer.types.includes("Files")) event.preventDefault();
    }} onDrop={(event) => event.preventDefault()}>
      <Sidebar view={view} onNavigate={navigate} />
      <div className="workspace">
        <Library view={view} activeSeries={activeSeries} settings={settings} onNavigate={navigate}
          backupsPanel={<BackupsPanel automatic={automatic} beforeRestore={settings.beforeRestore}
            onRestored={onRestored} restored={restored} />} onActiveSeries={(id) => {
          if (id !== null && view === "settings") setView("library");
          setActiveSeries(id);
        }} />
        <StorageStatus />
      </div>
    </div>
    </LocaleContext.Provider>
  );
}
