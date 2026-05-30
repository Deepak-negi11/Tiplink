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
  network: "mainnet" | "devnet";
  login: (user: User, token: string) => void;
  logout: () => void;
  setBalances: (balances: BalanceEntry[]) => void;
  setNetwork: (network: "mainnet" | "devnet") => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      balances: [],
      balancesLoaded: false,
      network: "mainnet",
      login: (user, token) => set({ user, token }),
      logout: () => set({ user: null, token: null, balances: [], balancesLoaded: false }),
      setBalances: (balances) => set({ balances, balancesLoaded: true }),
      setNetwork: (network) => set({ network }),
    }),
    {
      name: "orbit-auth",
    }
  )
);
