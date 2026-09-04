import { useEffect, useRef, useState } from "react";
import {
  registerHeights,
  unregisterHeights,
  subscribeHeights,
} from "@/lib/row-heights";

/**
 * Synchronizes the height of a table's data rows with another table on the
 * same screen. Each table measures its own `tr[data-idx]` rows and publishes
 * them to a shared store; both tables then apply the largest height for each
 * row index, so whichever row is taller, the other matches it.
 */
export function useSharedRowHeights<T>(
  rows: T[],
  side: string,
) {
  const tbodyRef = useRef<HTMLTableSectionElement | null>(null);
  const [heights, setHeights] = useState<Record<number, number>>({});
  const sideRef = useRef(side);
  sideRef.current = side;
  const rowsRef = useRef(rows);
  rowsRef.current = rows;

  useEffect(() => subscribeHeights(setHeights), []);

  useEffect(() => {
    const measure = () => {
      const el = tbodyRef.current;
      if (!el) return;
      const own: Record<number, number> = {};
      const trs = Array.from(el.querySelectorAll<HTMLTableRowElement>("tr[data-idx]"));
      for (const tr of trs) {
        const idx = Number(tr.getAttribute("data-idx"));
        if (Number.isFinite(idx)) {
          own[idx] = tr.getBoundingClientRect().height;
        }
      }
      registerHeights(sideRef.current, own);
    };

    measure();
    const timer = window.setTimeout(measure, 120);
    const ro = new ResizeObserver(measure);
    if (tbodyRef.current) {
      ro.observe(tbodyRef.current);
    }

    return () => {
      window.clearTimeout(timer);
      ro.disconnect();
      unregisterHeights(sideRef.current);
    };
  }, [rows, side]);

  return { tbodyRef, heights };
}
