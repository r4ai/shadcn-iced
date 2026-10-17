import {
  InputOTP,
  InputOTPGroup,
  InputOTPSeparator,
  InputOTPSlot,
} from "@/components/ui/input-otp"
import type { Fixtures } from "@/fixtures"

function Six({
  invalid,
  ...props
}: {
  invalid?: boolean
  defaultValue?: string
  disabled?: boolean
}) {
  return (
    <InputOTP maxLength={6} aria-invalid={invalid} {...props}>
      <InputOTPGroup>
        <InputOTPSlot index={0} aria-invalid={invalid} />
        <InputOTPSlot index={1} aria-invalid={invalid} />
        <InputOTPSlot index={2} aria-invalid={invalid} />
      </InputOTPGroup>
      <InputOTPSeparator />
      <InputOTPGroup>
        <InputOTPSlot index={3} aria-invalid={invalid} />
        <InputOTPSlot index={4} aria-invalid={invalid} />
        <InputOTPSlot index={5} aria-invalid={invalid} />
      </InputOTPGroup>
    </InputOTP>
  )
}

export default {
  default: () => <Six />,
  value: () => (
    <InputOTP maxLength={4} defaultValue="12">
      <InputOTPGroup>
        <InputOTPSlot index={0} />
        <InputOTPSlot index={1} />
        <InputOTPSlot index={2} />
        <InputOTPSlot index={3} />
      </InputOTPGroup>
    </InputOTP>
  ),
  invalid: () => <Six defaultValue="123" invalid />,
  disabled: () => <Six defaultValue="123" disabled />,
} satisfies Fixtures
