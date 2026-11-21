import { useEffect, useId, useRef, type ReactNode } from "react";

export function Modal({ title, busy, onCancel, children }: {
  title: string;
  busy: boolean;
  onCancel: () => void;
  children: ReactNode;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const dialog = ref.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);

  return (
    <dialog ref={ref} className="modal" aria-labelledby={titleId} onCancel={(event) => {
      event.preventDefault();
      if (!busy) onCancel();
    }}>
      <h2 id={titleId}>{title}</h2>
      {children}
    </dialog>
  );
}
