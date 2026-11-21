import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";
import { useLocale } from "../locales/locale";

export function Cover({ coverKey, placeholder, panorama = false, eager = false }: {
  coverKey: string | null;
  placeholder?: string;
  panorama?: boolean;
  eager?: boolean;
}) {
  const { t } = useLocale();
  const [failed, setFailed] = useState<string | null>(null);
  return (
    <div className={panorama ? "cover cover--panorama" : "cover"}>
      {coverKey && failed !== coverKey ? (
        <img src={convertFileSrc(coverKey, "mylist-cover")} alt="" loading={eager ? "eager" : "lazy"} decoding="async"
          onError={() => setFailed(coverKey)} />
      ) : (
        <div className="cover__placeholder">
          <div className="film-frame" aria-hidden="true"><span /><span /><span /></div>
          {placeholder ?? t.library.placeholder}
        </div>
      )}
    </div>
  );
}
