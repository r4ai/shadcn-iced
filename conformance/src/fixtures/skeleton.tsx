import { Skeleton } from "@/components/ui/skeleton"
import type { Fixtures } from "@/fixtures"

export default {
  avatar: () => (
    <div className="flex items-center gap-4">
      <Skeleton className="size-10 shrink-0 rounded-full" />
      <div className="grid gap-2">
        <Skeleton className="h-4 w-37.5" />
        <Skeleton className="h-4 w-25" />
      </div>
    </div>
  ),
  lines: () => (
    <div className="flex w-full max-w-sm flex-col gap-3">
      <Skeleton className="h-4 w-full" />
      <Skeleton className="h-4 w-2/3" />
      <Skeleton className="h-8 w-full" />
      <Skeleton className="h-8 w-24" />
    </div>
  ),
} satisfies Fixtures
