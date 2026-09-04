type Listener = (heights: Record<number, number>) => void;

const contributors = new Map<string, Record<number, number>>();
const listeners = new Set<Listener>();

export function registerHeights(id: string, heights: Record<number, number>) {
  contributors.set(id, heights);
  emit();
}

export function unregisterHeights(id: string) {
  contributors.delete(id);
  emit();
}

export function subscribeHeights(fn: Listener): () => void {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
}

function emit() {
  const merged: Record<number, number> = {};
  for (const heights of contributors.values()) {
    for (const [i, h] of Object.entries(heights)) {
      const idx = Number(i);
      if (h > (merged[idx] || 0)) {
        merged[idx] = h;
      }
    }
  }
  for (const fn of listeners) {
    fn(merged);
  }
}
