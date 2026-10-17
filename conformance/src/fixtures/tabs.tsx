import { AppWindowIcon, CodeIcon } from "lucide-react"

import type { Fixtures } from "@/fixtures"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"

const account = () => (
  <Tabs defaultValue="account" className="w-[400px]">
    <TabsList>
      <TabsTrigger value="account">Account</TabsTrigger>
      <TabsTrigger value="password">Password</TabsTrigger>
    </TabsList>
    <TabsContent value="account">
      <p className="text-muted-foreground">
        Make changes to your account here.
      </p>
    </TabsContent>
    <TabsContent value="password">
      <p className="text-muted-foreground">Change your password here.</p>
    </TabsContent>
  </Tabs>
)

export default {
  default: account,
  keyboard: account,
  line: () => (
    <Tabs defaultValue="overview">
      <TabsList variant="line">
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="analytics">Analytics</TabsTrigger>
        <TabsTrigger value="reports">Reports</TabsTrigger>
      </TabsList>
      <TabsContent value="overview">
        <p className="text-muted-foreground">View your key metrics.</p>
      </TabsContent>
      <TabsContent value="analytics">
        <p className="text-muted-foreground">
          Track performance and engagement.
        </p>
      </TabsContent>
      <TabsContent value="reports">
        <p className="text-muted-foreground">Generate your reports.</p>
      </TabsContent>
    </Tabs>
  ),
  vertical: () => (
    <Tabs defaultValue="account" orientation="vertical">
      <TabsList>
        <TabsTrigger value="account">Account</TabsTrigger>
        <TabsTrigger value="password">Password</TabsTrigger>
        <TabsTrigger value="notifications">Notifications</TabsTrigger>
      </TabsList>
      <TabsContent value="account">
        <p className="text-muted-foreground">Update your account details.</p>
      </TabsContent>
      <TabsContent value="password">
        <p className="text-muted-foreground">Change your password here.</p>
      </TabsContent>
      <TabsContent value="notifications">
        <p className="text-muted-foreground">
          Choose what you get notified about.
        </p>
      </TabsContent>
    </Tabs>
  ),
  disabled: () => (
    <Tabs defaultValue="home">
      <TabsList>
        <TabsTrigger value="home">Home</TabsTrigger>
        <TabsTrigger value="settings" disabled>
          Disabled
        </TabsTrigger>
      </TabsList>
      <TabsContent value="home">
        <p className="text-muted-foreground">Home content.</p>
      </TabsContent>
      <TabsContent value="settings">
        <p className="text-muted-foreground">Disabled content.</p>
      </TabsContent>
    </Tabs>
  ),
  icons: () => (
    <Tabs defaultValue="preview">
      <TabsList>
        <TabsTrigger value="preview">
          <AppWindowIcon />
          Preview
        </TabsTrigger>
        <TabsTrigger value="code">
          <CodeIcon />
          Code
        </TabsTrigger>
      </TabsList>
      <TabsContent value="preview">
        <p className="text-muted-foreground">A live preview.</p>
      </TabsContent>
      <TabsContent value="code">
        <p className="text-muted-foreground">Source code.</p>
      </TabsContent>
    </Tabs>
  ),
} satisfies Fixtures
