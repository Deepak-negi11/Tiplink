"use client";

import { useAuthStore } from "@/store/useStore";
import { QRCodeSVG } from "qrcode.react";
import { motion } from "framer-motion";
import { Copy, CheckCircle2, Wallet, Info, CreditCard, Loader2 } from "lucide-react";
import { useState } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function DepositPage() {
  const { user } = useAuthStore();
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(false);
  const [amount, setAmount] = useState("10");

  const handleCopy = async () => {
    if (!user?.public_key) return;
    await navigator.clipboard.writeText(user.public_key);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleStripeCheckout = async () => {
    if (!user?.public_key) return;
    setLoading(true);
    try {
      const res = await fetch("/api/stripe/checkout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          amount: parseFloat(amount) || 10,
          currency: "usd",
          walletAddress: user.public_key,
          userId: user.user_id,
        }),
      });
      const data = await res.json();
      if (data.url) {
        window.location.href = data.url;
      } else {
        alert(data.error || "Failed to create checkout session");
      }
    } catch (err) {
      alert("Network error — please try again");
    } finally {
      setLoading(false);
    }
  };

  if (!user) return null;

  return (
    <div className="flex flex-col max-w-lg mx-auto w-full">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-[-0.03em] mb-1">Deposit Funds</h1>
          <p className="text-zinc-500 text-sm">Send SOL or USDC to your TipLink wallet.</p>
        </div>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, ease: "easeOut" }}
      >
        <Card className="overflow-hidden relative bg-[#0a0a0a]/80 border-white/[0.08] backdrop-blur-3xl p-8 flex flex-col items-center text-center">
          <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-[#f5c518] to-transparent opacity-50" />
          
          <div className="w-16 h-16 rounded-2xl bg-[#f5c518]/10 flex items-center justify-center mb-6 border border-[#f5c518]/20 shadow-[0_0_30px_rgba(245,197,24,0.15)]">
            <Wallet className="w-8 h-8 text-[#f5c518]" />
          </div>

          <h2 className="text-xl font-bold text-white mb-2">Your Solana Address</h2>
          <p className="text-sm text-zinc-400 mb-8 max-w-[280px]">
            Scan this QR code or copy the address below to deposit funds from any external wallet or exchange.
          </p>

          <div className="bg-white p-4 rounded-3xl shadow-xl mb-8 relative group">
            <div className="absolute -inset-0.5 bg-gradient-to-br from-[#f5c518] to-amber-600 rounded-[1.4rem] blur opacity-0 group-hover:opacity-40 transition duration-500"></div>
            <div className="relative bg-white rounded-2xl p-2">
              <QRCodeSVG
                value={user.public_key}
                size={180}
                bgColor={"#ffffff"}
                fgColor={"#000000"}
                level={"H"}
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

          <div className="flex items-start gap-3 w-full p-4 rounded-xl bg-blue-500/[0.05] border border-blue-500/10 text-left">
            <Info className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
            <p className="text-xs text-blue-200/70 leading-relaxed">
              <strong className="text-blue-300 font-semibold block mb-1">Network: Solana (SPL)</strong>
              Only send Solana native tokens (SOL, USDC) to this address via the Solana network.
            </p>
          </div>
        </Card>
      </motion.div>

      <div className="mt-6 flex flex-col items-center">
        <div className="flex items-center gap-4 w-full my-6">
          <div className="h-px bg-white/[0.06] flex-1" />
          <span className="text-xs text-zinc-500 font-medium uppercase tracking-wider">Or buy with card</span>
          <div className="h-px bg-white/[0.06] flex-1" />
        </div>

        <Card className="w-full p-6 bg-[#0a0a0a]/80 border-white/[0.08]">
          <div className="flex items-center gap-3 mb-4">
            <CreditCard className="w-5 h-5 text-[#f5c518]" />
            <h3 className="font-semibold text-white">Buy with Stripe</h3>
          </div>
          
          <p className="text-sm text-zinc-400 mb-5">
            Purchase SOL instantly using your credit or debit card via Stripe Checkout.
          </p>

          <div className="mb-5">
            <label className="text-xs text-zinc-500 uppercase tracking-wider font-semibold mb-2 block">Amount (USD)</label>
            <div className="flex gap-2">
              {["5", "10", "25", "50"].map((val) => (
                <button
                  key={val}
                  onClick={() => setAmount(val)}
                  className={`flex-1 py-2.5 rounded-xl text-sm font-medium transition-all ${
                    amount === val
                      ? "bg-[#f5c518] text-black shadow-[0_0_15px_rgba(245,197,24,0.3)]"
                      : "bg-white/[0.05] text-zinc-400 hover:bg-white/[0.08] border border-white/[0.06]"
                  }`}
                >
                  ${val}
                </button>
              ))}
            </div>
          </div>

          <Button
            onClick={handleStripeCheckout}
            disabled={loading}
            className="w-full py-6 rounded-2xl bg-[#635BFF] hover:bg-[#5046e4] text-white font-semibold text-base flex items-center justify-center gap-3 transition-colors shadow-[0_0_20px_rgba(99,91,255,0.3)] hover:shadow-[0_0_30px_rgba(99,91,255,0.4)] disabled:opacity-50"
          >
            {loading ? (
              <><Loader2 className="w-5 h-5 animate-spin" /> Processing...</>
            ) : (
              <><CreditCard className="w-5 h-5" /> Pay ${amount} with Stripe</>
            )}
          </Button>

          <p className="text-xs text-zinc-600 mt-3 text-center">
            Powered by Stripe · Test Mode
          </p>
        </Card>
      </div>
    </div>
  );
}
