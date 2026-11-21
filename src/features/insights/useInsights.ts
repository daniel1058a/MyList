import { useEffect, useState } from "react";
import { catalogError, type CatalogError } from "../../services/catalog";

export function useInsights<T>(read: () => Promise<T>, revision: object | null) {
  const [attempt, setAttempt] = useState(0);
  const [result, setResult] = useState<{ data: T; revision: object | null } | null>(null);
  const [error, setError] = useState<CatalogError | null>(null);
  useEffect(() => {
    let active = true;
    void read().then((data) => {
      if (active) {
        setResult({ data, revision });
        setError(null);
      }
    }).catch((reason: unknown) => {
      if (active) setError(catalogError(reason));
    });
    return () => { active = false; };
  }, [read, revision, attempt]);
  return { data: result?.data, ready: !!result && result.revision === revision, error, retry: () => {
    setError(null);
    setAttempt((previous) => previous + 1);
  } };
}
