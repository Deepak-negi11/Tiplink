"use client";

import { useAuthStore } from "@/store/useStore";
import { QRCodeSVG } from "qrcode.react";
import { motion } from "framer-motion";
import { Copy, CheckCircle2, Wallet, Info, CreditCard, Loader2, ArrowDownToLine } from "lucide-react";
import { useState, useEffect, useCallback } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { fetchApi } from "@/lib/api";

interface Quote {
  quoteCurrencyAmount: number;
  feeAmount: number;
  totalAmount: number;
}

interface Limits {
  baseCurrencyMinBuyAmount: number;
  baseCurrencyMaxBuyAmount: number;
}

export default function DepositPage() {
  const { user, token } = useAuthStore();
  const [copied, setCopied] = useState(false);
  const [tab, setTab] = useState<"buy" | "transfer">("buy");
  const [amount, setAmount] = useState("50");
  const [currency, setCurrency] = useState("sol");
  const [quote, setQuote] = useState<Quote | null>(null);
  const [limits, setLimits] = useState<Limits | null>(null);
  const [loading, setLoading] = useState(false);
  const [showWidget, setShowWidget] = useState(false);

  const handleCopy = async () => {
    if (!user?.public_key) return;
    await navigator.clipboard.writeText(user.public_key);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Fetch limits when currency changes
  useEffect(() => {
    fetchApi<Limits>(`/moonpay/limits?currency_code=${currency}`, { token })
      .then(setLimits)
      .catch(console.error);
  }, [currency, token]);

  // Fetch quote when amount changes (debounced)
  useEffect(() => {
    if (!amount || parseFloat(amount) < (limits?.baseCurrencyMinBuyAmount ?? 0)) {
      setQuote(null);
      return;
    }
    const timer = setTimeout(async () => {
      try {
        const data = await fetchApi<Quote>(
          `/moonpay/quote?currency_code=${currency}&fiat_currency=usd&fiat_amount=${amount}`,
          { token }
        );
        setQuote(data);
      } catch {
        setQuote(null);
      }
    }, 500);
    return () => clearTimeout(timer);
  }, [amount, currency, limits, token]);

  const handleBuy = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchApi<{ signed_url: string }>(
        "/moonpay/sign-url",
        { 
          method: "POST", 
          body: { 
            currency_code: currency,
            base_currency_code: "usd",
            base_currency_amount: parseFloat(amount)
          }, 
          token 
        }
      );
      window.open(data.signed_url, "moonpay", "width=480,height=700");
      setShowWidget(true);
    } catch (err: any) {
      alert(err.message || "Failed to open MoonPay");
    } finally {
      setLoading(false);
    }
  }, [currency, token]);

  if (!user) return null;

  return (
    <div className="flex flex-col max-w-lg mx-auto w-full">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-[-0.03em] mb-1">Deposit Funds</h1>
          <p className="text-zinc-500 text-sm">Add SOL or USDC to your TipLink wallet.</p>
        </div>
      </div>

      {/* Tab Switcher */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setTab("buy")}
          className={`flex-1 py-3 rounded-xl text-sm font-semibold transition-all flex items-center justify-center gap-2 ${tab === "buy"
            ? "bg-[#f5c518] text-black shadow-[0_0_15px_rgba(245,197,24,0.3)]"
            : "bg-white/[0.05] text-zinc-400 hover:bg-white/[0.08] border border-white/[0.06]"
            }`}
        >
          <CreditCard className="w-4 h-4" /> Buy with Card
        </button>
        <button
          onClick={() => setTab("transfer")}
          className={`flex-1 py-3 rounded-xl text-sm font-semibold transition-all flex items-center justify-center gap-2 ${tab === "transfer"
            ? "bg-[#f5c518] text-black shadow-[0_0_15px_rgba(245,197,24,0.3)]"
            : "bg-white/[0.05] text-zinc-400 hover:bg-white/[0.08] border border-white/[0.06]"
            }`}
        >
          <ArrowDownToLine className="w-4 h-4" /> Send from Wallet
        </button>
      </div>

      {/* ─── Buy with Card Tab ─── */}
      {tab === "buy" && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <Card className="overflow-hidden relative bg-[#0a0a0a]/80 border-white/[0.08] backdrop-blur-3xl p-6">
            <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-[#f5c518] to-transparent opacity-50" />

            {/* Token Selector */}
            <div className="mb-5">
              <label className="text-xs text-zinc-500 uppercase tracking-wider font-semibold mb-2 block">Token</label>
              <div className="flex gap-2">
                {[{ value: "sol", label: "SOL" }, { value: "usdc_sol", label: "USDC" }].map(({ value, label }) => (
                  <button
                    key={value}
                    onClick={() => setCurrency(value)}
                    className={`flex-1 py-2.5 rounded-xl text-sm font-medium transition-all ${currency === value
                      ? "bg-[#f5c518] text-black shadow-[0_0_15px_rgba(245,197,24,0.3)]"
                      : "bg-white/[0.05] text-zinc-400 hover:bg-white/[0.08] border border-white/[0.06]"
                      }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>

            {/* Amount Input */}
            <div className="mb-4">
              <label className="text-xs text-zinc-500 uppercase tracking-wider font-semibold mb-2 block">Amount (USD)</label>
              <div className="flex gap-2 mb-3">
                {["25", "50", "100", "250"].map((val) => (
                  <button
                    key={val}
                    onClick={() => setAmount(val)}
                    className={`flex-1 py-2.5 rounded-xl text-sm font-medium transition-all ${amount === val
                      ? "bg-[#f5c518] text-black shadow-[0_0_15px_rgba(245,197,24,0.3)]"
                      : "bg-white/[0.05] text-zinc-400 hover:bg-white/[0.08] border border-white/[0.06]"
                      }`}
                  >
                    ${val}
                  </button>
                ))}
              </div>
              <div className="flex items-center gap-2 bg-white/[0.03] border border-white/[0.06] rounded-xl px-4 py-3">
                <span className="text-zinc-500 text-lg">$</span>
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  min={limits?.baseCurrencyMinBuyAmount}
                  max={limits?.baseCurrencyMaxBuyAmount}
                  className="bg-transparent text-white text-lg w-full outline-none"
                  placeholder="Enter amount"
                />
              </div>
              {limits && (
                <p className="text-xs text-zinc-600 mt-2">
                  Min ${limits.baseCurrencyMinBuyAmount} — Max ${limits.baseCurrencyMaxBuyAmount}
                </p>
              )}
            </div>

            {/* Quote Preview */}
            {quote && (
              <div className="bg-white/[0.03] border border-white/[0.06] rounded-xl p-4 mb-5">
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-zinc-400">You receive</span>
                  <span className="text-white font-semibold">
                    {quote.quoteCurrencyAmount?.toFixed(4) ?? "0.0000"} {currency === "sol" ? "SOL" : "USDC"}
                  </span>
                </div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-zinc-400">Fee</span>
                  <span className="text-zinc-300">${quote.feeAmount?.toFixed(2) ?? "0.00"}</span>
                </div>
                <div className="h-px bg-white/[0.06] my-2" />
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-400 font-semibold">Total charged</span>
                  <span className="text-white font-bold">${quote.totalAmount?.toFixed(2) ?? "0.00"}</span>
                </div>
              </div>
            )}

            {/* Wallet preview */}
            <p className="text-xs text-zinc-600 mb-1">Funds will be sent to:</p>
            <p className="font-mono text-xs text-zinc-400 truncate mb-5">{user.public_key}</p>

            <Button
              onClick={handleBuy}
              disabled={loading || !quote}
              className="w-full py-6 rounded-2xl bg-[#f5c518] hover:bg-[#d4a813] text-black font-semibold text-base flex items-center justify-center gap-3 transition-colors shadow-[0_0_20px_rgba(245,197,24,0.3)] hover:shadow-[0_0_30px_rgba(245,197,24,0.4)] disabled:opacity-50"
            >
              {loading ? (
                <><Loader2 className="w-5 h-5 animate-spin" /> Opening MoonPay...</>
              ) : (
                <><CreditCard className="w-5 h-5" /> Buy {currency === "sol" ? "SOL" : "USDC"} with Card</>
              )}
            </Button>

            {showWidget && (
              <p className="text-xs text-zinc-500 mt-4 text-center">
                Complete your purchase in the MoonPay window. Your balance will update automatically.
              </p>
            )}

            <p className="text-xs text-zinc-600 mt-3 text-center">
              Powered by MoonPay · Sandbox Mode
            </p>
          </Card>
        </motion.div>
      )}

      {/* ─── Transfer from Wallet Tab ─── */}
      {tab === "transfer" && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
        >
          <Card className="overflow-hidden relative bg-[#0a0a0a]/80 border-white/[0.08] backdrop-blur-3xl p-8 flex flex-col items-center text-center">
            <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-[#f5c518] to-transparent opacity-50" />

            <div className="w-16 h-16 rounded-2xl bg-[#f5c518]/10 flex items-center justify-center mb-6 border border-[#f5c518]/20 shadow-[0_0_30px_rgba(245,197,24,0.15)]">
              <Wallet className="w-8 h-8 text-[#f5c518]" />
            </div>

            <h2 className="text-xl font-bold text-white mb-2">Your Solana Address</h2>
            <p className="text-sm text-zinc-400 mb-8 max-w-[280px]">
              Scan this QR code or copy the address below to deposit from any external wallet or exchange.
            </p>

            <div className="bg-white p-4 rounded-3xl shadow-xl mb-8 relative group">
              <div className="absolute -inset-0.5 bg-gradient-to-br from-[#f5c518] to-amber-600 rounded-[1.4rem] blur opacity-0 group-hover:opacity-40 transition duration-500"></div>
              <div className="relative bg-white rounded-2xl p-2">
                <QRCodeSVG
                  value={user.public_key}
                  size={180}
                  bgColor="#ffffff"
                  fgColor="#000000"
                  level="H"
                  includeMargin={false}
                />
              </div>
            </div>

            <div className="w-full bg-white/[0.03] border border-white/[0.06] rounded-2xl p-4 mb-4">
              <p className="text-xs text-zinc-500 uppercase tracking-wider font-semibold mb-2 text-left">Wallet Address</p>
              <div className="flex items-center justify-between gap-3">
                <span className="font-mono text-sm text-zinc-300 truncate select-all">{user.public_key}</span>
                <button
                  onClick={handleCopy}
                  className="shrink-0 flex items-center justify-center w-10 h-10 rounded-xl bg-white/[0.05] hover:bg-[#f5c518]/20 hover:text-[#f5c518] text-zinc-400 transition-all border border-transparent hover:border-[#f5c518]/30"
                >
                  {copied ? <CheckCircle2 className="w-5 h-5 text-emerald-400" /> : <Copy className="w-5 h-5" />}
                </button>
              </div>
            </div>

            <div className="flex items-start gap-3 w-full p-4 rounded-xl bg-amber-500/[0.05] border border-amber-500/10 text-left">
              <Info className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
              <p className="text-xs text-amber-200/70 leading-relaxed">
                <strong className="text-amber-300 font-semibold block mb-1">Network: Solana (SPL)</strong>
                Only send SOL or USDC via the Solana network. Sending on other networks will result in permanent loss.
              </p>
            </div>
          </Card>
        </motion.div>
      )}
    </div>
  );
}
