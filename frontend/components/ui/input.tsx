import * as React from "react"
import { cn } from "@/lib/utils"

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-12 w-full rounded-xl border border-white/[0.07] bg-[#111111] px-4 py-2 text-sm text-[#e8e3d5] placeholder:text-[#555550] focus-visible:outline-none focus-visible:border-[#EA3A59]/45 focus-visible:ring-[3px] focus-visible:ring-[#EA3A59]/8 disabled:cursor-not-allowed disabled:opacity-40 transition-all duration-200",
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }
