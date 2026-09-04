import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";

const SYNC_DATA_CHANGED_EVENT = "sync-data-changed";

/**
 * Listens for the Rust backend "sync-data-changed" event (emitted after a
 * successful pull that imported new/updated remote records) and invalidates
 * the incoming/outgoing queries so the UI re-renders with fresh data quickly.
 */
export function useSyncRealtime() {
  const qc = useQueryClient();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listen(SYNC_DATA_CHANGED_EVENT, () => {
      qc.invalidateQueries({ queryKey: ["incoming"] });
      qc.invalidateQueries({ queryKey: ["outgoing"] });
    }).then((fn) => {
      if (!cancelled) {
        unlisten = fn;
      } else {
        fn();
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [qc]);
}
