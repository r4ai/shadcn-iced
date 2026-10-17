import { useState } from "react"

import { Checkbox } from "@/components/ui/checkbox"
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldLabel,
} from "@/components/ui/field"
import type { Fixtures } from "@/fixtures"

function Controlled({ disabled }: { disabled?: boolean }) {
  const [checked, setChecked] = useState(false)

  return (
    <div className="flex flex-col gap-3">
      <Field orientation="horizontal" data-disabled={disabled || undefined}>
        <Checkbox
          id="terms"
          checked={checked}
          onCheckedChange={setChecked}
          disabled={disabled}
        />
        <FieldLabel htmlFor="terms">Accept terms and conditions</FieldLabel>
      </Field>
      <p className="text-sm text-muted-foreground">
        {checked ? "Accepted" : "Not accepted"}
      </p>
    </div>
  )
}

export default {
  default: () => (
    <Field orientation="horizontal">
      <Checkbox id="terms" />
      <FieldLabel htmlFor="terms">Accept terms and conditions</FieldLabel>
    </Field>
  ),
  states: () => (
    <div className="flex flex-col gap-3">
      <Field orientation="horizontal">
        <Checkbox id="checked" defaultChecked />
        <FieldLabel htmlFor="checked">Accept terms and conditions</FieldLabel>
      </Field>
      <Field orientation="horizontal">
        <Checkbox id="described" />
        <FieldContent>
          <FieldLabel htmlFor="described">
            Accept terms and conditions
          </FieldLabel>
          <FieldDescription>
            By clicking this checkbox, you agree to the terms.
          </FieldDescription>
        </FieldContent>
      </Field>
      <Field orientation="horizontal" data-disabled>
        <Checkbox id="disabled" disabled />
        <FieldLabel htmlFor="disabled">Enable notifications</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-disabled>
        <Checkbox id="disabled-checked" disabled defaultChecked />
        <FieldLabel htmlFor="disabled-checked">Enable notifications</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <Checkbox id="invalid" aria-invalid />
        <FieldLabel htmlFor="invalid">Accept terms and conditions</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <Checkbox id="invalid-checked" aria-invalid defaultChecked />
        <FieldLabel htmlFor="invalid-checked">
          Accept terms and conditions
        </FieldLabel>
      </Field>
    </div>
  ),
  keyboard: () => <Controlled />,
  tab: () => <Controlled />,
  disabled: () => <Controlled disabled />,
} satisfies Fixtures
