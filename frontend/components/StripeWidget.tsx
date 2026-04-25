"use client";

import { useEffect, useState, useRef } from "react";
import { X, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { loadStripeOnramp } from "@stripe/crypto";

interface StripeWidgetProps {
  walletAddress: string;
  onClose: () => void;
}

export function StripeWidget({ walletAddress, onClose }: StripeWidgetProps) {
  const [clientSecret, setClientSecret] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const onrampRef = useRef<any>(null);

  useEffect(() => {
    const fetchSession = async () => {
      try {
        const res = await fetch("/api/stripe/onramp", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ walletAddress }),
        });
        const data = await res.json();
        
        if (!res.ok) throw new Error(data.error || "Failed to initialize Stripe");
        setClientSecret(data.clientSecret);
      } catch (err: any) {
        setError(err.message);
      }
    };

    fetchSession();
  }, [walletAddress]);

  useEffect(() => {
    if (!clientSecret || !containerRef.current) return;

    let mounted = true;

    const initializeStripe = async () => {
      try {
        const publishableKey = process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY || "pk_test_TYooMQauvdEDq54NiTphI7jx";
        
        const stripeOnramp = await loadStripeOnramp(publishableKey);
        if (!stripeOnramp || !mounted) return;

        const onrampSession = stripeOnramp.createSession({
          clientSecret,
          appearance: {
            theme: "dark",
            variables: {
              colorPrimary: "#f5c518",
              colorBackground: "#0d0d0d",
              colorText: "#ffffff",
              colorDanger: "#ff3b30",
              fontFamily: "Inter, sans-serif",
              borderRadius: "12px",
            },
          },
        });

        if (containerRef.current) {
          onrampSession.mount(containerRef.current);
          onrampRef.current = onrampSession;
        }
      } catch (err: any) {
        if (mounted) setError(err.message);
      }
    };

    initializeStripe();

    return () => {
      mounted = false;
      if (onrampRef.current && containerRef.current) {
        try {
          containerRef.current.innerHTML = "";
        } catch (e) {}
      }
    };
  }, [clientSecret]);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-black/80 backdrop-blur-md flex items-center justify-center z-[100] p-4"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0, y: 20 }}
        animate={{ scale: 1, opacity: 1, y: 0 }}
        exit={{ scale: 0.95, opacity: 0, y: 20 }}
        className="relative bg-[#0d0d0d] rounded-2xl overflow-hidden shadow-[0_0_50px_rgba(0,0,0,0.5)] border border-white/[0.1] w-full max-w-[450px] flex flex-col min-h-[600px]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-white/[0.08] bg-[#111111]">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-white font-display">Buy with Stripe</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400">Crypto</span>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-full bg-white/[0.05] hover:bg-white/[0.1] text-zinc-400 hover:text-white transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="flex-1 w-full bg-[#0d0d0d] relative flex items-center justify-center">
          {error ? (
            <div className="p-6 text-center flex flex-col items-center">
              <p className="text-red-400 mb-2 font-medium">Initialization Error</p>
              <p className="text-sm text-zinc-500">{error}</p>
              <p className="text-xs text-zinc-600 mt-4 max-w-[250px]">Ensure your Stripe Secret Key is set in your .env file.</p>
            </div>
          ) : !clientSecret ? (
            <div className="flex flex-col items-center gap-3">
              <Loader2 className="w-6 h-6 text-[#f5c518] animate-spin" />
              <p className="text-sm text-zinc-400">Connecting to Stripe...</p>
            </div>
          ) : (
            <div ref={containerRef} className="w-full h-[600px] overflow-hidden" />
          )}
        </div>
      </motion.div>
    </motion.div>
  );
}
