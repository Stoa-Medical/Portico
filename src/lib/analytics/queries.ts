import { getDb } from '@/lib/db/client'
import { signals, runtimeSessions, agents } from '@/lib/db/schema'
import { sql, gte, eq, desc } from 'drizzle-orm'

export async function getSignalCounts(since: Date) {
  const db = getDb()

  const rows = await db
    .select({
      status: signals.status,
      count: sql<number>`count(*)::int`,
    })
    .from(signals)
    .where(gte(signals.createdAt, since))
    .groupBy(signals.status)

  return rows
}

export async function getAgentPerformance(since: Date) {
  const db = getDb()

  const rows = await db
    .select({
      agentId: runtimeSessions.agentId,
      agentName: agents.name,
      avgExecutionMs: sql<number>`avg(${runtimeSessions.totalExecutionMs})::int`,
      successCount: sql<number>`count(*) filter (where ${runtimeSessions.status} = 'completed')::int`,
      totalCount: sql<number>`count(*)::int`,
    })
    .from(runtimeSessions)
    .leftJoin(agents, eq(runtimeSessions.agentId, agents.id))
    .where(gte(runtimeSessions.createdAt, since))
    .groupBy(runtimeSessions.agentId, agents.name)

  return rows.map((row) => ({
    ...row,
    successRate: row.totalCount > 0 ? row.successCount / row.totalCount : 0,
  }))
}

export async function getRecentSessions(limit: number) {
  const db = getDb()

  const rows = await db
    .select({
      id: runtimeSessions.id,
      agentId: runtimeSessions.agentId,
      agentName: agents.name,
      status: runtimeSessions.status,
      totalExecutionMs: runtimeSessions.totalExecutionMs,
      dataQualityScore: runtimeSessions.dataQualityScore,
      createdAt: runtimeSessions.createdAt,
      completedAt: runtimeSessions.completedAt,
    })
    .from(runtimeSessions)
    .leftJoin(agents, eq(runtimeSessions.agentId, agents.id))
    .orderBy(desc(runtimeSessions.createdAt))
    .limit(limit)

  return rows
}
