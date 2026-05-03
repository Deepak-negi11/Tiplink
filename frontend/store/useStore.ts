import { create } from "zustand";
import { persist } from "zustand/middleware";

interface User {
  id: string;
  email: string;
  public_key: string;
}

export interface BalanceEntry {
  id: string;
  mint: string;
  symbol: string;
  amount: number;
  available: number;
  locked: number;
  decimals: number;
}

interface AuthState {
  user: User | null;
  token: string | null;
  balances: BalanceEntry[];
  balancesLoaded: boolean;
  login: (user: User, token: string) => void;
  logout: () => void;
  setBalances: (balances: BalanceEntry[]) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      balances: [],
      balancesLoaded: false,
      login: (user, token) => set({ user, token }),
      logout: () => set({ user: null, token: null, balances: [], balancesLoaded: false }),
      setBalances: (balances) => set({ balances, balancesLoaded: true }),
    }),
    {
      name: "orbit-auth",
    }
  )
);
