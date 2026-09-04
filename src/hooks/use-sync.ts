import { useCallback, useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { syncCommands } from "../db/commands";

export function useSyncStatus() {
  const [arabicLabel, setArabicLabel] = useState<string>("غير متصل");
  const [status, setStatus] = useState<string>("offline");

  useEffect(() => {
    let mounted = true;
    const poll = async () => {
      try {
        const [s, label] = await Promise.all([
          syncCommands.getStatus(),
          syncCommands.getArabicStatus(),
        ]);
        if (mounted) {
          setStatus(s);
          setArabicLabel(label);
        }
      } catch {
        if (mounted) {
          setStatus("error");
          setArabicLabel("تعذر الاتصال");
        }
      }
    };
    poll();
    const interval = setInterval(poll, 10000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  return { status, arabicLabel };
}

export function useSyncPush() {
  const qc = useQueryClient();
  return useCallback(async () => {
    const result = await syncCommands.push();
    qc.invalidateQueries({ queryKey: ["sync"] });
    return result;
  }, [qc]);
}

export function useSyncPull() {
  const qc = useQueryClient();
  return useCallback(async () => {
    const result = await syncCommands.pull();
    qc.invalidateQueries({ queryKey: ["sync"] });
    return result;
  }, [qc]);
}

export function useSyncFull() {
  const qc = useQueryClient();
  return useCallback(async () => {
    const result = await syncCommands.full();
    qc.invalidateQueries({ queryKey: ["sync"] });
    return result;
  }, [qc]);
}
