"use client";

import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  ArrowUpRight, ArrowDownLeft, Copy, Check,
  ExternalLink, Loader2, TrendingUp,
} from "lucide-react";
import Link from "next/link";
import { useAuthStore, BalanceEntry } from "@/store/useStore";
import { fetchApi } from "@/lib/api";

/** Fetches live SOL/USD price from Jupiter Price API v2 */
async function fetchSolPrice(): Promise<number> {
  try {
    const res = await fetch(
      "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd"
    );
    const data = await res.json();
    return Number(data?.solana?.usd ?? 0);
  } catch {
    return 0;
  }
}

interface TransactionEntry {
  id: string;
  amount: number;
  token_symbol: string;
  tx_hash: string;
  tx_type: string;
  from_address: string;
  to_address: string;
  block_time: string;
}

function Skeleton({ className }: { className?: string }) {
  return <div className={`skeleton ${className}`} />;
}

function TokenBadge({ symbol }: { symbol: string }) {
  const styles: Record<string, { ring: string; text: string; bg: string }> = {
    SOL:  { ring: "border-violet-500/25", text: "text-violet-400",  bg: "bg-violet-500/10" },
    USDC: { ring: "border-sky-500/25",    text: "text-sky-400",     bg: "bg-sky-500/10" },
    USDT: { ring: "border-emerald-500/25",text: "text-emerald-400", bg: "bg-emerald-500/10" },
  };
  const s = styles[symbol] || { ring: "border-white/10", text: "text-[#888880]", bg: "bg-white/5" };
  return (
    <div className={`w-10 h-10 rounded-full ${s.bg} border ${s.ring} flex items-center justify-center shrink-0`}>
      <span className={`${s.text} font-bold text-[10px] font-display tracking-wide`}>
        {symbol.slice(0, 3)}
      </span>
    </div>
  );
}

function AnimatedBalance({ value }: { value: number }) {
  const [display, setDisplay] = useState(0);
  useEffect(() => {
    const duration = 900;
    const start = performance.now();
    const to = value;
    function tick(now: number) {
      const elapsed = now - start;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      setDisplay(to * eased);
      if (progress < 1) requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  }, [value]);
  return <span>${display.toFixed(2)}</span>;
}

export default function Dashboard() {
  const { user, token, setBalances } = useAuthStore();
  const [balances, setLocalBalances] = useState<BalanceEntry[]>([]);
  const [transactions, setTransactions] = useState<TransactionEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [txLoading, setTxLoading] = useState(true);
  const [copied, setCopied] = useState(false);
  const [showReceive, setShowReceive] = useState(false);
  const [solPrice, setSolPrice] = useState(0);

  const loadBalances = useCallback(async () => {
    if (!token) return;
    try {
      const data = await fetchApi<BalanceEntry[]>("/wallet/balance", { token });
      setLocalBalances(data);
      setBalances(data);
    } catch {
      setLocalBalances([]);
    } finally {
      setLoading(false);
    }
  }, [token, setBalances]);

  const loadTransactions = useCallback(async () => {
    if (!token) return;
    try {
      const data = await fetchApi<TransactionEntry[]>("/wallet/history?limit=10", { token });
      if (Array.isArray(data)) setTransactions(data);
    } catch {
      setTransactions([]);
    } finally {
      setTxLoading(false);
    }
  }, [token]);

  useEffect(() => {
    loadBalances();
    loadTransactions();
    fetchSolPrice().then(setSolPrice);
    // Refresh SOL price every 30 seconds
    const priceInterval = setInterval(() => fetchSolPrice().then(setSolPrice), 30_000);
    return () => clearInterval(priceInterval);
  }, [loadBalances, loadTransactions]);

  const totalUsd = balances.reduce((sum, b) => {
    if (b.symbol === "SOL") return sum + (b.available / 10 ** b.decimals) * solPrice;
    if (b.symbol === "USDC" || b.symbol === "USDT") return sum + b.available / 10 ** b.decimals;
    return sum;
  }, 0);

  const shortKey = user?.public_key
    ? `${user.public_key.slice(0, 6)}...${user.public_key.slice(-4)}`
    : "Not connected";

  const handleCopy = () => {
    if (user?.public_key) {
      navigator.clipboard.writeText(user.public_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="flex flex-col gap-8">

      {/* ── Balance Hero ── */}
      <section className="animate-fade-up">
        <div className="flex items-center justify-between mb-3">
          <p className="text-xs font-display font-semibold text-[#555550] uppercase tracking-widest">
            Total Portfolio
          </p>
          <button
            onClick={handleCopy}
            className="flex items-center gap-2 text-xs text-[#555550] hover:text-[#e8e3d5] transition-colors bg-[#1a1a1a] border border-[rgba(245,197,24,0.1)] rounded-lg px-3 py-1.5 hover:border-[rgba(245,197,24,0.25)]"
          >
            <div className="w-1.5 h-1.5 rounded-full bg-[#00d26a] animate-pulse-green" />
            <span className="font-mono">{shortKey}</span>
            {copied
              ? <Check className="w-3 h-3 text-[#00d26a]" />
              : <Copy className="w-3 h-3" />}
          </button>
        </div>

        {loading ? (
          <Skeleton className="h-16 w-56 mb-2" />
        ) : (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-baseline gap-3"
          >
            <h1 className="font-display text-6xl font-bold tracking-[-0.04em] text-white text-glow-yellow">
              <AnimatedBalance value={totalUsd} />
            </h1>
            <span className="flex items-center gap-1 text-[#00d26a] text-sm font-semibold">
              <TrendingUp className="w-3.5 h-3.5" /> Live
            </span>
          </motion.div>
        )}
        <p className="text-[#555550] text-sm mt-1">USD equivalent value</p>
      </section>

      {/* ── Quick Actions ── */}
      <section className="flex gap-3 animate-fade-up animate-fade-up-delay-1">
        <Link href="/dashboard/create">
          <Button size="lg" className="gap-2 rounded-2xl">
            <ArrowUpRight className="w-5 h-5" />
            Send
          </Button>
        </Link>
        <Button
          size="lg"
          variant="outline"
          className="gap-2 rounded-2xl"
          onClick={() => setShowReceive(!showReceive)}
        >
          <ArrowDownLeft className="w-5 h-5" />
          Receive
        </Button>
      </section>

      {/* ── Receive address ── */}
      {showReceive && user?.public_key && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          exit={{ opacity: 0, height: 0 }}
        >
          <Card className="p-5 bg-[#f5c518]/[0.04] border-[#f5c518]/15">
            <p className="text-sm text-[#888880] mb-3 font-display font-semibold">
              Your deposit address:
            </p>
            <div className="flex items-center gap-3 bg-[#0d0d0d] border border-[rgba(245,197,24,0.1)] rounded-xl p-3">
              <code className="flex-1 text-sm text-[#e8e3d5] font-mono break-all">
                {user.public_key}
              </code>
              <Button size="icon" variant="ghost" onClick={handleCopy} className="shrink-0">
                {copied
                  ? <Check className="w-4 h-4 text-[#00d26a]" />
                  : <Copy className="w-4 h-4" />}
              </Button>
            </div>
          </Card>
        </motion.div>
      )}

      {/* ── Assets ── */}
      <section className="flex flex-col gap-3 animate-fade-up animate-fade-up-delay-2">
        <h2 className="font-display text-lg font-bold text-[#e8e3d5] tracking-tight">Assets</h2>
        <Card className="overflow-hidden">
          {loading ? (
            <div className="p-5 flex flex-col gap-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-4">
                  <Skeleton className="w-10 h-10 rounded-full" />
                  <div className="flex-1 flex flex-col gap-2">
                    <Skeleton className="h-4 w-24" />
                    <Skeleton className="h-3 w-16" />
                  </div>
                  <Skeleton className="h-4 w-20" />
                </div>
              ))}
            </div>
          ) : balances.length === 0 ? (
            <div className="p-12 text-center">
              <div className="w-12 h-12 rounded-full bg-[#f5c518]/10 border border-[#f5c518]/20 flex items-center justify-center mx-auto mb-4">
                <ArrowDownLeft className="w-5 h-5 text-[#f5c518]" />
              </div>
              <p className="text-[#888880] text-sm font-display font-semibold">No assets yet</p>
              <p className="text-[#555550] text-xs mt-1">Receive tokens to get started.</p>
            </div>
          ) : (
            balances.map((b, i) => {
              const human = (b.available / 10 ** b.decimals).toFixed(b.decimals > 6 ? 4 : 2);
              const usd = b.symbol === "SOL"
                ? (b.available / 10 ** b.decimals) * 170
                : b.symbol === "USDC" || b.symbol === "USDT"
                  ? b.available / 10 ** b.decimals : 0;
              return (
                <motion.div
                  key={b.mint}
                  initial={{ opacity: 0, x: -8 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: i * 0.06 }}
                  className="flex items-center justify-between px-5 py-4 hover:bg-[#f5c518]/[0.03] transition-colors border-b border-[rgba(245,197,24,0.06)] last:border-0"
                >
                  <div className="flex items-center gap-4">
                    <TokenBadge symbol={b.symbol} />
                    <div>
                      <p className="font-display font-bold text-sm text-white">{b.symbol}</p>
                      <p className="text-xs text-[#555550] font-mono">{b.mint.slice(0, 8)}...</p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="font-display font-bold text-sm text-white">{human}</p>
                    {usd > 0 && (
                      <p className="text-xs text-[#555550]">${usd.toFixed(2)}</p>
                    )}
                  </div>
                </motion.div>
              );
            })
          )}
        </Card>
      </section>

      {/* ── Transaction history ── */}
      <section className="flex flex-col gap-3 animate-fade-up animate-fade-up-delay-3">
        <h2 className="font-display text-lg font-bold text-[#e8e3d5] tracking-tight">
          Recent Activity
        </h2>
        <Card className="overflow-hidden">
          {txLoading ? (
            <div className="p-5 flex flex-col gap-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-4">
                  <Skeleton className="w-9 h-9 rounded-full" />
                  <div className="flex-1 flex flex-col gap-2">
                    <Skeleton className="h-4 w-32" />
                    <Skeleton className="h-3 w-20" />
                  </div>
                  <Skeleton className="h-4 w-16" />
                </div>
              ))}
            </div>
          ) : transactions.length === 0 ? (
            <div className="p-10 text-center">
              <p className="text-[#555550] text-sm font-display">No transactions yet.</p>
              <p className="text-[#333330] text-xs mt-1">
                Activity appears after the indexer picks up on-chain events.
              </p>
            </div>
          ) : (
            transactions.map((tx, i) => {
              const isDeposit = tx.tx_type.toLowerCase().includes("deposit") ||
                                tx.tx_type.toLowerCase().includes("receive");
              return (
                <motion.div
                  key={tx.id}
                  initial={{ opacity: 0, x: -8 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: i * 0.04 }}
                  className="flex items-center justify-between px-5 py-4 hover:bg-[#f5c518]/[0.03] transition-colors border-b border-[rgba(245,197,24,0.06)] last:border-0 group"
                >
                  <div className="flex items-center gap-3">
                    <div className={`w-9 h-9 rounded-full flex items-center justify-center shrink-0 ${
                      isDeposit
                        ? "bg-[#00d26a]/10 border border-[#00d26a]/20"
                        : "bg-[#ff3b30]/10 border border-[#ff3b30]/20"
                    }`}>
                      {isDeposit
                        ? <ArrowDownLeft className="w-4 h-4 text-[#00d26a]" />
                        : <ArrowUpRight   className="w-4 h-4 text-[#ff3b30]" />}
                    </div>
                    <div>
                      <p className="font-display font-semibold text-sm text-white capitalize">
                        {tx.tx_type}
                      </p>
                      <p className="text-xs text-[#555550] font-mono">
                        {tx.tx_hash.slice(0, 8)}...{tx.tx_hash.slice(-4)}
                      </p>
                    </div>
                  </div>
                  <div className="text-right flex items-center gap-2">
                    <div>
                      <p className={`font-display font-bold text-sm ${
                        isDeposit ? "text-[#00d26a]" : "text-[#ff3b30]"
                      }`}>
                        {isDeposit ? "+" : "−"}{tx.amount} {tx.token_symbol}
                      </p>
                      <p className="text-xs text-[#555550]">
                        {new Date(tx.block_time).toLocaleDateString("en-US", {
                          month: "short", day: "numeric",
                        })}
                      </p>
                    </div>
                    <a
                      href={`https://solscan.io/tx/${tx.tx_hash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      <ExternalLink className="w-3.5 h-3.5 text-[#555550] hover:text-[#f5c518]" />
                    </a>
                  </div>
                </motion.div>
              );
            })
          )}
        </Card>
      </section>

      {/* ── Footer ── */}
      <div className="flex items-center justify-center gap-2 py-4 text-xs text-[#333330] font-display">
        <div className="w-1.5 h-1.5 rounded-full bg-[#f5c518]/30" />
        <span>Powered by Solana · MPC Secured · Non-Custodial</span>
        <div className="w-1.5 h-1.5 rounded-full bg-[#f5c518]/30" />
      </div>
    </div>
  );
}
