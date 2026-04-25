import * as React from "react"
import { cn } from "@/lib/utils"

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
  size?: "default" | "sm" | "lg" | "icon";
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", size = "default", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center justify-center whitespace-nowrap rounded-xl text-sm font-semibold transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-yellow-400/50 disabled:pointer-events-none disabled:opacity-40 cursor-pointer active:scale-[0.97]",
          {
            "bg-[#f5c518] text-[#0a0a00] hover:bg-[#ffd740] shadow-lg shadow-yellow-500/20 hover:shadow-yellow-500/35 tracking-tight":
              variant === "default",
            "bg-[#ff3b30]/90 text-white hover:bg-[#ff3b30] shadow-lg shadow-red-500/20":
              variant === "destructive",
            "border border-[#f5c518]/30 bg-transparent text-[#f5c518] hover:bg-[#f5c518]/8 hover:border-[#f5c518]/60":
              variant === "outline",
            "bg-[#222222] text-[#e8e3d5] hover:bg-[#2d2d2d] border border-white/5":
              variant === "secondary",
            "hover:bg-white/[0.05] text-[#888880] hover:text-[#e8e3d5]":
              variant === "ghost",
            "text-[#f5c518] underline-offset-4 hover:underline":
              variant === "link",
          },
          {
            "h-11 px-6 py-2":              size === "default",
            "h-9 rounded-lg px-3 text-xs": size === "sm",
            "h-13 rounded-2xl px-8 text-base font-bold py-3.5": size === "lg",
            "h-11 w-11 rounded-xl":        size === "icon",
          },
          className
        )}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button }
