import { useState } from "react";
import { useLocale } from "../locales/locale";
import type { CatalogError, CoverChange } from "../services/catalog";
import { Cover } from "./Cover";

export function useCoverDraft(onError: (error: CatalogError | null) => void) {
  const [file, setFile] = useState<File | null>(null);
  const [removed, setRemoved] = useState(false);

  function choose(next: File | undefined) {
    if (!next) return;
    if (next.size > 12 * 1024 * 1024 || next.size === 0) {
      onError(next.size ? "cover_too_large" : "invalid_cover");
      return;
    }
    setFile(next);
    setRemoved(false);
    onError(null);
  }

  function remove() {
    setFile(null);
    setRemoved(true);
  }

  async function change(): Promise<CoverChange> {
    if (!file) return { action: removed ? "remove" : "keep" };
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject({ code: "cover_unavailable" });
      reader.onload = () => {
        const data = typeof reader.result === "string" ? reader.result.split(",", 2)[1] : "";
        if (data) resolve({ action: "replace", data });
        else reject({ code: "invalid_cover" });
      };
      reader.readAsDataURL(file);
    });
  }

  return { file, removed, choose, remove, change };
}

export function CoverPicker({ coverKey, draft, busy, panorama = false }: {
  coverKey: string | null;
  draft: ReturnType<typeof useCoverDraft>;
  busy: boolean;
  panorama?: boolean;
}) {
  const { t } = useLocale();
  const copy = t.library;
  const [dragging, setDragging] = useState(false);
  return (
    <fieldset className="cover-picker" data-panorama={panorama} data-dragging={dragging && !busy} onDragOver={(event) => {
      if (!busy && event.dataTransfer.types.includes("Files")) {
        event.preventDefault();
        setDragging(true);
      }
    }} onDragLeave={(event) => {
      if (!(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget)) {
        setDragging(false);
      }
    }} onDrop={(event) => {
      event.preventDefault();
      setDragging(false);
      if (!busy) draft.choose(event.dataTransfer.files[0]);
    }}>
      <legend>{panorama ? copy.panorama : copy.cover}</legend>
      <div className="cover-picker__layout">
        <Cover coverKey={draft.file || draft.removed ? null : coverKey}
          panorama={panorama} placeholder={panorama
            ? (draft.file ? copy.panoramaSelected : copy.panoramaPlaceholder)
            : (draft.file ? copy.coverSelected : undefined)} />
        <div className="field">
          <label className="field">{panorama ? copy.choosePanorama : copy.chooseCover}
            <input type="file" accept="image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp"
              onChange={(event) => {
                draft.choose(event.target.files?.[0]);
                event.target.value = "";
              }} />
          </label>
          {draft.file && <p className="field-hint">{draft.file.name}</p>}
          <p className="field-hint">{copy.coverHint}</p>
          {panorama && <p className="field-hint">{copy.panoramaHint}</p>}
          {(draft.file || (coverKey && !draft.removed)) &&
            <button type="button" onClick={draft.remove}>{panorama ? copy.removePanorama : copy.removeCover}</button>}
        </div>
      </div>
    </fieldset>
  );
}
