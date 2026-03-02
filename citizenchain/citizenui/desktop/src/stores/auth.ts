import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { LoginIdentity, LoginSession } from '../types/auth';

const AUTH_SESSION_STORAGE_KEY = 'citizenui.auth.session';
const AUTH_SESSION_TTL_SECONDS = 15 * 60;

function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

function sanitizeSession(session: LoginSession | null): LoginSession | null {
  if (!session) {
    return null;
  }
  return session.expiresAt > nowSeconds() ? session : null;
}

type AuthStore = {
  session: LoginSession | null;
  login: (identity: LoginIdentity) => void;
  logout: () => void;
  hydrateSession: () => void;
};

export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      session: null,
      login: (identity) => {
        const issuedAt = nowSeconds();
        set({
          session: {
            ...identity,
            issuedAt,
            expiresAt: issuedAt + AUTH_SESSION_TTL_SECONDS
          }
        });
      },
      logout: () => set({ session: null }),
      hydrateSession: () => set({ session: sanitizeSession(get().session) })
    }),
    {
      name: AUTH_SESSION_STORAGE_KEY,
      partialize: (state) => ({ session: state.session }),
      onRehydrateStorage: () => (state) => {
        if (!state) {
          return;
        }
        state.hydrateSession();
      }
    }
  )
);
