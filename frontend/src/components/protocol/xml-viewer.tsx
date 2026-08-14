import { createJavaScriptRegexEngine } from "@shikijs/engine-javascript"
import { useEffect, useMemo, useState } from "react"
import { createHighlighter } from "shiki"
import { toast } from "sonner"
import xmlFormat from "xml-formatter"

import { useTheme } from "@/components/theme-provider"
import { Button } from "@/components/ui/button"

type XmlViewerProps = { xml: string | null; title?: string }
let highlighterPromise: ReturnType<typeof createHighlighter> | undefined
function getXmlHighlighter() {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      engine: createJavaScriptRegexEngine(),
      themes: ["github-light", "github-dark"],
      langs: ["xml"],
    })
  }
  return highlighterPromise
}
function safeFormatXml(xml: string) {
  try {
    return xmlFormat(xml, { indentation: "  ", lineSeparator: "\n" })
  } catch {
    return xml
  }
}

export function XmlViewer({ xml, title }: XmlViewerProps) {
  const { resolvedTheme } = useTheme()
  const [raw, setRaw] = useState(false)
  const [wrap, setWrap] = useState(false)
  const [highlighted, setHighlighted] = useState<string | null>(null)
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
  useEffect(() => {
    let cancelled = false
    if (displayXml === null) {
      setHighlighted(null)
      return
    }
    getXmlHighlighter()
      .then((highlighter) => {
        if (!cancelled)
          setHighlighted(
            highlighter.codeToHtml(displayXml, {
              lang: "xml",
              theme: resolvedTheme === "dark" ? "github-dark" : "github-light",
            }),
          )
      })
      .catch(() => setHighlighted(null))
    return () => {
      cancelled = true
    }
  }, [displayXml, resolvedTheme])
  return (
    <section className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <h2 className="font-semibold">{title}</h2>
        {originalXml !== null ? (
          <div className="flex gap-1">
            <Button
              variant={raw ? "secondary" : "default"}
              size="sm"
              onClick={() => setRaw(false)}
            >
              Pretty
            </Button>
            <Button
              variant={raw ? "default" : "secondary"}
              size="sm"
              onClick={() => setRaw(true)}
            >
              Raw
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setWrap((value) => !value)}
            >
              {wrap ? "Unwrap" : "Wrap"}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                navigator.clipboard
                  .writeText(originalXml)
                  .then(() => toast.success("Copied to clipboard"))
              }
            >
              Copy
            </Button>
          </div>
        ) : null}
      </div>
      {originalXml === null ? (
        <p className="rounded-lg border p-4 text-sm text-muted-foreground">
          No XML recorded.
        </p>
      ) : highlighted ? (
        <div
          className={`max-h-[550px] overflow-auto rounded-lg border bg-muted/30 p-4 text-xs [&_pre]:m-0 [&_pre]:bg-transparent [&_code]:font-mono ${wrap ? "whitespace-pre-wrap" : "whitespace-pre"}`}
          // biome-ignore lint/security/noDangerouslySetInnerHtml: Shiki generates this HTML locally from the persisted XML string.
          dangerouslySetInnerHTML={{ __html: highlighted }}
        />
      ) : (
        <pre
          className={`max-h-[550px] overflow-auto rounded-lg border bg-muted/30 p-4 text-xs ${wrap ? "whitespace-pre-wrap" : "whitespace-pre"}`}
        >
          {displayXml}
        </pre>
      )}
    </section>
  )
}
