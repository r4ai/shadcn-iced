import { FileCodeIcon, TerminalIcon } from "lucide-react"

import {
  CodeBlock,
  CodeBlockActions,
  CodeBlockContent,
  CodeBlockCopyButton,
  CodeBlockHeader,
  CodeBlockLine,
  CodeBlockTitle,
} from "@/components/ui/code-block"
import type { Fixtures } from "@/fixtures"

const INSTALL = "pnpm add ai @ai-sdk/react zod"

const PACKAGE = [
  "{",
  '  "name": "chat",',
  '  "private": true,',
  '  "scripts": {',
  '    "dev": "next dev",',
  '    "build": "next build"',
  "  }",
  "}",
]

const ERROR =
  "TypeError: Cannot read properties of undefined (reading 'map') at MessageList (components/message-list.tsx:18:22)"

export default {
  header: () => (
    <div className="w-full max-w-md">
      <CodeBlock>
        <CodeBlockHeader>
          <CodeBlockTitle>
            <TerminalIcon />
            Terminal
          </CodeBlockTitle>
          <CodeBlockActions>
            <CodeBlockCopyButton value={INSTALL} />
          </CodeBlockActions>
        </CodeBlockHeader>
        <CodeBlockContent>{INSTALL}</CodeBlockContent>
      </CodeBlock>
    </div>
  ),
  "line-numbers": () => (
    <div className="w-full max-w-md">
      <CodeBlock>
        <CodeBlockHeader>
          <CodeBlockTitle>
            <FileCodeIcon />
            package.json
          </CodeBlockTitle>
        </CodeBlockHeader>
        <CodeBlockContent lineNumbers>
          {PACKAGE.map((line, index) => (
            <CodeBlockLine
              key={index}
              variant={index === 3 || index === 4 ? "highlighted" : "default"}
            >
              {line}
            </CodeBlockLine>
          ))}
        </CodeBlockContent>
      </CodeBlock>
    </div>
  ),
  diff: () => (
    <div className="w-full max-w-md">
      <CodeBlock>
        <CodeBlockHeader>
          <CodeBlockTitle>
            <FileCodeIcon />
            greeting.tsx
          </CodeBlockTitle>
        </CodeBlockHeader>
        <CodeBlockContent>
          <CodeBlockLine>export function Greeting() {"{"}</CodeBlockLine>
          <CodeBlockLine variant="removed">
            {"  return <p>Hello</p>"}
          </CodeBlockLine>
          <CodeBlockLine variant="added">
            {"  return <p>Hello!</p>"}
          </CodeBlockLine>
          <CodeBlockLine>{"}"}</CodeBlockLine>
        </CodeBlockContent>
      </CodeBlock>
    </div>
  ),
  "floating-actions": () => (
    <div className="w-full max-w-md">
      <CodeBlock>
        <CodeBlockActions>
          <CodeBlockCopyButton value={INSTALL} />
        </CodeBlockActions>
        <CodeBlockContent>{INSTALL}</CodeBlockContent>
      </CodeBlock>
    </div>
  ),
  "destructive-sm": () => (
    <div className="w-full max-w-md">
      <CodeBlock variant="destructive" size="sm">
        <CodeBlockHeader>
          <CodeBlockTitle>Error</CodeBlockTitle>
          <CodeBlockActions>
            <CodeBlockCopyButton value={ERROR} />
          </CodeBlockActions>
        </CodeBlockHeader>
        <CodeBlockContent wrap>{ERROR}</CodeBlockContent>
      </CodeBlock>
    </div>
  ),
} satisfies Fixtures
