import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox"
import type { Fixtures } from "@/fixtures"

const frameworks = ["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"]

const basic = () => (
  <Combobox items={frameworks}>
    <ComboboxInput className="w-64" placeholder="Select a framework" />
    <ComboboxContent>
      <ComboboxEmpty>No items found.</ComboboxEmpty>
      <ComboboxList>
        {(item) => (
          <ComboboxItem key={item} value={item}>
            {item}
          </ComboboxItem>
        )}
      </ComboboxList>
    </ComboboxContent>
  </Combobox>
)

export default {
  basic,
  filter: basic,
  empty: basic,
  keyboard: basic,
} satisfies Fixtures
