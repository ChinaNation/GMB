import { create } from 'zustand';
import type { LoginSession } from '../types/auth';

type AuthStore = {
  session: LoginSession | null;
  login: (session: LoginSession) => void;
  logout: () => void;
};

export const useAuthStore = create<AuthStore>((set) => ({
  session: null,
  login: (session) => set({ session }),
  logout: () => set({ session: null })
}));
