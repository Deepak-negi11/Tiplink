"use client";

import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { fetchApi } from "@/lib/api";
import { BalanceEntry, useAuthStore } from "@/store/useStore";
import { motion } from "framer-motion";
import { AlertTriangle, ArrowDownUp, Check, Loader2, Settings2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

const TOKENS = [
  { symbol: "SOL", mint: "So11111111111111111111111111111111111111112", color: "violet", logoURI: "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png" },
  { symbol: "USDC", mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", color: "sky", logoURI: "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png" },
  { symbol: "WBTC", mint: "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh", color: "orange", logoURI: "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh/logo.png" },
];

interface QuoteResponse {
  quote: any;
  fee_breakdown: {
    total_input: number;
    fee_amount: number;
    swap_amount: number;
    fee_bps: number;
  };
}

function TokenButton({ symbol, color, logoURI, onClick }: { symbol: string; color: string; logoURI?: string; onClick?: () => void }) {
  const styles: Record<string, string> = {
    violet: "bg-violet-500/10 border-violet-500/20 text-violet-400 hover:bg-violet-500/15",
    sky: "bg-sky-500/10 border-sky-500/20 text-sky-400 hover:bg-sky-500/15",
  };
  return (
    <button onClick={onClick} className={`rounded-xl px-3 py-2.5 border font-semibold flex items-center gap-2 transition-colors text-sm ${styles[color] || styles.violet}`}>
      {logoURI ? (
        <img src={logoURI} alt={symbol} className="w-5 h-5 rounded-full" />
      ) : (
        <div className={`w-5 h-5 rounded-full bg-${color}-500/20 border border-${color}-500/30 flex items-center justify-center text-[10px] font-bold`}>
          {symbol.charAt(0)}
        </div>
      )}
      {symbol}
    </button>
  );
}

export default function SwapInterface() {
  const { token, balances, balancesLoaded } = useAuthStore();
  const [inputToken, setInputToken] = useState(TOKENS[0]);
  const [outputToken, setOutputToken] = useState(TOKENS[1]);
  const [inputAmount, setInputAmount] = useState("");
  const [outputAmount, setOutputAmount] = useState("");
  const [quote, setQuote] = useState<QuoteResponse | null>(null);
  const [quoteLoading, setQuoteLoading] = useState(false);
  const [swapLoading, setSwapLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [slippage, setSlippage] = useState(50);
  const [showSettings, setShowSettings] = useState(false);
  const [localBalances, setLocalBalances] = useState<BalanceEntry[]>([]);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);

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

  const inputBalance = localBalances.find((b) => b.symbol === inputToken.symbol);
  const outputBalance = localBalances.find((b) => b.symbol === outputToken.symbol);
  const inputHuman = inputBalance ? (inputBalance.available / 10 ** inputBalance.decimals).toFixed(2) : "0.00";
  const outputHuman = outputBalance ? (outputBalance.available / 10 ** outputBalance.decimals).toFixed(2) : "0.00";

  const fetchQuote = useCallback(async (amount: string) => {
    if (!amount || parseFloat(amount) <= 0 || !token) {
      setOutputAmount("");
      setQuote(null);
      return;
    }
    setQuoteLoading(true);
    setError("");
    try {
      const decimals = inputToken.symbol === "SOL" ? 9 : 6;
      const rawAmount = Math.floor(parseFloat(amount) * 10 ** decimals);
      const data = await fetchApi<QuoteResponse>("/swap/quote", {
        method: "POST",
        token,
        body: {
          input_mint: inputToken.mint,
          output_mint: outputToken.mint,
          amount: rawAmount,
          slippage_bps: slippage,
        },
      });
      setQuote(data);
      const outDecimals = outputToken.symbol === "SOL" ? 9 : 6;
      const outAmountRaw = (data as any).quote?.outAmount || "0";
      setOutputAmount((parseInt(outAmountRaw) / 10 ** outDecimals).toFixed(outDecimals > 6 ? 4 : 2));
    } catch (err: any) {
      setError(err.message || "Failed to fetch quote");
      setOutputAmount("");
      setQuote(null);
    } finally {
      setQuoteLoading(false);
    }
  }, [token, inputToken, outputToken, slippage]);

  const handleInputChange = (val: string) => {
    setInputAmount(val);
    setSuccess("");
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => fetchQuote(val), 500);
  };

  const handleFlip = () => {
    const temp = inputToken;
    setInputToken(outputToken);
    setOutputToken(temp);
    setInputAmount("");
    setOutputAmount("");
    setQuote(null);
    setSuccess("");
  };

  const handleSwap = async () => {
    if (!inputAmount || !quote || !token) return;
    setSwapLoading(true);
    setError("");
    setSuccess("");
    try {
      const decimals = inputToken.symbol === "SOL" ? 9 : 6;
      const { nonce } = await fetchApi<{ nonce: string }>("/swap/execute", {
        method: "POST",
        token,
        body: {
          quote: (quote as any).quote,
        },
      });

      await fetchApi("/swap/submit", {
        method: "POST",
        token,
        body: {
          nonce,
          signed_tx: [], // Backend now handles MPC signing internally for swaps
        },
      });

      setSuccess(`Swapped ${inputAmount} ${inputToken.symbol} → ${outputAmount} ${outputToken.symbol}`);
      setInputAmount("");
      setOutputAmount("");
      setQuote(null);
    } catch (err: any) {
      setError(err.message || "Swap failed");
    } finally {
      setSwapLoading(false);
    }
  };

  return (
    <div className="flex flex-col max-w-md mx-auto">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-[-0.03em] mb-1">Swap</h1>
          <p className="text-zinc-500 text-sm">Instant Jupiter-powered liquidity.</p>
        </div>
        <Button size="icon" variant="ghost" className="rounded-full" onClick={() => setShowSettings(!showSettings)}>
          <Settings2 className="w-5 h-5 text-zinc-400" />
        </Button>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} className="mb-4">
          <Card className="p-4">
            <p className="text-xs text-zinc-500 uppercase tracking-wider mb-3">Max Slippage</p>
            <div className="flex gap-2">
              {[25, 50, 100, 200].map((bps) => (
                <button
                  key={bps}
                  onClick={() => setSlippage(bps)}
                  className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                    slippage === bps
                      ? "bg-indigo-500/20 text-indigo-300 border border-indigo-500/30"
                      : "bg-white/[0.03] text-zinc-400 border border-white/[0.06] hover:bg-white/[0.06]"
                  }`}
                >
                  {bps / 100}%
                </button>
              ))}
            </div>
          </Card>
        </motion.div>
      )}

      {error && (
        <motion.div initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} className="mb-4 p-3 rounded-xl bg-red-500/[0.08] border border-red-500/15 text-red-400 text-sm flex items-center gap-2">
          <AlertTriangle className="w-4 h-4 shrink-0" /> {error}
        </motion.div>
      )}
      {success && (
        <motion.div initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} className="mb-4 p-3 rounded-xl bg-emerald-500/[0.08] border border-emerald-500/15 text-emerald-400 text-sm flex items-center gap-2">
          <Check className="w-4 h-4 shrink-0" /> {success}
        </motion.div>
      )}

      <Card className="p-2 sm:p-4 backdrop-blur-3xl relative overflow-hidden">
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-indigo-500/20 to-transparent" />

        {/* Input Token */}
        <div className="bg-white/[0.02] rounded-2xl p-4 sm:p-5 border border-white/[0.04]">
          <div className="flex justify-between items-center mb-2">
            <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider">You Pay</label>
            <button
              onClick={() => { setInputAmount(inputHuman); handleInputChange(inputHuman); }}
              className="text-xs text-zinc-500 hover:text-indigo-400 transition-colors"
            >
              Balance: {inputHuman}
            </button>
          </div>
          <div className="flex items-center justify-between">
            <input
              type="text"
              placeholder="0.00"
              value={inputAmount}
              onChange={(e) => handleInputChange(e.target.value)}
              className="bg-transparent text-3xl sm:text-4xl font-bold text-white outline-none w-[60%] placeholder:text-zinc-800"
            />
            <TokenButton symbol={inputToken.symbol} color={inputToken.color} logoURI={(inputToken as any).logoURI} />
          </div>
        </div>

        {/* Swap Icon */}
        <div className="flex items-center justify-center -my-3 relative z-10">
          <button
            onClick={handleFlip}
            className="w-10 h-10 sm:w-12 sm:h-12 bg-zinc-900 border-[3px] border-[#050507] rounded-xl flex items-center justify-center hover:bg-zinc-800 transition-all text-white hover:rotate-180 duration-400"
          >
            <ArrowDownUp className="w-4 h-4 sm:w-5 sm:h-5" />
          </button>
        </div>

        {/* Output Token */}
        <div className="bg-white/[0.02] rounded-2xl p-4 sm:p-5 border border-white/[0.04]">
          <div className="flex justify-between items-center mb-2">
            <label className="text-xs font-medium text-zinc-500 uppercase tracking-wider">You Receive</label>
            <span className="text-xs text-zinc-600">Balance: {outputHuman}</span>
          </div>
          <div className="flex items-center justify-between">
            <div className="text-3xl sm:text-4xl font-bold w-[60%]">
              {quoteLoading ? (
                <Loader2 className="w-6 h-6 animate-spin text-zinc-600" />
              ) : (
                <span className={outputAmount ? "text-white" : "text-zinc-800"}>{outputAmount || "0.00"}</span>
              )}
            </div>
            <TokenButton symbol={outputToken.symbol} color={outputToken.color} logoURI={(outputToken as any).logoURI} />
          </div>
        </div>

        {/* Quote details */}
        {quote && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="mt-3 px-2 flex flex-col gap-1">
            <div className="flex justify-between text-xs">
              <span className="text-zinc-500">Price Impact</span>
              <span className={`font-mono ${parseFloat(quote.quote?.priceImpactPct || "0") > 1 ? "text-amber-400" : "text-zinc-400"}`}>
                {parseFloat(quote.quote?.priceImpactPct || "0").toFixed(4)}%
              </span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-zinc-500">Network Fee</span>
              <span className="text-zinc-400 font-mono">~0.000005 SOL</span>
            </div>
          </motion.div>
        )}

        <Button
          size="lg"
          className="w-full mt-4 py-6 rounded-2xl"
          disabled={!quote || swapLoading || !inputAmount}
          onClick={handleSwap}
        >
          {swapLoading ? (
            <Loader2 className="w-5 h-5 animate-spin" />
          ) : !inputAmount ? (
            "Enter an amount"
          ) : !quote ? (
            "Loading quote..."
          ) : (
            "Swap"
          )}
        </Button>
      </Card>

      <div className="flex items-center justify-between mt-4 px-2 text-xs font-medium text-zinc-600">
        <span>Powered by Jupiter</span>
        <span>Max slippage: {slippage / 100}%</span>
      </div>
    </div>
  );
}
