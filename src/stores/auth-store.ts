import { create } from "zustand";
import type { User } from "../db/types";

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  setAuth: (user: User, token: string) => void;
  clearAuth: () => void;
}

const STORAGE_KEY = "auth-session";

function loadSession(): { user: User | null; token: string | null } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { user: null, token: null };
    const parsed = JSON.parse(raw);
    if (parsed && parsed.user && parsed.token) {
      return { user: parsed.user as User, token: parsed.token as string };
    }
  } catch {
    // ignore corrupted storage
  }
  return { user: null, token: null };
}

function saveSession(user: User, token: string) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ user, token }));
  } catch {
    // storage may be unavailable; session just won't survive a refresh
  }
}

const saved = loadSession();

export const useAuthStore = create<AuthState>((set) => ({
  user: saved.user,
  token: saved.token,
  isAuthenticated: !!saved.user && !!saved.token,

  setAuth: (user, token) => {
    saveSession(user, token);
    set({ user, token, isAuthenticated: true });
  },

  clearAuth: () => {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
    set({ user: null, token: null, isAuthenticated: false });
  },
}));