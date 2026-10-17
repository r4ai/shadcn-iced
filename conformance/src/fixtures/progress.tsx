import {
  Progress,
  ProgressLabel,
  ProgressValue,
} from "@/components/ui/progress"
import type { Fixtures } from "@/fixtures"

export default {
  demo: () => <Progress value={66} className="w-[60%]" />,
  label: () => (
    <Progress value={56} className="w-full max-w-sm">
      <ProgressLabel>Upload progress</ProgressLabel>
      <ProgressValue />
    </Progress>
  ),
  values: () => (
    <div className="flex w-full max-w-sm flex-col gap-4">
      <Progress value={0} />
      <Progress value={25} />
      <Progress value={100} />
    </div>
  ),
} satisfies Fixtures
