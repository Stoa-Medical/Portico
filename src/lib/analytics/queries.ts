import { getSql } from '@/lib/db/client'
import type { SignalCountRow, AgentPerformanceRow, RecentSessionRow } from '@/lib/db/types'

export async function getSignalCounts(since: Date) {
  const sql = getSql()

  return await sql<SignalCountRow[]>`
    SELECT status, count(*)::int AS count
    FROM signals
    WHERE created_at >= ${since}
    GROUP BY status
  `
}

export async function getAgentPerformance(since: Date) {
  const sql = getSql()

  const rows = await sql<AgentPerformanceRow[]>`
    SELECT
      rs.agent_id AS "agentId",
      a.name AS "agentName",
      avg(rs.total_execution_ms)::int AS "avgExecutionMs",
      count(*) FILTER (WHERE rs.status = 'completed')::int AS "successCount",
      count(*)::int AS "totalCount"
    FROM runtime_sessions rs
    LEFT JOIN agents a ON rs.agent_id = a.id
    WHERE rs.created_at >= ${since}
    GROUP BY rs.agent_id, a.name
  `

  return rows.map((row) => ({
    ...row,
    successRate: row.totalCount > 0 ? row.successCount / row.totalCount : 0,
  }))
}

export async function getRecentSessions(limit: number) {
  const sql = getSql()

  return await sql<RecentSessionRow[]>`
    SELECT
      rs.id,
      rs.agent_id AS "agentId",
      a.name AS "agentName",
      rs.status,
      rs.total_execution_ms AS "totalExecutionMs",
      rs.data_quality_score AS "dataQualityScore",
      rs.created_at AS "createdAt",
      rs.completed_at AS "completedAt"
    FROM runtime_sessions rs
    LEFT JOIN agents a ON rs.agent_id = a.id
    ORDER BY rs.created_at DESC
    LIMIT ${limit}
  `
}

export type SignalVolumeBucket = {
  bucket: Date
  status: string
  count: number
}

export async function getSignalVolumeByDay(since: Date) {
  const sql = getSql()

  return await sql<SignalVolumeBucket[]>`
    SELECT
      date_trunc('day', created_at) AS bucket,
      status,
      count(*)::int AS count
    FROM signals
    WHERE created_at >= ${since}
    GROUP BY bucket, status
    ORDER BY bucket ASC
  `
}
