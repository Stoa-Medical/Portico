import Link from "next/link"
import { notFound } from "next/navigation"
import { ArrowLeft, Bot, GitBranch } from "lucide-react"
import { formatDistanceToNow } from "date-fns"

import { getSql } from "@/lib/db/client"
import type { Agent, RecentSessionRow, Step } from "@/lib/db/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { DeleteAgentButton } from "./agent-actions"

export default async function AgentDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const sql = getSql()

  const [agent] = await sql<Agent[]>`
    SELECT id, name, description, capabilities, policy,
           preferred_model AS "preferredModel",
           created_at AS "createdAt",
           updated_at AS "updatedAt"
    FROM agents WHERE id = ${id}
  `

  if (!agent) notFound()

  const [steps, recentSessions] = await Promise.all([
    sql<Step[]>`
      SELECT id, agent_id AS "agentId", name,
             step_type AS "stepType",
             step_order AS "stepOrder",
             config,
             created_at AS "createdAt",
             updated_at AS "updatedAt"
      FROM steps WHERE agent_id = ${id} ORDER BY step_order ASC
    `,
    sql<RecentSessionRow[]>`
      SELECT
        rs.id,
        rs.agent_id AS "agentId",
        ${agent.name}::text AS "agentName",
        rs.status,
        rs.total_execution_ms AS "totalExecutionMs",
        rs.data_quality_score AS "dataQualityScore",
        rs.created_at AS "createdAt",
        rs.completed_at AS "completedAt"
      FROM runtime_sessions rs
      WHERE rs.agent_id = ${id}
      ORDER BY rs.created_at DESC
      LIMIT 10
    `,
  ])

  const capabilities = Array.isArray(agent.capabilities)
    ? (agent.capabilities as string[])
    : []

  return (
    <div className="p-6">
      <Link
        href="/agents"
        className="mb-4 inline-flex items-center text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ArrowLeft className="mr-1 h-4 w-4" />
        Back to Agents
      </Link>

      <div className="mb-8 flex items-start justify-between gap-4">
        <div className="flex items-start gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/20">
            <Bot className="h-6 w-6 text-primary" />
          </div>
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">{agent.name}</h1>
            {agent.description && (
              <p className="mt-1 text-sm text-muted-foreground">{agent.description}</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" asChild>
            <Link href={`/workflows/${agent.id}`}>
              <GitBranch className="mr-2 h-4 w-4" />
              Edit steps
            </Link>
          </Button>
          <DeleteAgentButton id={agent.id} />
        </div>
      </div>

      <div className="mb-8 grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Preferred Model</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm">{agent.preferredModel ?? "default"}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Created</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm">{new Date(agent.createdAt).toLocaleDateString()}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">Capabilities</CardTitle>
          </CardHeader>
          <CardContent>
            {capabilities.length > 0 ? (
              <div className="flex flex-wrap gap-1">
                {capabilities.map((cap) => (
                  <Badge key={cap} variant="secondary" className="text-xs">
                    {cap}
                  </Badge>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">None specified</p>
            )}
          </CardContent>
        </Card>
      </div>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Steps</CardTitle>
          <CardDescription>
            {steps.length} {steps.length === 1 ? "step" : "steps"} configured.{" "}
            <Link href={`/workflows/${agent.id}`} className="text-primary hover:underline">
              Open step editor →
            </Link>
          </CardDescription>
        </CardHeader>
        <CardContent>
          {steps.length === 0 ? (
            <p className="py-6 text-center text-sm text-muted-foreground">
              No steps configured.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-16">#</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Config</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {steps.map((step) => (
                  <TableRow key={step.id}>
                    <TableCell className="text-muted-foreground">{step.stepOrder}</TableCell>
                    <TableCell>{step.name}</TableCell>
                    <TableCell>
                      <Badge variant="secondary" className="text-xs">
                        {step.stepType}
                      </Badge>
                    </TableCell>
                    <TableCell className="max-w-xs truncate font-mono text-xs text-muted-foreground">
                      {step.config ? JSON.stringify(step.config).slice(0, 80) : "—"}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Sessions</CardTitle>
          <CardDescription>Last 10 runtime sessions</CardDescription>
        </CardHeader>
        <CardContent>
          {recentSessions.length === 0 ? (
            <p className="py-6 text-center text-sm text-muted-foreground">No sessions yet</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Status</TableHead>
                  <TableHead>Duration</TableHead>
                  <TableHead>Quality</TableHead>
                  <TableHead>When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentSessions.map((session) => (
                  <TableRow key={session.id}>
                    <TableCell>
                      <Badge
                        variant={session.status === "completed" ? "default" : "secondary"}
                        className="text-xs"
                      >
                        {session.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-sm">
                      {session.totalExecutionMs != null ? `${session.totalExecutionMs}ms` : "—"}
                    </TableCell>
                    <TableCell className="text-sm">
                      {session.dataQualityScore != null
                        ? session.dataQualityScore.toFixed(2)
                        : "—"}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {formatDistanceToNow(new Date(session.createdAt), { addSuffix: true })}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
