'use server'

import { revalidatePath } from 'next/cache'
import { eq } from 'drizzle-orm'
import { getDb } from '@/lib/db/client'
import { agents, steps } from '@/lib/db/schema'

// ─── Agent actions ──────────────────────────────────────────────────────────

export async function createAgent(data: {
  name: string
  description?: string
  capabilities?: unknown
  policy?: unknown
  preferredModel?: string
}) {
  const db = getDb()

  const [agent] = await db
    .insert(agents)
    .values({
      name: data.name,
      description: data.description ?? null,
      capabilities: data.capabilities ?? null,
      policy: data.policy ?? null,
      preferredModel: data.preferredModel ?? null,
    })
    .returning()

  revalidatePath('/agents')
  return agent
}

export async function updateAgent(
  id: string,
  data: {
    name?: string
    description?: string | null
    capabilities?: unknown
    policy?: unknown
    preferredModel?: string | null
  },
) {
  const db = getDb()

  const [agent] = await db
    .update(agents)
    .set({ ...data, updatedAt: new Date() })
    .where(eq(agents.id, id))
    .returning()

  revalidatePath('/agents')
  return agent
}

export async function deleteAgent(id: string) {
  const db = getDb()

  await db.delete(agents).where(eq(agents.id, id))

  revalidatePath('/agents')
}

// ─── Step actions ───────────────────────────────────────────────────────────

export async function createStep(
  agentId: string,
  data: {
    name: string
    stepType: 'python' | 'llm' | 'transform' | 'validate' | 'fhir'
    stepOrder: number
    config?: unknown
  },
) {
  const db = getDb()

  const [step] = await db
    .insert(steps)
    .values({
      agentId,
      name: data.name,
      stepType: data.stepType,
      stepOrder: data.stepOrder,
      config: data.config ?? null,
    })
    .returning()

  revalidatePath('/agents')
  return step
}

export async function updateStep(
  id: string,
  data: {
    name?: string
    stepType?: 'python' | 'llm' | 'transform' | 'validate' | 'fhir'
    stepOrder?: number
    config?: unknown
  },
) {
  const db = getDb()

  const [step] = await db
    .update(steps)
    .set({ ...data, updatedAt: new Date() })
    .where(eq(steps.id, id))
    .returning()

  revalidatePath('/agents')
  return step
}

export async function deleteStep(id: string) {
  const db = getDb()

  await db.delete(steps).where(eq(steps.id, id))

  revalidatePath('/agents')
}
