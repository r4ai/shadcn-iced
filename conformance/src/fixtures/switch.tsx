import { useState } from "react"

import { Field, FieldLabel } from "@/components/ui/field"
import { Switch } from "@/components/ui/switch"
import type { Fixtures } from "@/fixtures"

function Controlled() {
  const [checked, setChecked] = useState(false)

  return (
    <div className="flex flex-col gap-3">
      <Field orientation="horizontal">
        <Switch
          id="airplane-mode"
          checked={checked}
          onCheckedChange={setChecked}
        />
        <FieldLabel htmlFor="airplane-mode">Airplane Mode</FieldLabel>
      </Field>
      <p className="text-sm text-muted-foreground">
        {checked ? "Airplane mode is on" : "Airplane mode is off"}
      </p>
    </div>
  )
}

export default {
  default: () => (
    <Field orientation="horizontal">
      <Switch id="airplane-mode" />
      <FieldLabel htmlFor="airplane-mode">Airplane Mode</FieldLabel>
    </Field>
  ),
  sizes: () => (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-4">
        <Switch size="sm" />
        <Switch size="sm" defaultChecked />
      </div>
      <div className="flex items-center gap-4">
        <Switch />
        <Switch defaultChecked />
      </div>
    </div>
  ),
  states: () => (
    <div className="flex flex-col gap-3">
      <Field orientation="horizontal" data-disabled>
        <Switch id="disabled" disabled />
        <FieldLabel htmlFor="disabled">Disabled</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-disabled>
        <Switch id="disabled-checked" disabled defaultChecked />
        <FieldLabel htmlFor="disabled-checked">Disabled checked</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <Switch id="invalid" aria-invalid />
        <FieldLabel htmlFor="invalid">Invalid</FieldLabel>
      </Field>
      <Field orientation="horizontal" data-invalid>
        <Switch id="invalid-checked" aria-invalid defaultChecked />
        <FieldLabel htmlFor="invalid-checked">Invalid checked</FieldLabel>
      </Field>
    </div>
  ),
  keyboard: () => <Controlled />,
  tab: () => <Controlled />,
} satisfies Fixtures
