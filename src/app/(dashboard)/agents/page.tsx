import Link from 'next/link'
import { getDb } from '@/lib/db/client'
import { agents } from '@/lib/db/schema'
import { desc } from 'drizzle-orm'

export default async function AgentsPage() {
  const db = getDb()
  const allAgents = await db
    .select()
    .from(agents)
    .orderBy(desc(agents.createdAt))

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Agents</h1>
          <p className="mt-1 text-sm text-zinc-400">
            {allAgents.length} {allAgents.length === 1 ? 'agent' : 'agents'} configured
          </p>
        </div>
        <Link
          href="/agents/new"
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 transition-colors hover:bg-white"
        >
          New Agent
        </Link>
      </div>

      {allAgents.length === 0 ? (
        <div className="rounded-lg border border-zinc-800 bg-zinc-900 px-6 py-16 text-center">
          <p className="text-lg font-medium text-zinc-300">No agents configured yet</p>
          <p className="mt-2 text-sm text-zinc-500">
            Create your first agent to start processing signals.
          </p>
          <Link
            href="/agents/new"
            className="mt-6 inline-block rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 transition-colors hover:bg-white"
          >
            Create Agent
          </Link>
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-zinc-800 bg-zinc-900 text-zinc-400">
              <tr>
                <th className="px-4 py-3 font-medium">Name</th>
                <th className="px-4 py-3 font-medium">Preferred Model</th>
                <th className="px-4 py-3 font-medium">Created</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800">
              {allAgents.map((agent) => (
                <tr key={agent.id} className="transition-colors hover:bg-zinc-900/50">
                  <td className="px-4 py-3">
                    <Link
                      href={`/agents/${agent.id}`}
                      className="text-zinc-100 underline-offset-4 hover:underline"
                    >
                      {agent.name}
                    </Link>
                  </td>
                  <td className="px-4 py-3 text-zinc-400">
                    {agent.preferredModel ?? 'default'}
                  </td>
                  <td className="px-4 py-3 text-zinc-400">
                    {agent.createdAt.toLocaleDateString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
