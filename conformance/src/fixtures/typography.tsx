import type { Fixtures } from "@/fixtures"

// There is no registry component: these are the classes of the typography
// documentation, without the margins between blocks (a `gap` stands in).
export default {
  headings: () => (
    <div className="flex flex-col gap-4">
      <h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight">
        Taxing Laughter: The Joke Tax Chronicles
      </h1>
      <h2 className="scroll-m-20 border-b pb-2 text-3xl font-semibold tracking-tight">
        The People of the Kingdom
      </h2>
      <h3 className="scroll-m-20 text-2xl font-semibold tracking-tight">
        The Joke Tax
      </h3>
      <h4 className="scroll-m-20 text-xl font-semibold tracking-tight">
        People stopped telling jokes
      </h4>
    </div>
  ),
  text: () => (
    <div className="flex flex-col gap-4">
      <p className="leading-7">
        The king thought long and hard, and finally came up with a brilliant
        plan: he would tax the jokes in the kingdom.
      </p>
      <p className="text-xl text-muted-foreground">
        A modal dialog that interrupts the user with important content.
      </p>
      <div className="text-lg font-semibold">Are you absolutely sure?</div>
      <small className="text-sm leading-none font-medium">Email address</small>
      <p className="text-sm text-muted-foreground">Enter your email address.</p>
      <div className="flex items-center gap-1.5">
        <span className="leading-7">Run</span>
        <code className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold">
          cargo shadcn-iced add button
        </code>
        <span className="leading-7">to copy it.</span>
      </div>
    </div>
  ),
  "blockquote-list": () => (
    <div className="flex flex-col gap-4">
      <blockquote className="border-l-2 pl-6 italic">
        &quot;After all,&quot; he said, &quot;everyone enjoys a good joke, so
        it&apos;s only fair that they should pay for the privilege.&quot;
      </blockquote>
      <ul className="ml-6 list-disc [&>li]:mt-2">
        <li>1st level of puns: 5 gold coins</li>
        <li>2nd level of jokes: 10 gold coins</li>
        <li>3rd level of one-liners: 20 gold coins</li>
      </ul>
    </div>
  ),
} satisfies Fixtures
