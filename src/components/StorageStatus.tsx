import { useEffect, useState } from "react";
import { useLocale } from "../locales/locale";
import { es } from "../locales/es";
import { checkStorage, type StorageErrorCode } from "../services/storage";

type Status =
  | { kind: "checking" }
  | { kind: "ready"; version: number }
  | { kind: "error"; code: StorageErrorCode };

function errorCode(error: unknown): StorageErrorCode {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string" &&
    Object.hasOwn(es.storage.errors, error.code)
  ) {
    return error.code as StorageErrorCode;
  }

  return "diagnostic_failed";
}

export function StorageStatus() {
  const { t: { storage } } = useLocale();
  const [status, setStatus] = useState<Status>({ kind: "checking" });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;

    void (async () => {
      try {
        const { schemaVersion } = await checkStorage();
        if (active) setStatus({ kind: "ready", version: schemaVersion });
      } catch (error: unknown) {
        if (active) setStatus({ kind: "error", code: errorCode(error) });
      }
    })();

    return () => {
      active = false;
    };
  }, [attempt]);

  const message =
    status.kind === "checking"
      ? storage.checking
      : status.kind === "ready"
        ? storage.ready(status.version)
        : storage.errors[status.code];

  return (
    <footer className="storage">
      <span className="storage__label">
        <span className="storage__signal" data-state={status.kind} aria-hidden="true" />
        {storage.label}
      </span>
      <p className="storage__message" role="status">{message}</p>
      {status.kind === "error" && (
        <button
          className="storage__retry"
          type="button"
          onClick={() => {
            setStatus({ kind: "checking" });
            setAttempt((current) => current + 1);
          }}
        >
          {storage.retry}
        </button>
      )}
    </footer>
  );
}
