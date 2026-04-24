"use client";

import { motion, AnimatePresence } from "framer-motion";
import { Wallet, Send, RefreshCcw, LogOut, ChevronRight } from "lucide-react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useAuthStore } from "@/store/useStore";
import { useEffect } from "react";

const navItems = [
  { label: "Wallet",      href: "/dashboard",        icon: Wallet },
  { label: "Send",        href: "/dashboard/create",  icon: Send },
  { label: "Swap",        href: "/dashboard/swap",    icon: RefreshCcw },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const router   = useRouter();
  const { user, token, logout } = useAuthStore();

  useEffect(() => {
    if (!token) router.push("/");
  }, [token, router]);

  const handleLogout = () => { logout(); router.push("/"); };
  const shortEmail   = user?.email ? user.email.split("@")[0] : "";

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-[#e8e3d5] flex flex-col md:flex-row relative overflow-hidden">

      {/* ── Ambient orbs ── */}
      <div className="orb orb-yellow fixed top-[-20%] left-[-10%] w-[45%] h-[45%] pointer-events-none z-0" />
      <div className="orb orb-yellow-dim fixed bottom-[-15%] right-[-10%] w-[35%] h-[35%] pointer-events-none z-0" />

      {/* ══ Sidebar ══ */}
      <nav className="w-full md:w-[260px] md:min-h-screen border-b md:border-b-0 md:border-r border-[rgba(245,197,24,0.07)] bg-[#0d0d0d]/95 backdrop-blur-xl flex flex-col z-20 md:fixed md:left-0 md:top-0 shrink-0">

        {/* Logo */}
        <div className="px-6 py-6 flex items-center gap-3 border-b border-[rgba(245,197,24,0.07)]">
          <div className="w-9 h-9 rounded-lg bg-[#f5c518] flex items-center justify-center font-bold text-lg text-[#0a0a00] shadow-lg shadow-yellow-500/25 font-display">
            T
          </div>
          <span className="font-display font-bold text-xl tracking-tight text-white">TipLink</span>
        </div>

        {/* Nav items */}
        <div className="flex-1 flex flex-row md:flex-col gap-1 p-3 md:p-4 overflow-x-auto md:overflow-visible md:mt-3">
          {navItems.map((item) => {
            const isActive = pathname === item.href;
            const Icon = item.icon;
            return (
              <Link key={item.href} href={item.href} className="shrink-0">
                <div
                  className={`flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 relative ${
                    isActive
                      ? "bg-[#f5c518]/[0.1] text-[#f5c518]"
                      : "text-[#555550] hover:text-[#e8e3d5] hover:bg-white/[0.04]"
                  }`}
                >
                  {isActive && (
                    <motion.div
                      layoutId="nav-indicator"
                      className="nav-active-bar"
                      transition={{ type: "spring", stiffness: 380, damping: 32 }}
                    />
                  )}
                  <Icon className="w-[18px] h-[18px] shrink-0" />
                  <span className="hidden md:inline text-sm font-semibold font-display">
                    {item.label}
                  </span>
                  {isActive && (
                    <ChevronRight className="w-3.5 h-3.5 ml-auto hidden md:block opacity-50" />
                  )}
                </div>
              </Link>
            );
          })}
        </div>

        {/* User + Logout */}
        <div className="p-4 hidden md:flex flex-col gap-2 border-t border-[rgba(245,197,24,0.07)]">
          {user && (
            <div className="flex items-center gap-3 px-3 py-2">
              <div className="w-8 h-8 rounded-full bg-[#f5c518]/10 border border-[#f5c518]/20 flex items-center justify-center text-xs font-bold text-[#f5c518] font-display shrink-0">
                {shortEmail.charAt(0).toUpperCase()}
              </div>
              <div className="flex flex-col min-w-0">
                <span className="text-sm text-[#e8e3d5] font-semibold truncate">{user.email}</span>
                <span className="text-xs text-[#555550] font-mono">
                  {user.public_key.slice(0, 6)}...{user.public_key.slice(-4)}
                </span>
              </div>
            </div>
          )}
          <button
            onClick={handleLogout}
            className="flex items-center gap-3 px-4 py-2.5 w-full text-left text-[#555550] hover:text-[#ff3b30] hover:bg-[#ff3b30]/[0.06] rounded-xl transition-all duration-200 text-sm font-semibold font-display"
          >
            <LogOut className="w-4 h-4" />
            <span>Sign Out</span>
          </button>
        </div>
      </nav>

      {/* ══ Main content ══ */}
      <main className="flex-1 overflow-y-auto p-4 md:p-8 lg:p-12 z-10 md:ml-[260px] min-h-screen">
        <AnimatePresence mode="wait">
          <motion.div
            key={pathname}
            initial={{ opacity: 0, y: 14 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
            className="max-w-4xl mx-auto w-full"
          >
            {children}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
