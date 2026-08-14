import Prism from "prismjs"
import "prismjs/components/prism-markup"
import { CheckIcon, CopyIcon } from "lucide-react"
import { useMemo, useState } from "react"
import { toast } from "sonner"
import xmlFormat from "xml-formatter"

import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip"

type XmlViewerProps = {
  xml: string | null
  title?: string
  raw: boolean
  wrap: boolean
  showDisplayControls?: boolean
  onRawChange?: (value: boolean) => void
  onWrapChange?: (value: boolean) => void
}
function safeFormatXml(xml: string) {
  try {
    return xmlFormat(xml, { indentation: "  ", lineSeparator: "\n" })
  } catch {
    return xml
  }
}

export function XmlViewer({
  xml,
  title,
  raw,
  wrap,
  showDisplayControls = false,
  onRawChange,
  onWrapChange,
}: XmlViewerProps) {
  const [copied, setCopied] = useState(false)
  const originalXml = xml
  const displayXml = useMemo(
    () =>
      originalXml === null
        ? null
        : raw
          ? originalXml
          : safeFormatXml(originalXml),
    [originalXml, raw],
  )
  const highlighted = useMemo(
    () =>
      displayXml === null
        ? null
        : Prism.highlight(displayXml, Prism.languages.markup, "xml"),
    [displayXml],
  )
  const copyXml = () => {
    if (originalXml === null) return
    navigator.clipboard.writeText(originalXml).then(() => {
      setCopied(true)
      toast.success("Copied raw XML")
      window.setTimeout(() => setCopied(false), 1200)
    })
  }
  const copyControl =
    originalXml !== null ? (
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Copy raw XML"
            className="absolute right-2 top-2 z-10"
            onClick={copyXml}
          >
            {copied ? <CheckIcon /> : <CopyIcon />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>Copy raw XML</TooltipContent>
      </Tooltip>
    ) : null
  return (
    <section className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <h2 className="font-semibold">{title}</h2>
        <div className="flex items-center gap-2">
          {showDisplayControls ? (
            <div className="flex items-center gap-3 text-xs text-muted-foreground">
              <SwitchControl label="Raw" checked={raw} onChange={onRawChange} />
              <SwitchControl
                label="Wrap"
                checked={wrap}
                onChange={onWrapChange}
              />
            </div>
          ) : null}
        </div>
      </div>
      {originalXml === null ? (
        <p className="rounded-lg border p-4 text-sm text-muted-foreground">
          No XML recorded.
        </p>
      ) : highlighted ? (
        <div className="relative">
          {copyControl}
          <div
            className={`xml-viewer max-h-[550px] overflow-auto rounded-lg border bg-muted/30 p-4 pr-12 text-xs font-mono ${wrap ? "whitespace-pre-wrap" : "whitespace-pre"}`}
            // biome-ignore lint/security/noDangerouslySetInnerHtml: Prism escapes XML before producing local token markup.
            dangerouslySetInnerHTML={{ __html: highlighted }}
          />
        </div>
      ) : (
        <div className="relative">
          {copyControl}
          <pre
            className={`max-h-[550px] overflow-auto rounded-lg border bg-muted/30 p-4 pr-12 text-xs ${wrap ? "whitespace-pre-wrap" : "whitespace-pre"}`}
          >
            {displayXml}
          </pre>
        </div>
      )}
    </section>
  )
}

function SwitchControl({
  label,
  checked,
  onChange,
}: {
  label: string
  checked: boolean
  onChange?: (value: boolean) => void
}) {
  return (
    <div className="flex items-center gap-1.5">
      {label}
      <Switch
        size="sm"
        checked={checked}
        onCheckedChange={onChange}
        aria-label={label}
      />
    </div>
  )
}
