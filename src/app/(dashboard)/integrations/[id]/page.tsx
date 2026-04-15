import Link from "next/link"
import { notFound } from "next/navigation"
import { ArrowLeft, Radio } from "lucide-react"
import { formatDistanceToNow } from "date-fns"

import { getSql } from "@/lib/db/client"
import type { Integration } from "@/lib/agents/types"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export default async function IntegrationDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const sql = getSql()

  const [integration] = await sql<Integration[]>`
    SELECT
      id, name,
      connection_type AS "connectionType",
      direction,
      config,
      agent_id AS "agentId",
      status,
      last_seen_at AS "lastSeenAt",
      created_at AS "createdAt",
      updated_at AS "updatedAt"
    FROM integrations WHERE id = ${id}
  `

  if (!integration) notFound()

  let linkedAgentName: string | null = null
  if (integration.agentId) {
    const [row] = await sql<{ name: string }[]>`
      SELECT name FROM agents WHERE id = ${integration.agentId}
    `
    linkedAgentName = row?.name ?? null
  }

  return (
    <div className="p-6">
      <Link
        href="/integrations"
        className="mb-4 inline-flex items-center text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ArrowLeft className="mr-1 h-4 w-4" />
        Back to Integrations
      </Link>

      <div className="mb-8 flex items-start gap-3">
        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-sky/20">
          <Radio className="h-6 w-6 text-sky" />
        </div>
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{integration.name}</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            {integration.connectionType} · {integration.direction}
          </p>
        </div>
      </div>

      <div className="mb-8 grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Status</CardTitle>
          </CardHeader>
          <CardContent>
            <Badge
              variant={integration.status === "active" ? "default" : "secondary"}
              className="text-xs"
            >
              {integration.status}
            </Badge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Direction</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm capitalize">{integration.direction}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Last Seen</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm">
              {integration.lastSeenAt
                ? formatDistanceToNow(new Date(integration.lastSeenAt), { addSuffix: true })
                : "Never"}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Linked Agent</CardTitle>
          </CardHeader>
          <CardContent>
            {linkedAgentName && integration.agentId ? (
              <Link
                href={`/agents/${integration.agentId}`}
                className="text-sm text-primary hover:underline"
              >
                {linkedAgentName}
              </Link>
            ) : (
              <p className="text-sm text-muted-foreground">None</p>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          {integration.config ? (
            <pre className="overflow-auto rounded bg-muted p-4 font-mono text-xs">
              {JSON.stringify(integration.config, null, 2)}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground">No configuration set</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
