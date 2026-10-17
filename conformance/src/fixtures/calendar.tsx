import { useState } from "react"
import type { DateRange } from "react-day-picker"

import type { Fixtures } from "@/fixtures"
import { Calendar } from "@/components/ui/calendar"

/** The days of June 2025 that are booked (17th to 30th). */
const bookedDates = Array.from(
  { length: 14 },
  (_, index) => new Date(2025, 5, 17 + index)
)

function Single() {
  const [date, setDate] = useState<Date | undefined>(new Date(2025, 5, 12))

  return (
    <Calendar
      mode="single"
      defaultMonth={new Date(2025, 5)}
      selected={date}
      onSelect={setDate}
      className="rounded-lg border"
    />
  )
}

function Range() {
  const [range, setRange] = useState<DateRange | undefined>({
    from: new Date(2025, 5, 12),
    to: new Date(2025, 5, 17),
  })

  return (
    <Calendar
      mode="range"
      defaultMonth={new Date(2025, 5)}
      selected={range}
      onSelect={setRange}
      numberOfMonths={2}
      className="rounded-lg border"
    />
  )
}

function Booked() {
  const [date, setDate] = useState<Date | undefined>(new Date(2025, 5, 12))

  return (
    <Calendar
      mode="single"
      defaultMonth={new Date(2025, 5)}
      selected={date}
      onSelect={setDate}
      disabled={bookedDates}
      modifiers={{ booked: bookedDates }}
      modifiersClassNames={{ booked: "[&>button]:line-through opacity-100" }}
      className="rounded-lg border"
    />
  )
}

export default {
  default: () => <Single />,
  range: () => <Range />,
  dropdowns: () => (
    <Calendar
      mode="single"
      captionLayout="dropdown"
      defaultMonth={new Date(2025, 5)}
      startMonth={new Date(2016, 0)}
      endMonth={new Date(2036, 11)}
      className="rounded-lg border"
    />
  ),
  "week-numbers": () => (
    <Calendar
      mode="single"
      defaultMonth={new Date(2025, 5)}
      selected={new Date(2025, 5, 12)}
      showWeekNumber
      className="rounded-lg border"
    />
  ),
  booked: () => <Booked />,
} satisfies Fixtures
