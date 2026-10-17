import { StrictMode } from "react"
import { createRoot } from "react-dom/client"

import "./index.css"
import { TooltipProvider } from "@/components/ui/tooltip"
import { fixtures } from "@/fixtures"

// `/?component=button&scenario=default&theme=dark` renders a single fixture,
// the way the Playwright runner sees it. `/` lists every fixture.
const params = new URLSearchParams(location.search)
const component = params.get("component")
const scenario = params.get("scenario")

if (params.get("theme") === "dark") {
  document.documentElement.classList.add("dark")
}

if (params.has("static")) {
  // Deterministic snapshots: iced renders with animations disabled too.
  document.documentElement.dataset.static = ""
}

function Index() {
  return (
    <ul className="p-4 text-sm">
      {Object.entries(fixtures).map(([name, scenarios]) =>
        Object.keys(scenarios).map((id) => (
          <li key={`${name}/${id}`}>
            <a className="underline" href={`?component=${name}&scenario=${id}`}>
              {name}/{id}
            </a>
          </li>
        ))
      )}
    </ul>
  )
}

function Fixture() {
  const render =
    component && scenario ? fixtures[component]?.[scenario] : undefined

  if (!render) {
    return (
      <p id="missing">
        No fixture for {component}/{scenario}
      </p>
    )
  }

  // Mirrors `fixtures::frame` on the iced side.
  return (
    <div id="frame" className="p-4">
      {render()}
    </div>
  )
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <TooltipProvider>{component ? <Fixture /> : <Index />}</TooltipProvider>
  </StrictMode>
)
