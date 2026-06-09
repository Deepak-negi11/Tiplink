"use client";

import { motion } from "framer-motion";
import {
  ArrowRight, Shield, Zap, Loader2, Sparkles, Link2,
  Globe, Lock, Rocket, Send, CreditCard, RefreshCcw,
  ChevronRight, CheckCircle2, XCircle, Mail, MapPin, MessageSquare,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { fetchApi } from "@/lib/api";
import { useAuthStore } from "@/store/useStore";
import Navbar from "@/components/Navbar";

interface AuthResponse {
  token: string;
  refresh_token?: string;
  user_id: string;
  email: string;
  public_key: string;
}

/* ── Flowing wave SVG lines for hero background ── */
function WaveLines() {
  return (
    <div className="absolute inset-0 overflow-hidden pointer-events-none opacity-20">
      <div className="animate-wave-flow" style={{ width: "200%" }}>
        <svg
          viewBox="0 0 2400 400"
          className="w-full h-full"
          preserveAspectRatio="none"
        >
          {[...Array(8)].map((_, i) => (
            <path
              key={i}
              d={`M0,${180 + i * 25} C400,${150 + i * 20 + Math.sin(i) * 30} 800,${220 + i * 15} 1200,${180 + i * 25} S2000,${150 + i * 20} 2400,${180 + i * 25}`}
              fill="none"
              stroke="#EA3A59"
              strokeWidth={0.5 + (Math.sin(i * 123) + 1) * 0.25}
              opacity={0.3 + (Math.cos(i * 456) + 1) * 0.2}
            />
          ))}
        </svg>
      </div>
    </div>
  );
}

/* ── Feature card ── */
function FeatureCard({
  icon: Icon,
  title,
  desc,
  delay,
}: {
  icon: any;
  title: string;
  desc: string;
  delay: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-50px" }}
      transition={{ duration: 0.6, delay, ease: [0.16, 1, 0.3, 1] }}
      className="group relative bg-[#0a0a0a] border border-[#EA3A59]/10 rounded-2xl p-7 hover:border-[#EA3A59]/30 transition-all duration-300 hover:shadow-xl hover:shadow-[#EA3A59]/5"
    >
      <div className="absolute top-0 left-[10%] right-[10%] h-[1px] bg-gradient-to-r from-transparent via-[#EA3A59]/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
      <div className="w-12 h-12 rounded-xl bg-[#EA3A59]/10 border border-[#EA3A59]/15 flex items-center justify-center mb-5 group-hover:bg-[#EA3A59]/15 transition-colors">
        <Icon className="w-5 h-5 text-[#EA3A59]" />
      </div>
      <h3 className="font-display font-bold text-lg text-white mb-2 tracking-tight">
        {title}
      </h3>
      <p className="text-[#888880] text-sm leading-relaxed">{desc}</p>
    </motion.div>
  );
}

/* ── Compare table row ── */
function CompareRow({
  feature,
  orbit,
  traditional,
  other,
}: {
  feature: string;
  orbit: boolean;
  traditional: boolean;
  other: boolean;
}) {
  const Icon = ({ yes }: { yes: boolean }) =>
    yes ? (
      <CheckCircle2 className="w-5 h-5 text-[#00d26a]" />
    ) : (
      <XCircle className="w-5 h-5 text-[#ff3b30]/60" />
    );

  return (
    <div className="grid grid-cols-4 gap-4 py-4 border-b border-white/[0.04] last:border-0 items-center">
      <span className="text-sm text-[#e8e3d5] font-medium">{feature}</span>
      <div className="flex justify-center">
        <Icon yes={orbit} />
      </div>
      <div className="flex justify-center">
        <Icon yes={traditional} />
      </div>
      <div className="flex justify-center">
        <Icon yes={other} />
      </div>
    </div>
  );
}

export default function LandingPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLogin, setIsLogin] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [contactName, setContactName] = useState("");
  const [contactEmail, setContactEmail] = useState("");
  const [contactMessage, setContactMessage] = useState("");
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

  const handleContactSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const subject = encodeURIComponent(`Orbit enquiry from ${contactName}`);
    const body = encodeURIComponent(
      `Name: ${contactName}\nEmail: ${contactEmail}\n\n${contactMessage}`
    );

    const gmailComposeUrl =
      `https://mail.google.com/mail/?view=cm&fs=1&to=deepaknegi108r@gmail.com` +
      `&su=${subject}&body=${body}`;

    window.open(gmailComposeUrl, "_blank", "noopener,noreferrer");
  };

  return (
    <div className="min-h-screen bg-black text-[#e8e3d5] flex flex-col relative overflow-hidden">
      {/* Navbar */}
      <Navbar />

      {/* ══════════════════════════════════════
          HERO SECTION
         ══════════════════════════════════════ */}
      <section
        id="home"
        className="relative min-h-[720px] lg:min-h-[760px] flex flex-col items-center justify-center px-6 pt-28 pb-16"
      >
        {/* Ambient orbs */}
        <div className="orb orb-brand absolute top-[-25%] left-[-15%] w-[55%] h-[55%] animate-float" />
        <div className="orb orb-brand-dim absolute bottom-[-20%] right-[-10%] w-[40%] h-[40%]" />

        {/* Flowing wave lines */}
        <WaveLines />

        {/* Grid overlay */}
        <div
          className="absolute inset-0 pointer-events-none opacity-[0.02]"
          style={{
            backgroundImage:
              "linear-gradient(rgba(234,58,89,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(234,58,89,0.5) 1px, transparent 1px)",
            backgroundSize: "60px 60px",
          }}
        />

        <div className="relative z-10 text-center max-w-5xl mx-auto">
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: 0.1, duration: 0.5 }}
            className="inline-flex items-center gap-2 rounded-full border border-[#EA3A59]/20 bg-[#EA3A59]/[0.07] px-4 py-1.5 text-xs font-semibold text-[#ff6b84] mb-7 font-display uppercase"
          >
            <Sparkles className="h-3 w-3" />
            Secure · Non-custodial · Built on Solana
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.9, ease: [0.16, 1, 0.3, 1] }}
            className="font-display text-5xl sm:text-6xl lg:text-7xl font-bold mb-6 leading-[0.94] text-balance"
          >
            <span className="text-white block">SEND CRYPTO.</span>
            <span className="text-brand-gradient block mt-2">SHARE A LINK.</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.7 }}
            className="text-[#888880] text-base sm:text-lg mb-8 max-w-xl mx-auto leading-relaxed text-pretty"
          >
            Create a secure link, share it anywhere, and let anyone claim SOL
            or USDC. No wallet setup required.
          </motion.p>

          {/* CTA buttons */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.6 }}
            className="flex flex-col sm:flex-row gap-4 justify-center"
          >
            <button
              onClick={() => {
                const el = document.querySelector("#signup");
                if (el) el.scrollIntoView({ behavior: "smooth" });
              }}
              className="px-8 py-4 bg-[#EA3A59] text-white font-bold rounded-2xl hover:bg-[#ff6b84] transition-all duration-200 shadow-xl shadow-[#EA3A59]/25 hover:shadow-[#EA3A59]/40 active:scale-[0.97] text-base font-display flex items-center justify-center gap-2"
            >
              Get Started <ArrowRight className="w-4 h-4" />
            </button>
            <button
              onClick={() => {
                const el = document.querySelector("#features");
                if (el) el.scrollIntoView({ behavior: "smooth" });
              }}
              className="px-8 py-4 border border-white/10 text-white font-bold rounded-2xl hover:bg-white/[0.04] hover:border-white/20 transition-all duration-200 text-base font-display"
            >
              Learn More
            </button>
          </motion.div>

          {/* Trust badges */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7, duration: 0.6 }}
            className="flex flex-col sm:flex-row gap-5 justify-center mt-12"
          >
            {[
              { icon: Shield, label: "Non-Custodial", sub: "You hold your keys" },
              { icon: Zap, label: "Instant Settlement", sub: "Solana speed, ~$0 fees" },
              { icon: Link2, label: "Share Anywhere", sub: "Just copy and paste" },
            ].map((item, i) => (
              <div key={i} className="flex items-center gap-3 text-sm">
                <div className="size-9 rounded-xl border border-[#EA3A59]/20 bg-[#EA3A59]/[0.06] flex items-center justify-center shrink-0">
                  <item.icon className="w-4 h-4 text-[#EA3A59]" />
                </div>
                <div className="text-left">
                  <p className="text-white font-semibold text-sm font-display">{item.label}</p>
                  <p className="text-[#555550] text-xs">{item.sub}</p>
                </div>
              </div>
            ))}
          </motion.div>
        </div>
      </section>

      {/* ══════════════════════════════════════
          FEATURES SECTION
         ══════════════════════════════════════ */}
      <section id="features" className="relative py-24 px-6 lg:px-12">
        <div className="max-w-6xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <span className="inline-flex items-center gap-2 rounded-full border border-[#EA3A59]/20 bg-[#EA3A59]/[0.07] px-4 py-1.5 text-xs font-semibold text-[#ff6b84] mb-6 font-display tracking-wide uppercase">
              Features
            </span>
            <h2 className="font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em] text-white mb-4">
              Everything you need
            </h2>
            <p className="text-[#888880] text-lg max-w-md mx-auto">
              A complete crypto wallet experience — secure, fast, and
              beautifully simple.
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            <FeatureCard
              icon={Lock}
              title="MPC Security"
              desc="Your keys are split across multiple servers using FROST threshold signatures. No single point of failure."
              delay={0}
            />
            <FeatureCard
              icon={Send}
              title="Send via Link"
              desc="Generate a shareable Orbit link. Recipient doesn't even need an account — just click and claim."
              delay={0.1}
            />
            <FeatureCard
              icon={CreditCard}
              title="Buy with Card"
              desc="Purchase SOL or USDC directly with your credit card powered by MoonPay. No crypto experience needed."
              delay={0.2}
            />
            <FeatureCard
              icon={RefreshCcw}
              title="Instant Swaps"
              desc="Swap between tokens in seconds with Jupiter-powered liquidity and tight spreads."
              delay={0.1}
            />
            <FeatureCard
              icon={Zap}
              title="Gasless Claiming"
              desc="Recipients don't need a wallet or SOL to get started. Claiming is completely free and sponsored by Orbit."
              delay={0.2}
            />
            <FeatureCard
              icon={Rocket}
              title="No Seed Phrases"
              desc="Forget 24-word backups forever. Your email + password is all you need — MPC handles the rest."
              delay={0.3}
            />
          </div>
        </div>
      </section>

      {/* ══════════════════════════════════════
          COMPARE SECTION
         ══════════════════════════════════════ */}
      <section id="compare" className="relative py-24 px-6 lg:px-12">
        <div className="max-w-4xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <span className="inline-flex items-center gap-2 rounded-full border border-[#EA3A59]/20 bg-[#EA3A59]/[0.07] px-4 py-1.5 text-xs font-semibold text-[#ff6b84] mb-6 font-display tracking-wide uppercase">
              Compare
            </span>
            <h2 className="font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em] text-white mb-4">
              Why Orbit?
            </h2>
            <p className="text-[#888880] text-lg max-w-md mx-auto">
              See how we stack up against traditional wallets and other solutions.
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="bg-[#0a0a0a] border border-[#EA3A59]/10 rounded-2xl p-6 sm:p-8 overflow-x-auto"
          >
            {/* Header */}
            <div className="grid grid-cols-4 gap-4 pb-4 border-b border-white/[0.08] mb-2">
              <span className="text-sm font-bold text-[#888880] font-display">Feature</span>
              <div className="text-center">
                <span className="text-sm font-bold text-[#EA3A59] font-display">Orbit</span>
              </div>
              <div className="text-center">
                <span className="text-sm font-bold text-[#888880] font-display">Phantom / Solflare</span>
              </div>
              <div className="text-center">
                <span className="text-sm font-bold text-[#888880] font-display">TipLink</span>
              </div>
            </div>

            <CompareRow feature="No Seed Phrase" orbit={true} traditional={false} other={true} />
            <CompareRow feature="Email Login" orbit={true} traditional={false} other={true} />
            <CompareRow feature="Send via Link" orbit={true} traditional={false} other={true} />
            <CompareRow feature="MPC Key Security" orbit={true} traditional={false} other={false} />
            <CompareRow feature="Built-in Swaps" orbit={true} traditional={true} other={false} />
            <CompareRow feature="Buy with Card" orbit={true} traditional={true} other={false} />
            <CompareRow feature="Non-Custodial" orbit={true} traditional={true} other={false} />
            <CompareRow feature="Open Source" orbit={true} traditional={false} other={false} />
          </motion.div>
        </div>
      </section>

      {/* ══════════════════════════════════════
          CONTACT SECTION
         ══════════════════════════════════════ */}
      <section id="contact" className="relative py-24 px-6 lg:px-12">
        <div className="max-w-4xl mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <span className="inline-flex items-center gap-2 rounded-full border border-[#EA3A59]/20 bg-[#EA3A59]/[0.07] px-4 py-1.5 text-xs font-semibold text-[#ff6b84] mb-6 font-display tracking-wide uppercase">
              Contact
            </span>
            <h2 className="font-display text-4xl sm:text-5xl font-bold text-white mb-4 text-balance">
              Contact Orbit
            </h2>
            <p className="text-[#888880] text-lg max-w-lg mx-auto text-pretty">
              For product questions, feedback, or partnership enquiries,
              contact Deepak directly.
            </p>
          </motion.div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Contact info */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="flex flex-col gap-6"
            >
              {[
                { icon: Mail, label: "Email", value: "deepaknegi108r@gmail.com", href: "mailto:deepaknegi108r@gmail.com" },
                { icon: MapPin, label: "Based in", value: "Decentralized — Everywhere" },
                { icon: MessageSquare, label: "Response time", value: "Usually within 24 hours" },
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-4 p-5 bg-[#0a0a0a] border border-[#EA3A59]/10 rounded-2xl hover:border-[#EA3A59]/20 transition-colors">
                  <div className="size-10 rounded-xl bg-[#EA3A59]/10 border border-[#EA3A59]/15 flex items-center justify-center shrink-0">
                    <item.icon className="w-4 h-4 text-[#EA3A59]" />
                  </div>
                  <div>
                    <p className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display mb-1">{item.label}</p>
                    {item.href ? (
                      <a className="text-white text-sm font-medium break-all hover:text-[#ff6b84] transition-colors" href={item.href}>
                        {item.value}
                      </a>
                    ) : (
                      <p className="text-white text-sm font-medium">{item.value}</p>
                    )}
                  </div>
                </div>
              ))}
            </motion.div>

            {/* Contact form */}
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
            >
              <form
                onSubmit={handleContactSubmit}
                className="bg-[#0a0a0a] border border-[#EA3A59]/10 rounded-2xl p-6 flex flex-col gap-4"
              >
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display">
                    Name
                  </label>
                  <Input
                    placeholder="Your name"
                    value={contactName}
                    onChange={(e) => setContactName(e.target.value)}
                    required
                  />
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display">
                    Email
                  </label>
                  <Input
                    type="email"
                    placeholder="you@example.com"
                    value={contactEmail}
                    onChange={(e) => setContactEmail(e.target.value)}
                    required
                  />
                </div>
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-semibold text-[#555550] uppercase tracking-wider font-display">
                    Message
                  </label>
                  <textarea
                    placeholder="Tell us what's on your mind..."
                    value={contactMessage}
                    onChange={(e) => setContactMessage(e.target.value)}
                    rows={4}
                    required
                    className="flex w-full rounded-xl border border-white/[0.07] bg-[#111111] px-4 py-3 text-sm text-[#e8e3d5] placeholder:text-[#555550] focus-visible:outline-none focus-visible:border-[#EA3A59]/45 focus-visible:ring-[3px] focus-visible:ring-[#EA3A59]/8 transition-all duration-200 resize-none"
                  />
                </div>
                <Button type="submit" size="lg" className="w-full mt-2">
                  Open Gmail draft <ArrowRight className="ml-2 w-4 h-4" />
                </Button>
                <p className="text-[#888880] text-xs text-center mt-1">
                  Gmail opens with the recipient and message ready to send.
                </p>
              </form>
            </motion.div>
          </div>
        </div>
      </section>

      {/* ══════════════════════════════════════
          SIGN UP / AUTH SECTION
         ══════════════════════════════════════ */}
      <section id="signup" className="relative py-24 px-6 lg:px-12">
        <div className="max-w-md mx-auto">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-center mb-10"
          >
            <h2 className="font-display text-4xl sm:text-5xl font-bold tracking-[-0.04em] text-white mb-4">
              {isLogin ? "Welcome back" : "Join Orbit"}
            </h2>
            <p className="text-[#888880] text-lg">
              {isLogin
                ? "Sign in to access your wallet."
                : "Create your MPC-secured wallet in seconds."}
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, scale: 0.96, y: 20 }}
            whileInView={{ opacity: 1, scale: 1, y: 0 }}
            viewport={{ once: true }}
          >
            <div className="relative rounded-2xl border border-[#EA3A59]/10 bg-[#0a0a0a] shadow-2xl shadow-black/60 overflow-hidden">
              {/* Brand top accent line */}
              <div className="absolute top-0 left-[10%] right-[10%] h-[1px] bg-gradient-to-r from-transparent via-[#EA3A59]/70 to-transparent" />
              <div className="absolute top-0 right-0 w-32 h-32 bg-[#EA3A59]/[0.04] rounded-full blur-2xl pointer-events-none" />

              <div className="p-8 relative">
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
                    onClick={() => {
                      setIsLogin(!isLogin);
                      setError("");
                    }}
                    className="text-[#EA3A59] hover:text-[#ff6b84] font-semibold transition-colors"
                  >
                    {isLogin ? "Sign up" : "Log in"}
                  </button>
                </p>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* ── Footer ── */}
      <footer className="border-t border-white/[0.04] py-8 px-6 text-center">
        <div className="flex items-center justify-center gap-2 text-xs text-[#333330] font-display">
          <div className="w-1.5 h-1.5 rounded-full bg-[#EA3A59]/30" />
          <span>Powered by Solana · MPC Secured · Non-Custodial</span>
          <div className="w-1.5 h-1.5 rounded-full bg-[#EA3A59]/30" />
        </div>
        <p className="text-[#222] text-xs mt-2 font-display">
          © 2026 Orbit. All rights reserved.
        </p>
      </footer>
    </div>
  );
}
