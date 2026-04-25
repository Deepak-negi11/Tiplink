"use client";

import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Copy, Check, Link as LinkIcon, Loader2, ArrowLeft,
  Send as SendIcon, User, AtSign, Wallet,
} from "lucide-react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useAuthStore, BalanceEntry } from "@/store/useStore";
import { fetchApi } from "@/lib/api";

type SendMode = "address" | "email" | "link";

interface LookupResult {
  found: boolean;
  public_key?: string;
  email?: string;
}

function TokenIcon({ symbol }: { symbol: string }) {
  const colors: Record<string, { bg: string; text: string; border: string }> = {
    SOL: { bg: "bg-violet-500/15", text: "text-violet-400", border: "border-violet-500/20" },
    USDC: { bg: "bg-sky-500/15", text: "text-sky-400", border: "border-sky-500/20" },
    WBTC: { bg: "bg-orange-500/15", text: "text-orange-400", border: "border-orange-500/20" },
  };
  const c = colors[symbol] || { bg: "bg-zinc-500/15", text: "text-zinc-400", border: "border-zinc-500/20" };
  return (
    <div className={`w-7 h-7 rounded-full ${c.bg} flex items-center justify-center border ${c.border}`}>
      <span className={`${c.text} font-bold text-[10px]`}>{symbol}</span>
    </div>
  );
}

export default function SendPage() {
  const { token, balances, balancesLoaded } = useAuthStore();
  const [amount, setAmount] = useState("");
  const [asset, setAsset] = useState("SOL");
  const [recipient, setRecipient] = useState("");
  const [memo, setMemo] = useState("");
  const [mode, setMode] = useState<SendMode>("address");
  const [loading, setLoading] = useState(false);
  const [lookupLoading, setLookupLoading] = useState(false);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);
  const [localBalances, setLocalBalances] = useState<BalanceEntry[]>([]);

  const [success, setSuccess] = useState(false);
  const [resultType, setResultType] = useState<"direct" | "link">("direct");
  const [resultData, setResultData] = useState<{ signature?: string; claim_url?: string }>({});

  const [lookupResult, setLookupResult] = useState<LookupResult | null>(null);

  useEffect(() => {
    async function loadBalances() {
      if (balancesLoaded && balances.length > 0) {
        setLocalBalances(balances);
        return;
      }
      if (!token) return;
      try {
        const data = await fetchApi<BalanceEntry[]>("/wallet/balance", { token });
        setLocalBalances(data);
      } catch {}
    }
    loadBalances();
  }, [token, balances, balancesLoaded]);

  const selectedBalance = localBalances.find((b) => b.symbol === asset);
  const availableHuman = selectedBalance
    ? (selectedBalance.available / 10 ** selectedBalance.decimals).toFixed(selectedBalance.decimals > 6 ? 4 : 2)
    : "0.00";

  const tokenMint = asset === "SOL"
    ? "So11111111111111111111111111111111111111112"
    : asset === "USDC"
    ? "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    : "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh";

  useEffect(() => {
    if (!recipient) {
      setLookupResult(null);
      return;
    }
    if (recipient.includes("@")) {
      setMode("email");
    }
    else if (/^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(recipient)) {
      setMode("address");
    }
    else {
      setMode("link");
    }
  }, [recipient]);

  useEffect(() => {
    if (!recipient || mode === "link") {
      setLookupResult(null);
      return;
    }

    const timer = setTimeout(async () => {
      if (!token) return;
      setLookupLoading(true);
      try {
        const lookup = await fetchApi<LookupResult>("/user/lookup", {
          method: "POST",
          token,
          body: mode === "email"
            ? { email: recipient }
            : { public_key: recipient },
        });
        setLookupResult(lookup);
      } catch {
        setLookupResult(null);
      } finally {
        setLookupLoading(false);
      }
    }, 600);

    return () => clearTimeout(timer);
  }, [recipient, mode, token]);

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!amount || parseFloat(amount) <= 0) {
      setError("Enter a valid amount");
      return;
    }
    if (!recipient && mode !== "link") {
      setError("Enter a recipient");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const recipientExists = lookupResult?.found === true;
      const recipientPubkey = lookupResult?.public_key;

      if (recipientExists && recipientPubkey) {
        const now = Math.floor(Date.now() / 1000);
        const sendData = await fetchApi<{ nonce: string; unsigned_tx: string }>("/wallet/send", {
          method: "POST",
          token,
          body: {
            to: recipientPubkey,
            amount: amount,
            mint: tokenMint,
            timestamp: now,
            signature: "client_intent_sig", // placeholder for intent signature
          },
        });

        setResultType("direct");
        setResultData({ signature: sendData.nonce });
        setSuccess(true);
      } else {
        const data = await fetchApi<{ link_id: string; claim_url: string }>("/link/create", {
          method: "POST",
          token,
          body: {
            amount: parseFloat(amount),
            token_mint: tokenMint,
          },
        });

        const claimUrl = `${window.location.origin}${data.claim_url}`;
        setResultType("link");
        setResultData({ claim_url: claimUrl });
        setSuccess(true);
      }
    } catch (err: any) {
      setError(err.message || "Transaction failed");
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    if (resultData.claim_url) {
      navigator.clipboard.writeText(resultData.claim_url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleReset = () => {
    setSuccess(false);
    setResultData({});
    setAmount("");
    setRecipient("");
    setMemo("");
    setLookupResult(null);
  };

  if (success) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="flex flex-col items-center justify-center max-w-md mx-auto pt-10"
      >
        <div className="w-20 h-20 bg-emerald-500/10 rounded-full flex items-center justify-center mb-8 border border-emerald-500/20">
          <Check className="w-10 h-10 text-emerald-400" />
        </div>

        {resultType === "direct" ? (
          <>
            <h1 className="text-3xl font-bold mb-2 text-center text-glow-green">Sent!</h1>
            <p className="text-zinc-400 text-center mb-4 text-sm">
              {amount} {asset} sent directly to {recipient.slice(0, 8)}...
            </p>
            <Card className="w-full p-5">
              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-500">Transaction</span>
                <span className="text-zinc-300 font-mono text-xs">
                  {resultData.signature?.slice(0, 16)}...
                </span>
              </div>
            </Card>
          </>
        ) : (
          <>
            <h1 className="text-3xl font-bold mb-2 text-center text-glow-green">Link Created!</h1>
            <p className="text-zinc-400 text-center mb-2 text-sm">
              Recipient doesn&apos;t have a TipLink account yet.
            </p>
            <p className="text-zinc-500 text-center mb-8 text-xs">
              Funds are locked. Share this link — anyone with it can claim {amount} {asset}.
            </p>

            <Card className="w-full p-5 relative overflow-hidden group">
              <div className="absolute inset-0 bg-gradient-to-r from-indigo-500/[0.03] to-violet-500/[0.03] opacity-0 group-hover:opacity-100 transition-opacity" />
              <div className="flex bg-black/30 border border-white/[0.06] rounded-xl p-3 items-center gap-3">
                <LinkIcon className="w-4 h-4 text-zinc-500 shrink-0 hidden sm:block" />
                <input
                  readOnly
                  value={resultData.claim_url || ""}
                  className="bg-transparent flex-1 outline-none text-zinc-300 font-mono text-sm w-full truncate"
                />
                <Button size="icon" variant="secondary" onClick={handleCopy} className="shrink-0">
                  {copied ? <Check className="w-4 h-4 text-emerald-400" /> : <Copy className="w-4 h-4" />}
                </Button>
              </div>
            </Card>
          </>
        )}

        <Button className="mt-8 w-full" size="lg" onClick={handleReset}>
          <ArrowLeft className="w-4 h-4 mr-2" /> Send Another
        </Button>
      </motion.div>
    );
  }

  return (
    <div className="flex flex-col max-w-md mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-[-0.03em] mb-1">Send</h1>
        <p className="text-zinc-500 text-sm">Send crypto directly or via a shareable link.</p>
      </div>

      <Card className="p-6 backdrop-blur-3xl relative overflow-hidden">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-indigo-500/30 to-transparent" />

        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="mb-5 p-3 rounded-xl bg-red-500/[0.08] border border-red-500/15 text-red-400 text-sm"
            >
              {error}
            </motion.div>
          )}
        </AnimatePresence>

        <form onSubmit={handleSend} className="flex flex-col gap-6">
          {/* Recipient Input */}
          <div className="flex flex-col gap-2">
            <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider">
              Recipient
            </label>
            <div className="relative">
              <input
                type="text"
                placeholder="Wallet address, email, or leave empty for link"
                value={recipient}
                onChange={(e) => setRecipient(e.target.value)}
                className="w-full bg-white/[0.03] border border-white/[0.06] rounded-xl px-4 py-3 pl-10 text-sm text-white placeholder:text-zinc-600 outline-none focus:ring-2 focus:ring-indigo-500/30 transition-all"
              />
              <div className="absolute left-3 top-1/2 -translate-y-1/2">
                {mode === "email" ? (
                  <AtSign className="w-4 h-4 text-zinc-500" />
                ) : mode === "address" ? (
                  <Wallet className="w-4 h-4 text-zinc-500" />
                ) : (
                  <User className="w-4 h-4 text-zinc-500" />
                )}
              </div>
            </div>

            {/* Lookup status indicator */}
            <AnimatePresence>
              {lookupLoading && (
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                  className="flex items-center gap-2 text-xs text-zinc-500"
                >
                  <Loader2 className="w-3 h-3 animate-spin" /> Looking up recipient...
                </motion.div>
              )}
              {lookupResult && !lookupLoading && (
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                  className={`flex items-center gap-2 text-xs ${
                    lookupResult.found
                      ? "text-emerald-400"
                      : "text-amber-400"
                  }`}
                >
                  {lookupResult.found ? (
                    <>
                      <Check className="w-3 h-3" />
                      TipLink user found — will send directly
                    </>
                  ) : (
                    <>
                      <LinkIcon className="w-3 h-3" />
                      Not on TipLink — will create a claimable link
                    </>
                  )}
                </motion.div>
              )}
              {!recipient && (
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                  className="flex items-center gap-2 text-xs text-zinc-600"
                >
                  <LinkIcon className="w-3 h-3" />
                  Leave empty to generate a shareable link
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          {/* Amount Input */}
          <div className="flex flex-col gap-2">
            <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider">You Send</label>
            <div className="flex items-center gap-3">
              <input
                type="number"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="bg-transparent text-5xl font-bold text-white outline-none w-full placeholder:text-zinc-800 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                required
                autoFocus
                step="any"
              />
              <div className="relative">
                <select
                  value={asset}
                  onChange={(e) => setAsset(e.target.value)}
                  className="bg-white/[0.05] text-white rounded-xl pl-10 pr-4 py-2.5 border border-white/[0.06] outline-none appearance-none font-semibold cursor-pointer hover:bg-white/[0.08] transition-colors text-sm"
                >
                  <option value="SOL">SOL</option>
                  <option value="USDC">USDC</option>
                  <option value="WBTC">WBTC</option>
                </select>
                <div className="absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none">
                  <TokenIcon symbol={asset} />
                </div>
              </div>
            </div>
            <p className="text-xs text-zinc-600 mt-1">Available: {availableHuman} {asset}</p>
          </div>

          {/* Memo */}
          <div className="flex flex-col gap-2">
            <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider">Memo (optional)</label>
            <input
              type="text"
              placeholder="e.g. Coffee money ☕"
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              className="bg-white/[0.03] border border-white/[0.06] rounded-xl px-4 py-3 text-sm text-white placeholder:text-zinc-600 outline-none focus:ring-2 focus:ring-indigo-500/30 transition-all"
            />
          </div>

          <div className="h-px w-full bg-white/[0.04]" />

          <div className="flex items-center justify-between text-sm">
            <span className="text-zinc-500">Method</span>
            <span className="text-zinc-300 font-mono text-xs">
              {lookupResult?.found ? "⚡ Direct Transfer" : "🔗 TipLink"}
            </span>
          </div>

          <div className="flex items-center justify-between text-sm">
            <span className="text-zinc-500">Network Fee</span>
            <span className="text-zinc-300 font-mono text-xs">~0.000005 SOL</span>
          </div>

          <Button type="submit" size="lg" className="w-full mt-2 py-6 rounded-2xl" disabled={loading}>
            {loading ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : lookupResult?.found ? (
              <>
                <SendIcon className="w-4 h-4 mr-2" /> Send Directly
              </>
            ) : (
              <>
                <LinkIcon className="w-4 h-4 mr-2" /> {recipient ? "Create Link" : "Generate Link"}
              </>
            )}
          </Button>
        </form>
      </Card>
    </div>
  );
}
