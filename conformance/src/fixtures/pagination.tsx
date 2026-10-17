import { useState } from "react"

import type { Fixtures } from "@/fixtures"
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination"

/** The pages to display; `null` stands for an ellipsis. */
function pages(current: number, total: number, siblings: number) {
  const start = Math.max(current - siblings, 1)
  const end = Math.min(current + siblings, total)
  const result: (number | null)[] = []

  if (start > 1) {
    result.push(1)
    if (start > 2) result.push(null)
  }
  for (let page = start; page <= end; page++) result.push(page)
  if (end < total) {
    if (end + 1 < total) result.push(null)
    result.push(total)
  }

  return result
}

function Demo({
  initial,
  total,
  siblings,
  controls,
}: {
  initial: number
  total: number
  siblings: number
  controls: boolean
}) {
  const [page, setPage] = useState(initial)
  const go = (next: number) => (event: React.MouseEvent) => {
    event.preventDefault()
    setPage(Math.min(Math.max(next, 1), total))
  }

  return (
    <Pagination>
      <PaginationContent>
        {controls && (
          <PaginationItem>
            <PaginationPrevious href="#" onClick={go(page - 1)} />
          </PaginationItem>
        )}
        {pages(page, total, siblings).map((item, index) => (
          <PaginationItem key={index}>
            {item === null ? (
              <PaginationEllipsis />
            ) : (
              <PaginationLink
                href="#"
                isActive={item === page}
                onClick={go(item)}
              >
                {item}
              </PaginationLink>
            )}
          </PaginationItem>
        ))}
        {controls && (
          <PaginationItem>
            <PaginationNext href="#" onClick={go(page + 1)} />
          </PaginationItem>
        )}
      </PaginationContent>
    </Pagination>
  )
}

export default {
  default: () => <Demo initial={2} total={10} siblings={1} controls />,
  simple: () => <Demo initial={3} total={5} siblings={5} controls={false} />,
} satisfies Fixtures
