import { getDb } from '@/lib/db/client'
import { agents, signals, runtimeSessions } from '@/lib/db/schema'
import { sql, eq } from 'drizzle-orm'

export default async function DashboardPage() {
  const db = getDb()

  const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000)

  const [[agentCount], [pendingCount], [sessionCount]] = await Promise.all([
    db.select({ count: sql<number>`count(*)::int` }).from(agents),
    db.select({ count: sql<number>`count(*)::int` }).from(signals).where(eq(signals.status, 'pending')),
    db.select({ count: sql<number>`count(*)::int` }).from(runtimeSessions).where(sql`${runtimeSessions.createdAt} >= ${oneDayAgo}`),
  ])

  return (
    <div>
      <h1 className="mb-6 text-2xl font-bold text-white">System Overview</h1>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {/* Active Agents */}
        <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
          <p className="text-sm text-zinc-400">Active Agents</p>
          <p className="mt-2 text-3xl font-semibold text-white">
            {agentCount.count}
          </p>
        </div>

        {/* Pending Signals */}
        <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
          <p className="text-sm text-zinc-400">Pending Signals</p>
          <p className="mt-2 text-3xl font-semibold text-white">
            {pendingCount.count}
          </p>
        </div>

        {/* Recent Sessions */}
        <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
          <p className="text-sm text-zinc-400">Sessions (24h)</p>
          <p className="mt-2 text-3xl font-semibold text-white">
            {sessionCount.count}
          </p>
        </div>
      </div>
    </div>
  )
}
