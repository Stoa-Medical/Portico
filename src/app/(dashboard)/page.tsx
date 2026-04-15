import { Activity, Bot, Database, GitBranch, Radio } from "lucide-react"
import Link from "next/link"
import { formatDistanceToNow } from "date-fns"

import { getSql } from "@/lib/db/client"
import { getRecentSessions, getSignalCounts } from "@/lib/analytics/queries"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

const ONE_DAY_MS = 24 * 60 * 60 * 1000

function statusToTone(status: string): "success" | "warning" | "destructive" {
  if (status === "completed") return "success"
  if (status === "failed" || status === "cancelled") return "destructive"
  return "warning"
}

export default async function OverviewPage() {
  const sql = getSql()
  const since = new Date(Date.now() - ONE_DAY_MS)

  const [
    [integrationsRow],
    [agentsRow],
    [activeAgentsRow],
    signalCounts,
    recentSessions,
  ] = await Promise.all([
    sql<[{ count: number }]>`SELECT count(*)::int AS count FROM integrations`,
    sql<[{ count: number }]>`SELECT count(*)::int AS count FROM agents`,
    sql<[{ count: number }]>`
      SELECT count(DISTINCT agent_id)::int AS count
      FROM runtime_sessions
      WHERE created_at >= ${since} AND agent_id IS NOT NULL
    `,
    getSignalCounts(since),
    getRecentSessions(5),
  ])

  const messages24h = signalCounts.reduce((sum, row) => sum + row.count, 0)

  const stats = [
    { title: "Integrations", value: integrationsRow.count, icon: Radio, href: "/integrations" },
    { title: "Workflows", value: agentsRow.count, icon: GitBranch, href: "/workflows" },
    { title: "Active Agents (24h)", value: activeAgentsRow.count, icon: Bot, href: "/agents" },
    { title: "Messages (24h)", value: messages24h, icon: Activity, href: "/analytics" },
  ]

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">Overview</h1>
        <p className="text-muted-foreground">Monitor your integration engine performance</p>
      </div>

      <div className="mb-8 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Link key={stat.title} href={stat.href}>
            <Card className="cursor-pointer transition-colors hover:bg-accent/50">
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  {stat.title}
                </CardTitle>
                <stat.icon className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <span className="text-2xl font-bold">{stat.value.toLocaleString()}</span>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Latest agent runtime sessions</CardDescription>
          </CardHeader>
          <CardContent>
            {recentSessions.length === 0 ? (
              <p className="py-6 text-center text-sm text-muted-foreground">No sessions yet</p>
            ) : (
              <div className="space-y-4">
                {recentSessions.map((session) => {
                  const tone = statusToTone(session.status)
                  return (
                    <div
                      key={session.id}
                      className="flex items-center justify-between border-b border-border py-2 last:border-0"
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className={`h-2 w-2 rounded-full ${
                            tone === "success"
                              ? "bg-success"
                              : tone === "warning"
                                ? "bg-warning"
                                : "bg-destructive"
                          }`}
                        />
                        <div>
                          <p className="text-sm font-medium">
                            {session.agentName ?? "Unknown agent"}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            Session {session.id.slice(0, 8)} · {session.status}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {session.totalExecutionMs != null && (
                          <Badge variant="secondary" className="text-xs">
                            {session.totalExecutionMs}ms
                          </Badge>
                        )}
                        <span className="text-xs text-muted-foreground">
                          {formatDistanceToNow(new Date(session.createdAt), { addSuffix: true })}
                        </span>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks and shortcuts</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3">
              <Link href="/agents">
                <Button variant="outline" className="w-full justify-start gap-2 bg-transparent">
                  <Bot className="h-4 w-4" />
                  Manage Agents
                </Button>
              </Link>
              <Link href="/workflows">
                <Button variant="outline" className="w-full justify-start gap-2 bg-transparent">
                  <GitBranch className="h-4 w-4" />
                  Build Workflow
                </Button>
              </Link>
              <Link href="/integrations">
                <Button variant="outline" className="w-full justify-start gap-2 bg-transparent">
                  <Radio className="h-4 w-4" />
                  Configure Integration
                </Button>
              </Link>
              <Link href="/mappings">
                <Button variant="outline" className="w-full justify-start gap-2 bg-transparent">
                  <Database className="h-4 w-4" />
                  Create Mapping
                </Button>
              </Link>
              <Link href="/analytics">
                <Button variant="outline" className="w-full justify-start gap-2 bg-transparent">
                  <Activity className="h-4 w-4" />
                  View Analytics
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
