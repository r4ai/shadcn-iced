import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSeparator,
  FieldSet,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import type { Fixtures } from "@/fixtures"

const fixtures = {
  default: () => (
    <Field>
      <FieldLabel htmlFor="username">Username</FieldLabel>
      <Input id="username" placeholder="Max Leiter" />
      <FieldDescription>
        Choose a unique username for your account.
      </FieldDescription>
    </Field>
  ),
  error: () => (
    <Field data-invalid>
      <FieldLabel htmlFor="username">Username</FieldLabel>
      <Input id="username" placeholder="Max Leiter" aria-invalid />
      <FieldError>Username is already taken.</FieldError>
    </Field>
  ),
  disabled: () => (
    <Field data-disabled>
      <FieldLabel htmlFor="username">Username</FieldLabel>
      <Input id="username" placeholder="Max Leiter" disabled />
      <FieldDescription>
        Choose a unique username for your account.
      </FieldDescription>
    </Field>
  ),
  horizontal: () => (
    <Field orientation="horizontal">
      <FieldContent>
        <FieldLabel htmlFor="mfa">Multi-factor authentication</FieldLabel>
        <FieldDescription>Enable multi-factor authentication.</FieldDescription>
      </FieldContent>
      <Switch id="mfa" />
    </Field>
  ),
  fieldset: () => (
    <FieldSet>
      <FieldLegend>Address Information</FieldLegend>
      <FieldDescription>
        We need your address to deliver your order.
      </FieldDescription>
      <FieldGroup>
        <Field>
          <FieldLabel htmlFor="street">Street Address</FieldLabel>
          <Input id="street" placeholder="123 Main St" />
        </Field>
        <Field>
          <FieldLabel htmlFor="city">City</FieldLabel>
          <Input id="city" placeholder="New York" />
        </Field>
      </FieldGroup>
    </FieldSet>
  ),
  separator: () => (
    <FieldGroup>
      <Field>
        <FieldLabel htmlFor="email">Email</FieldLabel>
        <Input id="email" placeholder="m@example.com" />
      </Field>
      <FieldSeparator>Or continue with</FieldSeparator>
      <Field>
        <FieldLabel htmlFor="phone">Phone</FieldLabel>
        <Input id="phone" placeholder="+1 555 0100" />
      </Field>
    </FieldGroup>
  ),
} satisfies Fixtures

// The page is 768px wide so that `md:text-sm` applies, like on iced; the
// fixtures are `w-80` wide.
const narrow = new Set([
  "default",
  "error",
  "disabled",
  "horizontal",
  "fieldset",
  "separator",
])

export default Object.fromEntries(
  Object.entries(fixtures).map(([id, render]) => [
    id,
    narrow.has(id) ? () => <div className="w-80">{render()}</div> : render,
  ])
) satisfies Fixtures
