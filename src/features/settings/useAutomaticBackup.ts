import { useEffect, useState } from "react";
import { accessBackup, type BackupStatus } from "../../services/backups";

export function useAutomaticBackup() {
  const [status, setStatus] = useState<BackupStatus | null>(null);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    let active = true;
    function check() {
      void accessBackup({ kind: "check_automatic" }).then((result) => {
        if (active && result?.kind === "status") {
          setStatus(result.status);
          setError(null);
        }
      }).catch((reason: unknown) => {
        if (!active) return;
        setError(reason);
        void accessBackup({ kind: "status" }).then((result) => {
          if (active && result?.kind === "status") setStatus(result.status);
        }).catch(() => {});
      });
    }
    check();
    const interval = window.setInterval(check, 15 * 60 * 1000);
    return () => { active = false; window.clearInterval(interval); };
  }, []);
  return { status, error, setStatus, setError };
}
