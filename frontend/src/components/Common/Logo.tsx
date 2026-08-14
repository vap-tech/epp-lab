import { cn } from "@/lib/utils"

interface LogoProps {
  variant?: "full" | "icon" | "responsive"
  className?: string
  asLink?: boolean
}

export function Logo({
  variant = "full",
  className,
  asLink = true,
}: LogoProps) {
  const content =
    variant === "responsive" ? (
      <>
        <span
          className={cn(
            "font-semibold group-data-[collapsible=icon]:hidden",
            className,
          )}
        >
          EPP Lab
        </span>
        <span
          className={cn(
            "hidden font-semibold group-data-[collapsible=icon]:block",
            className,
          )}
        >
          E
        </span>
      </>
    ) : (
      <span className={cn("font-semibold", className)}>EPP Lab</span>
    )

  if (!asLink) {
    return content
  }

  return content
}
