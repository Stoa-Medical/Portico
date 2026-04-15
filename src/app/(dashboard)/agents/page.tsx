import { getSql } from "@/lib/db/client"
import { AgentsList, type AgentRow } from "./agents-list"

export default async function AgentsPage() {
  const sql = getSql()

  const agents = await sql<AgentRow[]>`
    SELECT
      a.id,
      a.name,
      a.description,
      a.preferred_model AS "preferredModel",
      a.created_at AS "createdAt",
      COALESCE(s.step_count, 0)::int AS "stepCount"
    FROM agents a
    LEFT JOIN (
      SELECT agent_id, count(*)::int AS step_count
      FROM steps
      GROUP BY agent_id
    ) s ON s.agent_id = a.id
    ORDER BY a.created_at DESC
  `

  return <AgentsList agents={agents} />
}
