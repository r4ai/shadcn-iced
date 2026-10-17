import { CircleAlertIcon, CircleCheckIcon, InfoIcon } from "lucide-react"

import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import type { Fixtures } from "@/fixtures"

export default {
  default: () => (
    <div className="grid max-w-md items-start gap-4">
      <Alert>
        <CircleCheckIcon />
        <AlertTitle>Payment successful</AlertTitle>
        <AlertDescription>
          Your payment of $29.99 has been processed. A receipt has been sent to
          your email address.
        </AlertDescription>
      </Alert>
      <Alert>
        <InfoIcon />
        <AlertTitle>New feature available</AlertTitle>
        <AlertDescription>
          We've added dark mode support. You can enable it in your account
          settings.
        </AlertDescription>
      </Alert>
    </div>
  ),
  destructive: () => (
    <Alert variant="destructive" className="max-w-md">
      <CircleAlertIcon />
      <AlertTitle>Payment failed</AlertTitle>
      <AlertDescription>
        Your payment could not be processed. Please check your payment method
        and try again.
      </AlertDescription>
    </Alert>
  ),
  action: () => (
    <Alert className="max-w-md">
      <AlertTitle>Dark mode is now available</AlertTitle>
      <AlertDescription>
        Enable it under your profile settings to get started.
      </AlertDescription>
      <AlertAction>
        <Button size="xs">Enable</Button>
      </AlertAction>
    </Alert>
  ),
} satisfies Fixtures
