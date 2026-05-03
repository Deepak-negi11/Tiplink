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
          "inline-flex items-center justify-center whitespace-nowrap rounded-xl text-sm font-semibold transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--primary)]/50 disabled:pointer-events-none disabled:opacity-40 cursor-pointer active:scale-[0.97]",
          {
            "bg-[var(--primary)] text-[var(--primary-foreground)] hover:brightness-110 shadow-lg shadow-[var(--primary)]/20 hover:shadow-[var(--primary)]/35 tracking-tight":
              variant === "default",
            "bg-[var(--danger)] text-white hover:brightness-110 shadow-lg shadow-[var(--danger)]/20":
              variant === "destructive",
            "border border-[var(--primary)]/30 bg-transparent text-[var(--primary-light)] hover:bg-[var(--primary)]/5 hover:border-[var(--primary)]/60":
              variant === "outline",
            "bg-[var(--surface-high)] text-[var(--foreground)] hover:bg-[var(--surface-highest)] border border-white/5":
              variant === "secondary",
            "hover:bg-white/[0.05] text-[var(--muted)] hover:text-[var(--foreground)]":
              variant === "ghost",
            "text-[var(--primary-light)] underline-offset-4 hover:underline":
              variant === "link",
          },
          {
            "h-11 px-6 py-2": size === "default",
            "h-9 rounded-lg px-3 text-xs": size === "sm",
            "h-13 rounded-2xl px-8 text-base font-bold py-3.5": size === "lg",
            "h-11 w-11 rounded-xl": size === "icon",
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
