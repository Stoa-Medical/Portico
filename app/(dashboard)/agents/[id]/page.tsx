import Link from 'next/link'
import { notFound } from 'next/navigation'
import { getDb } from '@/lib/db/client'
import { agents, steps } from '@/lib/db/schema'
import { eq, asc } from 'drizzle-orm'

export default async function AgentDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params

  const db = getDb()

  const [agent] = await db.select().from(agents).where(eq(agents.id, id))

  if (!agent) notFound()

  const agentSteps = await db
    .select()
    .from(steps)
    .where(eq(steps.agentId, id))
    .orderBy(asc(steps.stepOrder))

  const capabilities = agent.capabilities as string[] | null

  return (
    <div>
      <Link
        href="/agents"
        className="mb-4 inline-flex items-center text-sm text-zinc-400 transition-colors hover:text-zinc-200"
      >
        &larr; Back to Agents
      </Link>

      <h1 className="mb-1 text-2xl font-bold text-white">{agent.name}</h1>
      {agent.description && (
        <p className="mb-6 text-sm text-zinc-400">{agent.description}</p>
      )}

      <div className="mb-8 rounded-lg border border-zinc-800 bg-zinc-900 p-5">
        <dl className="grid gap-4 sm:grid-cols-3">
          <div>
            <dt className="text-xs font-medium uppercase tracking-wider text-zinc-500">
              Preferred Model
            </dt>
            <dd className="mt-1 text-sm text-zinc-200">
              {agent.preferredModel ?? 'default'}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase tracking-wider text-zinc-500">
              Created
            </dt>
            <dd className="mt-1 text-sm text-zinc-200">
              {agent.createdAt.toLocaleDateString()}
            </dd>
          </div>
          <div>
            <dt className="text-xs font-medium uppercase tracking-wider text-zinc-500">
              Capabilities
            </dt>
            <dd className="mt-1 text-sm text-zinc-200">
              {capabilities && capabilities.length > 0 ? (
                <span className="flex flex-wrap gap-1">
                  {(capabilities as string[]).map((cap: string) => (
                    <span
                      key={cap}
                      className="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300"
                    >
                      {cap}
                    </span>
                  ))}
                </span>
              ) : (
                <span className="text-zinc-500">None specified</span>
              )}
            </dd>
          </div>
        </dl>
      </div>

      <h2 className="mb-4 text-lg font-semibold text-white">
        Steps ({agentSteps.length})
      </h2>

      {agentSteps.length === 0 ? (
        <p className="text-sm text-zinc-500">No steps configured.</p>
      ) : (
        <div className="overflow-hidden rounded-lg border border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-zinc-800 bg-zinc-900 text-zinc-400">
              <tr>
                <th className="w-16 px-4 py-3 font-medium">Order</th>
                <th className="px-4 py-3 font-medium">Name</th>
                <th className="px-4 py-3 font-medium">Type</th>
                <th className="px-4 py-3 font-medium">Config</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800">
              {agentSteps.map((step) => (
                <tr key={step.id} className="transition-colors hover:bg-zinc-900/50">
                  <td className="px-4 py-3 text-zinc-400">{step.stepOrder}</td>
                  <td className="px-4 py-3 text-zinc-100">{step.name}</td>
                  <td className="px-4 py-3">
                    <span className="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-300">
                      {step.stepType}
                    </span>
                  </td>
                  <td className="max-w-xs truncate px-4 py-3 font-mono text-xs text-zinc-500">
                    {step.config ? JSON.stringify(step.config).slice(0, 80) : '--'}
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
