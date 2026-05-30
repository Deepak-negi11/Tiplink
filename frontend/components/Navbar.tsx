"use client";

import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Rocket, Menu, X } from "lucide-react";

const navLinks = [
  { label: "Home", href: "#home" },
  { label: "Features", href: "#features" },
  { label: "Compare", href: "#compare" },
  { label: "Contact", href: "#contact" },
];

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const handleNav = (href: string) => {
    setMobileOpen(false);
    const el = document.querySelector(href);
    if (el) el.scrollIntoView({ behavior: "smooth" });
  };

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "bg-black/80 backdrop-blur-xl border-b border-[#EA3A59]/10 shadow-2xl shadow-black/50"
          : "bg-transparent"
      }`}
    >
      {/* Flying rocket animation */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="animate-fly-across absolute top-1/2 -translate-y-1/2">
          <Rocket className="w-4.5 h-4.5 text-[#EA3A59]/80 rotate-45" />
        </div>
      </div>

      <nav className="max-w-7xl mx-auto px-6 lg:px-12 py-4 flex items-center justify-between relative">
        {/* Logo — left side */}
        <button
          onClick={() => handleNav("#home")}
          className="flex items-center gap-2.5 group cursor-pointer"
        >
          <div className="relative w-8 h-8 flex items-center justify-center">
            <div className="absolute inset-0 bg-gradient-to-tr from-[#EA3A59] to-[#ff6b84] rounded-full animate-pulse-slow blur-[4px] opacity-70" />
            <div className="relative w-6 h-6 rounded-full border-2 border-white/90 flex items-center justify-center">
              <div className="w-2 h-2 bg-white rounded-full animate-orbit" />
            </div>
          </div>
          <span className="font-display font-bold text-2xl tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-white via-white to-white/70 group-hover:to-[#ff6b84] transition-all duration-300">
            Orbit
          </span>
        </button>

        {/* Desktop nav — right side */}
        <div className="hidden md:flex items-center gap-1">
          {navLinks.map((link) => (
            <button
              key={link.href}
              onClick={() => handleNav(link.href)}
              className="px-4 py-2 text-sm font-medium text-[#888880] hover:text-white transition-colors duration-200 rounded-lg hover:bg-white/[0.04] font-display"
            >
              {link.label}
            </button>
          ))}
          <div className="w-px h-6 bg-white/10 mx-2" />
          <button
            onClick={() => handleNav("#signup")}
            className="ml-1 px-5 py-2.5 text-sm font-bold text-white bg-[#EA3A59] rounded-xl hover:bg-[#ff6b84] transition-all duration-200 shadow-lg shadow-[#EA3A59]/20 hover:shadow-[#EA3A59]/40 active:scale-[0.97] font-display"
          >
            Sign Up
          </button>
        </div>

        {/* Mobile menu button */}
        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="md:hidden w-10 h-10 flex items-center justify-center rounded-xl text-white hover:bg-white/[0.05] transition-colors"
        >
          {mobileOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
        </button>
      </nav>

      {/* Mobile menu */}
      {mobileOpen && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -10 }}
          className="md:hidden bg-black/95 backdrop-blur-xl border-t border-[#EA3A59]/10 px-6 pb-6"
        >
          <div className="flex flex-col gap-1 pt-2">
            {navLinks.map((link) => (
              <button
                key={link.href}
                onClick={() => handleNav(link.href)}
                className="text-left px-4 py-3 text-sm font-medium text-[#888880] hover:text-white transition-colors rounded-xl hover:bg-white/[0.04] font-display"
              >
                {link.label}
              </button>
            ))}
            <button
              onClick={() => handleNav("#signup")}
              className="mt-2 w-full px-5 py-3 text-sm font-bold text-white bg-[#EA3A59] rounded-xl hover:bg-[#ff6b84] transition-all font-display"
            >
              Sign Up
            </button>
          </div>
        </motion.div>
      )}
    </header>
  );
}
