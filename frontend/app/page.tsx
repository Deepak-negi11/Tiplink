"use client";

import { motion } from "framer-motion";
import { ArrowRight, Shield, Zap, Loader2, Sparkles, Link2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { fetchApi } from "@/lib/api";
import { useAuthStore } from "@/store/useStore";

interface AuthResponse {
  token: string;
  refresh_token?: string;
  user_id: string;
  email: string;
  public_key: string;
}

export default function LandingPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLogin, setIsLogin] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const router = useRouter();
  const login = useAuthStore((s) => s.login);

  const handleAuth = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError("");
    try {
      const endpoint = isLogin ? "/auth/signin" : "/auth/signup";
      const data = await fetchApi<AuthResponse>(endpoint, {
        method: "POST",
        body: { email, password },
      });
      login(
        { id: data.user_id, email: data.email, public_key: data.public_key },
        data.token
      );
      router.push("/dashboard");
    } catch (err: any) {
      setError(err.message || "Something went wrong");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-[#e8e3d5] flex flex-col relative overflow-hidden">

      {/* ── Ambient brand orbs ── */}
      <div className="orb orb-brand absolute top-[-25%] left-[-15%] w-[55%] h-[55%] animate-float" />
      <div className="orb orb-brand-dim absolute bottom-[-20%] right-[-10%] w-[40%] h-[40%]" />

      {/* ── Subtle grid overlay ── */}
      <div
        className="absolute inset-0 pointer-events-none opacity-[0.025]"
        style={{
          backgroundImage:
            "linear-gradient(oklch(28.3% 0.141 291.089 / 0.6) 1px, transparent 1px), linear-gradient(90deg, oklch(28.3% 0.141 291.089 / 0.6) 1px, transparent 1px)",
          backgroundSize: "60px 60px",
        }}
      />

      {/* ── Navbar ── */}
      <header className="w-full flex items-center justify-between px-6 py-5 lg:px-12 z-10 border-b border-[var(--primary)]/10">
        {/* Logo */}
        <div className="flex items-center gap-2 group cursor-pointer">
          <div className="relative w-8 h-8 flex items-center justify-center">
            <div className="absolute inset-0 bg-gradient-to-tr from-[var(--primary)] to-[var(--primary-light)] rounded-full animate-pulse-slow blur-[4px] opacity-70" />
            <div className="relative w-6 h-6 rounded-full border-2 border-white/90 flex items-center justify-center">
              <div className="w-2 h-2 bg-white rounded-full animate-orbit" />
            </div>
          </div>
          <span className="font-display font-bold text-2xl tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-white via-white to-white/70 group-hover:to-[var(--primary-light)] transition-all duration-300">
            Orbit
          </span>
        </div>
        {/* Network status */}
        <div className="flex items-center gap-2.5 text-[#888880] text-sm">
          <div className="w-2 h-2 rounded-full bg-[var(--primary-light)] animate-pulse" />
          <span>Solana Devnet</span>
        </div>
      </header>

      {/* ── Main ── */}
      <main className="flex-1 flex flex-col lg:flex-row items-center justify-center px-6 py-16 lg:px-24 gap-14 lg:gap-24 z-10 w-full max-w-7xl mx-auto">

        {/* Hero left */}
        <motion.div
          initial={{ opacity: 0, y: 32 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.85, ease: [0.16, 1, 0.3, 1] }}
          className="flex-1 flex flex-col items-center lg:items-start text-center lg:text-left"
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: 0.15, duration: 0.45 }}
            className="inline-flex items-center gap-2 rounded-full border border-[var(--primary)]/20 bg-[var(--primary)]/[0.07] px-4 py-1.5 text-xs font-semibold text-[var(--primary-light)] mb-8 font-display tracking-wide uppercase"
          >
            <Sparkles className="h-3 w-3" />
            MPC Secured · Built on Solana
          </motion.div>

          {/* Headline */}
          <h1 className="font-display text-5xl sm:text-6xl lg:text-7xl font-bold tracking-[-0.04em] mb-6 leading-[1.04]">
            <span className="text-white">Send crypto</span>
            <br />
            <span className="text-brand-gradient">with just a link.</span>
          </h1>

          <p className="text-[#888880] text-lg lg:text-xl mb-10 max-w-md leading-relaxed">
            No wallets required. Generate a secure Orbit link and send SOL or USDC
            to anyone over any messenger, instantly.
          </p>

          {/* Trust badges */}
          <div className="flex flex-col sm:flex-row gap-5">
            <div className="flex items-center gap-3 text-[#888880] text-sm">
              <div className="w-9 h-9 rounded-xl border border-[var(--primary)]/20 bg-[var(--primary)]/[0.06] flex items-center justify-center shrink-0">
                <Shield className="w-4 h-4 text-[var(--primary-light)]" />
              </div>
              <div className="text-left">
                <p className="text-[#e8e3d5] font-semibold text-sm font-display">Non-Custodial</p>
                <p className="text-[#555550] text-xs">You hold your keys</p>
              </div>
            </div>
            <div className="flex items-center gap-3 text-[#888880] text-sm">
              <div className="w-9 h-9 rounded-xl border border-[var(--primary)]/20 bg-[var(--primary)]/[0.06] flex items-center justify-center shrink-0">
                <Zap className="w-4 h-4 text-[var(--primary-light)]" />
              </div>
              <div className="text-left">
                <p className="text-[#e8e3d5] font-semibold text-sm font-display">Instant Settlement</p>
                <p className="text-[#555550] text-xs">Solana speed, ~$0 fees</p>
              </div>
            </div>
            <div className="flex items-center gap-3 text-[#888880] text-sm">
              <div className="w-9 h-9 rounded-xl border border-[var(--primary)]/20 bg-[var(--primary)]/[0.06] flex items-center justify-center shrink-0">
                <Link2 className="w-4 h-4 text-[var(--primary-light)]" />
              </div>
              <div className="text-left">
                <p className="text-[#e8e3d5] font-semibold text-sm font-display">Share Anywhere</p>
                <p className="text-[#555550] text-xs">Just copy and paste</p>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Auth Card right */}
        <motion.div
          initial={{ opacity: 0, scale: 0.94, y: 24 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
          className="w-full max-w-[400px]"
        >
          <div className="relative rounded-2xl border border-[var(--primary)]/10 bg-[#111111] shadow-2xl shadow-black/60 overflow-hidden">
            {/* Brand top accent line */}
            <div className="absolute top-0 left-[10%] right-[10%] h-[1px] bg-gradient-to-r from-transparent via-[var(--primary)]/70 to-transparent" />
            {/* Subtle brand corner glow */}
            <div className="absolute top-0 right-0 w-32 h-32 bg-[var(--primary)]/[0.04] rounded-full blur-2xl pointer-events-none" />

            <div className="p-8 relative">
              <h2 className="font-display text-2xl font-bold mb-1 text-white tracking-tight">
                {isLogin ? "Welcome back" : "Create account"}
              </h2>
              <p className="text-[#555550] text-sm mb-7">
                {isLogin
                  ? "Sign in to access your wallet."
                  : "Your keys are secured by threshold MPC."}
              </p>

              {error && (
                <motion.div
                  initial={{ opacity: 0, y: -8 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="mb-5 p-3.5 rounded-xl bg-[#ff3b30]/[0.08] border border-[#ff3b30]/15 text-[#ff3b30] text-sm"
                >
                  {error}
                </motion.div>
              )}

              <form onSubmit={handleAuth} className="flex flex-col gap-4">
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display">
                    Email
                  </label>
                  <Input
                    type="email"
                    placeholder="name@example.com"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                  />
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display">
                    Password
                  </label>
                  <Input
                    type="password"
                    placeholder="••••••••"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                  />
                </div>

                <Button
                  type="submit"
                  size="lg"
                  className="w-full mt-3 animate-pulse-brand"
                  disabled={loading}
                >
                  {loading ? (
                    <Loader2 className="w-5 h-5 animate-spin" />
                  ) : (
                    <>
                      {isLogin ? "Sign In" : "Create Account"}
                      <ArrowRight className="ml-2 w-4 h-4" />
                    </>
                  )}
                </Button>
              </form>

              <p className="mt-6 text-center text-sm text-[#555550]">
                {isLogin ? "Don\u2019t have an account? " : "Already have an account? "}
                <button
                  onClick={() => { setIsLogin(!isLogin); setError(""); }}
                  className="text-[var(--primary-light)] hover:text-white font-semibold transition-colors"
                >
                  {isLogin ? "Sign up" : "Log in"}
                </button>
              </p>
            </div>
          </div>
        </motion.div>
      </main>
    </div>
  );
}
