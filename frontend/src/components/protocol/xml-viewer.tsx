import Prism from "prismjs"
import "prismjs/components/prism-markup"
import { useMemo, useState } from "react"
import { toast } from "sonner"
import xmlFormat from "xml-formatter"

import { Button } from "@/components/ui/button"

type XmlViewerProps = { xml: string | null; title?: string }
function safeFormatXml(xml: string) {
  try {
    return xmlFormat(xml, { indentation: "  ", lineSeparator: "\n" })
  } catch {
    return xml
  }
}

export function XmlViewer({ xml, title }: XmlViewerProps) {
  const [raw, setRaw] = useState(false)
  const [wrap, setWrap] = useState(false)
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
          className={`xml-viewer max-h-[550px] overflow-auto rounded-lg border bg-muted/30 p-4 text-xs font-mono ${wrap ? "whitespace-pre-wrap" : "whitespace-pre"}`}
          // biome-ignore lint/security/noDangerouslySetInnerHtml: Prism escapes XML before producing local token markup.
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
