import { create } from 'zustand';
import { DEFAULT_NODE_ENDPOINT } from '../constants/node';
import type { ChainConnectionState } from '../types/chain';

type SessionStore = {
  endpoint: string;
  state: ChainConnectionState;
  error?: string;
  setEndpoint: (endpoint: string) => void;
  setState: (state: ChainConnectionState, error?: string) => void;
};

export const useSessionStore = create<SessionStore>((set) => ({
  endpoint: DEFAULT_NODE_ENDPOINT,
  state: 'idle',
  error: undefined,
  setEndpoint: (endpoint) => set({ endpoint }),
  setState: (state, error) => set({ state, error })
}));
