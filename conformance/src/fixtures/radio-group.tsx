import { useState } from "react"

import {
  Field,
  FieldContent,
  FieldDescription,
  FieldLabel,
} from "@/components/ui/field"
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"
import type { Fixtures } from "@/fixtures"

function Density({
  disabled,
  ...props
}: React.ComponentProps<typeof RadioGroup>) {
  const data = { "data-disabled": disabled || undefined }

  return (
    <RadioGroup
      defaultValue="comfortable"
      className="w-fit"
      disabled={disabled}
      {...props}
    >
      <Field orientation="horizontal" {...data}>
        <RadioGroupItem value="default" id="r1" />
        <FieldLabel htmlFor="r1" className="font-normal">
          Default
        </FieldLabel>
      </Field>
      <Field orientation="horizontal" {...data}>
        <RadioGroupItem value="comfortable" id="r2" />
        <FieldLabel htmlFor="r2" className="font-normal">
          Comfortable
        </FieldLabel>
      </Field>
      <Field orientation="horizontal" {...data}>
        <RadioGroupItem value="compact" id="r3" />
        <FieldLabel htmlFor="r3" className="font-normal">
          Compact
        </FieldLabel>
      </Field>
    </RadioGroup>
  )
}

function Controlled() {
  const [value, setValue] = useState("comfortable")

  return (
    <div className="flex flex-col gap-3">
      <Density
        defaultValue={undefined}
        value={value}
        onValueChange={(next) => setValue(next as string)}
      />
      <p className="text-sm text-muted-foreground">Selected: {value}</p>
    </div>
  )
}

export default {
  default: () => <Density />,
  keyboard: () => <Controlled />,
  description: () => (
    <RadioGroup defaultValue="comfortable">
      <Field orientation="horizontal">
        <RadioGroupItem value="default" id="r1" />
        <FieldContent>
          <FieldLabel htmlFor="r1" className="font-normal">
            Default
          </FieldLabel>
          <FieldDescription>
            Standard spacing for most use cases.
          </FieldDescription>
        </FieldContent>
      </Field>
      <Field orientation="horizontal">
        <RadioGroupItem value="comfortable" id="r2" />
        <FieldContent>
          <FieldLabel htmlFor="r2" className="font-normal">
            Comfortable
          </FieldLabel>
          <FieldDescription>More space between elements.</FieldDescription>
        </FieldContent>
      </Field>
    </RadioGroup>
  ),
  disabled: () => <Density disabled />,
  invalid: () => (
    <RadioGroup defaultValue="comfortable" className="w-fit">
      <Field orientation="horizontal" data-invalid>
        <RadioGroupItem value="default" id="r1" aria-invalid />
        <FieldLabel htmlFor="r1" className="font-normal">
          Default
        </FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <RadioGroupItem value="comfortable" id="r2" aria-invalid />
        <FieldLabel htmlFor="r2" className="font-normal">
          Comfortable
        </FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <RadioGroupItem value="compact" id="r3" aria-invalid />
        <FieldLabel htmlFor="r3" className="font-normal">
          Compact
        </FieldLabel>
      </Field>
    </RadioGroup>
  ),
} satisfies Fixtures
