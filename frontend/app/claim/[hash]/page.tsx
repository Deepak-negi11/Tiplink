"use client";

import { motion } from "framer-motion";
import { ArrowDownCircle, CheckCircle2, Loader2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useState, useEffect } from "react";
import { useParams, useRouter } from "next/navigation";
import { fetchApi } from "@/lib/api";
import { useAuthStore } from "@/store/useStore";

interface LinkDetails {
  id: string;
  amount: number;
  token_mint: string;
  token_symbol: string;
  status: string;
  memo?: string;
  decimals: number;
}

export default function ClaimTipLink() {
  const params = useParams();
  const router = useRouter();
  const { token } = useAuthStore();
  const hash = Array.isArray(params?.hash) ? params.hash[0] : params?.hash;

  const [linkDetails, setLinkDetails] = useState<LinkDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [claiming, setClaiming] = useState(false);
  const [claimed, setClaimed] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    async function fetchLink() {
      if (!hash) return;
      try {
        const data = await fetchApi<LinkDetails>(`/link/${hash}`);
        setLinkDetails(data);
      } catch (err: any) {
        setError(err.message || "Link not found or already claimed");
      } finally {
        setLoading(false);
      }
    }
    fetchLink();
  }, [hash]);

  const handleClaim = async () => {
    if (!hash) return;

    if (!token) {
      router.push(`/?redirect=/claim/${hash}`);
      return;
    }

    setClaiming(true);
    setError("");
    try {
      await fetchApi("/link/claim", {
        method: "POST",
        token,
        body: { claim_hash: hash },
      });
      setClaimed(true);
      setTimeout(() => {
        router.push("/dashboard");
      }, 2500);
    } catch (err: any) {
      setError(err.message || "Failed to claim link");
    } finally {
      setClaiming(false);
    }
  };

  const displayAmount = linkDetails
    ? (linkDetails.amount / 10 ** (linkDetails.decimals || 9)).toFixed(2)
    : "0.00";
  const displaySymbol = linkDetails?.token_symbol || "SOL";

  if (claimed) {
    return (
      <div className="min-h-screen bg-[#050507] text-white flex flex-col items-center justify-center p-6 relative overflow-hidden">
        <motion.div
          initial={{ scale: 0.5, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ type: "spring", stiffness: 200, damping: 20 }}
          className="flex flex-col items-center z-10"
        >
          <div className="w-24 h-24 bg-emerald-500/10 rounded-full flex items-center justify-center mb-8 border border-emerald-500/20">
            <CheckCircle2 className="w-12 h-12 text-emerald-400" />
          </div>
          <h1 className="text-4xl font-bold mb-4 text-center text-glow-green">Claim Successful!</h1>
          <p className="text-zinc-400 text-center text-lg">{displayAmount} {displaySymbol} has been added to your wallet.</p>
          <p className="text-zinc-600 text-sm mt-4">Redirecting to dashboard...</p>
        </motion.div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#050507] text-white flex flex-col items-center justify-center p-6 relative overflow-hidden">
      {/* Background */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[60%] h-[60%] bg-indigo-600/[0.06] blur-[150px] rounded-full pointer-events-none" />

      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        className="w-full max-w-md z-10"
      >
        {/* Logo */}
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-tr from-indigo-500 to-violet-500 flex items-center justify-center font-bold text-2xl shadow-xl shadow-indigo-500/30 mb-6">
            T
          </div>
          <h1 className="text-2xl font-semibold text-center">You received crypto!</h1>
          <p className="text-zinc-500 text-center mt-2 text-sm">Someone sent you a TipLink.</p>
        </div>

        {error && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-6 p-3.5 rounded-xl bg-red-500/[0.08] border border-red-500/15 text-red-400 text-sm flex items-center gap-2"
          >
            <AlertTriangle className="w-4 h-4 shrink-0" /> {error}
          </motion.div>
        )}

        <Card className="p-8 backdrop-blur-3xl relative overflow-visible group">
          {/* Glow border */}
          <div className="absolute -inset-[1px] bg-gradient-to-r from-indigo-500 to-violet-500 rounded-2xl opacity-[0.12] group-hover:opacity-[0.25] transition-opacity blur-sm pointer-events-none" />

          <div className="relative flex flex-col items-center z-10">
            {loading ? (
              <div className="py-10">
                <Loader2 className="w-8 h-8 animate-spin text-zinc-500" />
              </div>
            ) : linkDetails ? (
              <>
                <div className="text-6xl font-bold tracking-[-0.04em] text-glow mt-4 mb-3">
                  {displayAmount}
                </div>
                <div className={`px-4 py-1.5 rounded-full text-sm font-semibold border mb-8 ${
                  displaySymbol === "SOL"
                    ? "bg-violet-500/10 text-violet-400 border-violet-500/20"
                    : "bg-sky-500/10 text-sky-400 border-sky-500/20"
                }`}>
                  {displaySymbol}
                </div>

                {linkDetails.memo && (
                  <p className="text-sm text-zinc-400 mb-6 text-center italic">&ldquo;{linkDetails.memo}&rdquo;</p>
                )}

                <Button
                  size="lg"
                  className="w-full py-6 rounded-2xl text-lg gap-2"
                  onClick={handleClaim}
                  disabled={claiming || linkDetails.status === "claimed"}
                >
                  {claiming ? (
                    <Loader2 className="w-5 h-5 animate-spin" />
                  ) : linkDetails.status === "claimed" ? (
                    "Already Claimed"
                  ) : (
                    <>
                      <ArrowDownCircle className="w-5 h-5" />
                      {token ? "Claim into Wallet" : "Sign in to Claim"}
                    </>
                  )}
                </Button>

                <p className="text-[10px] text-zinc-600 mt-6 text-center leading-relaxed">
                  By claiming, you agree to generate an MPC-secured wallet
                  if you don&apos;t already have one.
                </p>
              </>
            ) : (
              <div className="py-10 text-center">
                <p className="text-zinc-500">This link is invalid or has expired.</p>
              </div>
            )}
          </div>
        </Card>
      </motion.div>
    </div>
  );
}
