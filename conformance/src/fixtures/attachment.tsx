import { useState } from "react"
import {
  CheckIcon,
  ClockIcon,
  FileTextIcon,
  FileWarningIcon,
  ImageIcon,
  RefreshCwIcon,
  XIcon,
} from "lucide-react"

import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentGroup,
  AttachmentMedia,
  AttachmentTitle,
  AttachmentTrigger,
} from "@/components/ui/attachment"
import { Spinner } from "@/components/ui/spinner"
import type { Fixtures } from "@/fixtures"

function Remove() {
  return (
    <AttachmentAction aria-label="Remove">
      <XIcon />
    </AttachmentAction>
  )
}

function Tile({ name, meta }: { name: string; meta: string }) {
  return (
    <Attachment orientation="vertical">
      <AttachmentMedia>
        <ImageIcon />
      </AttachmentMedia>
      <AttachmentContent>
        <AttachmentTitle>{name}</AttachmentTitle>
        <AttachmentDescription>{meta}</AttachmentDescription>
      </AttachmentContent>
    </Attachment>
  )
}

function Trigger() {
  const [opened, setOpened] = useState(false)

  return (
    <div className="flex w-full max-w-sm flex-col gap-3">
      <Attachment className="w-full">
        <AttachmentMedia>
          <FileTextIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>project-brief.pdf</AttachmentTitle>
          <AttachmentDescription>Click to preview</AttachmentDescription>
        </AttachmentContent>
        <AttachmentTrigger
          aria-label="Preview"
          onClick={() => setOpened(true)}
        />
      </Attachment>
      <p className="text-sm">{opened ? "Opened" : "Not opened"}</p>
    </div>
  )
}

export default {
  states: () => (
    <div className="flex w-full max-w-sm flex-col gap-2">
      <Attachment state="idle" className="w-full">
        <AttachmentMedia>
          <ClockIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>selected-file.pdf</AttachmentTitle>
          <AttachmentDescription>Ready to upload</AttachmentDescription>
        </AttachmentContent>
        <AttachmentActions>
          <Remove />
        </AttachmentActions>
      </Attachment>
      <Attachment state="uploading" className="w-full">
        <AttachmentMedia>
          <Spinner />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>design-system.zip</AttachmentTitle>
          <AttachmentDescription>Uploading · 64%</AttachmentDescription>
        </AttachmentContent>
        <AttachmentActions>
          <Remove />
        </AttachmentActions>
      </Attachment>
      <Attachment state="processing" className="w-full">
        <AttachmentMedia>
          <FileTextIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>market-research.pdf</AttachmentTitle>
          <AttachmentDescription>Processing document</AttachmentDescription>
        </AttachmentContent>
        <AttachmentActions>
          <Remove />
        </AttachmentActions>
      </Attachment>
      <Attachment state="error" className="w-full">
        <AttachmentMedia>
          <FileWarningIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>financial-model.xlsx</AttachmentTitle>
          <AttachmentDescription>
            Upload failed. Try again.
          </AttachmentDescription>
        </AttachmentContent>
        <AttachmentActions>
          <AttachmentAction aria-label="Retry">
            <RefreshCwIcon />
          </AttachmentAction>
          <Remove />
        </AttachmentActions>
      </Attachment>
      <Attachment className="w-full">
        <AttachmentMedia>
          <CheckIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>uploaded-report.pdf</AttachmentTitle>
          <AttachmentDescription>Uploaded · 1.8 MB</AttachmentDescription>
        </AttachmentContent>
        <AttachmentActions>
          <Remove />
        </AttachmentActions>
      </Attachment>
    </div>
  ),
  sizes: () => (
    <div className="flex w-full max-w-sm flex-col gap-3">
      <Attachment className="w-full">
        <AttachmentMedia>
          <FileTextIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>Default attachment</AttachmentTitle>
          <AttachmentDescription>PDF · 2.4 MB</AttachmentDescription>
        </AttachmentContent>
      </Attachment>
      <Attachment size="sm" className="w-full">
        <AttachmentMedia>
          <FileTextIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>Small attachment</AttachmentTitle>
          <AttachmentDescription>PDF · 2.4 MB</AttachmentDescription>
        </AttachmentContent>
      </Attachment>
      <Attachment size="xs" className="w-full">
        <AttachmentMedia>
          <FileTextIcon />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle>Extra small attachment</AttachmentTitle>
        </AttachmentContent>
      </Attachment>
    </div>
  ),
  group: () => (
    <div className="w-full max-w-sm">
      <AttachmentGroup>
        <Tile name="workspace.png" meta="PNG · 820 KB" />
        <Tile name="desk-reference.jpg" meta="JPG · 1.1 MB" />
        <Tile name="office-reference.jpg" meta="JPG · 940 KB" />
      </AttachmentGroup>
    </div>
  ),
  trigger: () => <Trigger />,
} satisfies Fixtures
