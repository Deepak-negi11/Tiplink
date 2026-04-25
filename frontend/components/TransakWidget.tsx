"use client";

import { X, ExternalLink, CreditCard } from "lucide-react";
import { motion } from "framer-motion";
import { useEffect, useState } from "react";

interface TransakProps {
  walletAddress: string;
  onClose: () => void;
}

export function TransakWidget({ walletAddress, onClose }: TransakProps) {
  const [opened, setOpened] = useState(false);

  const apiKey = process.env.NEXT_PUBLIC_TRANSAK_API_KEY || "8b8433ec-58a4-472d-88b9-fb7578351509";
  
  const params = new URLSearchParams({
    apiKey: apiKey,
    network: "solana",
    cryptoCurrencyList: "SOL,USDC",
    defaultCryptoCurrency: "SOL",
    walletAddress: walletAddress,
    disableWalletAddressForm: "true",
    fiatCurrency: "USD",
    defaultFiatAmount: "50",
    themeColor: "f5c518",
    colorMode: "DARK",
  });

  const transakUrl = `https://global-stg.transak.com?${params.toString()}`;

  const handleOpen = () => {
    window.open(transakUrl, "_blank", "width=450,height=700,noopener,noreferrer");
    setOpened(true);
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-black/90 backdrop-blur-md flex items-center justify-center z-[100] p-4"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0, y: 20 }}
        animate={{ scale: 1, opacity: 1, y: 0 }}
        exit={{ scale: 0.95, opacity: 0, y: 20 }}
        className="relative bg-[#111] rounded-2xl overflow-hidden shadow-[0_0_80px_rgba(0,0,0,0.8)] border border-white/[0.1] w-full max-w-[420px] flex flex-col p-8"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          onClick={onClose}
          className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded-full bg-white/[0.05] hover:bg-white/[0.1] text-zinc-400 hover:text-white transition-colors"
        >
          <X className="w-4 h-4" />
        </button>

        <div className="flex flex-col items-center text-center">
          <div className="w-16 h-16 rounded-2xl bg-[#f5c518]/10 flex items-center justify-center mb-6 border border-[#f5c518]/20">
            <CreditCard className="w-8 h-8 text-[#f5c518]" />
          </div>

          <h3 className="text-xl font-bold text-white mb-2">Buy Crypto with Card</h3>
          <p className="text-sm text-zinc-400 mb-6 leading-relaxed">
            Purchase SOL or USDC instantly using your credit card, debit card, or bank transfer via Transak.
          </p>

          {!opened ? (
            <button
              onClick={handleOpen}
              className="w-full py-4 rounded-xl bg-[#f5c518] hover:bg-[#d4a810] text-black font-semibold text-base flex items-center justify-center gap-2 transition-colors shadow-[0_0_20px_rgba(245,197,24,0.3)]"
            >
              <ExternalLink className="w-4 h-4" />
              Open Transak
            </button>
          ) : (
            <div className="w-full">
              <div className="w-full py-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 font-medium text-sm flex items-center justify-center gap-2 mb-4">
                ✓ Transak opened in new window
              </div>
              <p className="text-xs text-zinc-500 mb-4">
                Complete your purchase in the Transak window. Your balance will update automatically once the transaction is confirmed.
              </p>
              <button
                onClick={handleOpen}
                className="w-full py-3 rounded-xl bg-white/[0.05] hover:bg-white/[0.08] text-zinc-300 font-medium text-sm flex items-center justify-center gap-2 transition-colors border border-white/[0.08]"
              >
                <ExternalLink className="w-3 h-3" />
                Reopen Transak
              </button>
            </div>
          )}

          <p className="text-xs text-zinc-600 mt-6">
            Powered by Transak · Staging Mode
          </p>
        </div>
      </motion.div>
    </motion.div>
  );
}
