import Link from "next/link"
import { notFound } from "next/navigation"
import { ArrowLeft, GitBranch } from "lucide-react"

import { getSql } from "@/lib/db/client"
import type { Agent, Step } from "@/lib/db/types"
import { StepEditor } from "./step-editor"

export default async function WorkflowDetailPage({
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

  const steps = await sql<Step[]>`
    SELECT id, agent_id AS "agentId", name,
           step_type AS "stepType",
           step_order AS "stepOrder",
           config,
           created_at AS "createdAt",
           updated_at AS "updatedAt"
    FROM steps WHERE agent_id = ${id} ORDER BY step_order ASC
  `

  return (
    <div className="p-6">
      <Link
        href="/workflows"
        className="mb-4 inline-flex items-center text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ArrowLeft className="mr-1 h-4 w-4" />
        Back to Workflows
      </Link>

      <div className="mb-8 flex items-start gap-3">
        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-accent/30">
          <GitBranch className="h-6 w-6 text-accent" />
        </div>
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{agent.name}</h1>
          {agent.description && (
            <p className="mt-1 text-sm text-muted-foreground">{agent.description}</p>
          )}
        </div>
      </div>

      <StepEditor agentId={agent.id} steps={steps} />
    </div>
  )
}
