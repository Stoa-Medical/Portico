import Link from "next/link"
import { GitBranch } from "lucide-react"

import { getSql } from "@/lib/db/client"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

type WorkflowRow = {
  id: string
  name: string
  description: string | null
  stepCount: number
}

export default async function WorkflowsPage() {
  const sql = getSql()

  const workflows = await sql<WorkflowRow[]>`
    SELECT
      a.id,
      a.name,
      a.description,
      COALESCE(s.step_count, 0)::int AS "stepCount"
    FROM agents a
    LEFT JOIN (
      SELECT agent_id, count(*)::int AS step_count
      FROM steps
      GROUP BY agent_id
    ) s ON s.agent_id = a.id
    ORDER BY a.created_at DESC
  `

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">Workflows</h1>
        <p className="text-muted-foreground">
          Each workflow is an agent with an ordered sequence of steps
        </p>
      </div>

      {workflows.length === 0 ? (
        <Card>
          <CardContent className="py-16 text-center">
            <GitBranch className="mx-auto mb-4 h-10 w-10 text-muted-foreground" />
            <p className="text-lg font-medium">No workflows yet</p>
            <p className="mt-2 text-sm text-muted-foreground">
              Create an agent first, then add steps to define a workflow.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {workflows.map((w) => (
            <Link key={w.id} href={`/workflows/${w.id}`}>
              <Card className="h-full cursor-pointer transition-colors hover:border-primary/50">
                <CardHeader>
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/30">
                      <GitBranch className="h-5 w-5 text-accent" />
                    </div>
                    <div>
                      <CardTitle className="text-base">{w.name}</CardTitle>
                      <CardDescription className="text-xs">
                        {w.stepCount} {w.stepCount === 1 ? "step" : "steps"}
                      </CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  {w.description ? (
                    <p className="line-clamp-2 text-sm text-muted-foreground">{w.description}</p>
                  ) : (
                    <Badge variant="secondary" className="text-xs">
                      No description
                    </Badge>
                  )}
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
