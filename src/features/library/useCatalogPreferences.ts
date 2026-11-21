import { useEffect, useRef, useState } from "react";
import { accessPreferences, defaultPreferences, type Preferences } from "../../services/preferences";

export function useCatalogPreferences() {
  const [preferences, setPreferences] = useState<Preferences | null>(null);
  const [error, setError] = useState<"read" | "write" | null>(null);
  const [attempt, setAttempt] = useState(0);
  const queued = useRef<Preferences | null>(null);
  const saving = useRef<Promise<void> | null>(null);

  useEffect(() => {
    let active = true;
    void accessPreferences(null).then((saved) => {
      if (active) setPreferences(saved);
    }).catch(() => {
      if (active) {
        setPreferences(defaultPreferences);
        setError("read");
      }
    });
    return () => { active = false; };
  }, [attempt]);

  function flush() {
    if (saving.current) return saving.current;
    const pending = (async () => {
      while (queued.current) {
        const value = queued.current;
        queued.current = null;
        try {
          await accessPreferences(value);
          setError(null);
        } catch {
          if (!queued.current) {
            queued.current = value;
            setError("write");
            return;
          }
        }
      }
    })();
    saving.current = pending;
    void pending.finally(() => { saving.current = null; });
    return pending;
  }

  function update(value: Preferences) {
    setPreferences(value);
    queued.current = value;
    void flush();
  }

  function retry() {
    if (error === "write") {
      void flush();
    } else {
      setPreferences(null);
      setError(null);
      setAttempt((current) => current + 1);
    }
  }

  async function beforeRestore() {
    queued.current = null;
    await saving.current;
    queued.current = null;
  }

  return { preferences, error, update, retry, beforeRestore };
}
