import type { Fixtures } from "@/fixtures"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"

const product = () => (
  <Accordion defaultValue={["item-1"]} className="max-w-lg">
    <AccordionItem value="item-1">
      <AccordionTrigger>Product Information</AccordionTrigger>
      <AccordionContent>
        We offer standard and express shipping.
      </AccordionContent>
    </AccordionItem>
    <AccordionItem value="item-2">
      <AccordionTrigger>Return Policy</AccordionTrigger>
      <AccordionContent>Returns are accepted within 30 days.</AccordionContent>
    </AccordionItem>
    <AccordionItem value="item-3">
      <AccordionTrigger>Contact Us</AccordionTrigger>
      <AccordionContent>Reach us by email or chat.</AccordionContent>
    </AccordionItem>
  </Accordion>
)

export default {
  default: product,
  keyboard: product,
  multiple: () => (
    <Accordion multiple defaultValue={["item-1"]} className="max-w-lg">
      <AccordionItem value="item-1">
        <AccordionTrigger>Product Information</AccordionTrigger>
        <AccordionContent>Our products are made to last.</AccordionContent>
      </AccordionItem>
      <AccordionItem value="item-2">
        <AccordionTrigger>Return Policy</AccordionTrigger>
        <AccordionContent>
          Returns are accepted within 30 days.
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  ),
  disabled: () => (
    <Accordion className="max-w-lg">
      <AccordionItem value="item-1">
        <AccordionTrigger>Account history</AccordionTrigger>
        <AccordionContent>View your complete history.</AccordionContent>
      </AccordionItem>
      <AccordionItem value="item-2" disabled>
        <AccordionTrigger>Premium feature</AccordionTrigger>
        <AccordionContent>Upgrade your plan to see this.</AccordionContent>
      </AccordionItem>
      <AccordionItem value="item-3">
        <AccordionTrigger>Update your email</AccordionTrigger>
        <AccordionContent>Change it in your settings.</AccordionContent>
      </AccordionItem>
    </Accordion>
  ),
  borders: () => (
    <Accordion
      defaultValue={["billing"]}
      className="max-w-lg rounded-lg border"
    >
      <AccordionItem value="billing" className="border-b px-4 last:border-b-0">
        <AccordionTrigger>How does billing work?</AccordionTrigger>
        <AccordionContent>Monthly and annual plans.</AccordionContent>
      </AccordionItem>
      <AccordionItem value="security" className="border-b px-4 last:border-b-0">
        <AccordionTrigger>Is my data secure?</AccordionTrigger>
        <AccordionContent>Yes. Everything is encrypted.</AccordionContent>
      </AccordionItem>
      <AccordionItem
        value="integrations"
        className="border-b px-4 last:border-b-0"
      >
        <AccordionTrigger>Which integrations exist?</AccordionTrigger>
        <AccordionContent>Over 500 popular tools.</AccordionContent>
      </AccordionItem>
    </Accordion>
  ),
} satisfies Fixtures
