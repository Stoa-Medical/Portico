'use server'

import { revalidatePath } from 'next/cache'
import { getSql } from '@/lib/db/client'
import type { Agent, Step } from '@/lib/db/types'

// ─── Agent actions ──────────────────────────────────────────────────────────

export async function createAgent(data: {
  name: string
  description?: string
  capabilities?: unknown
  policy?: unknown
  preferredModel?: string
}) {
  const sql = getSql()

  const [agent] = await sql<Agent[]>`
    INSERT INTO agents (name, description, capabilities, policy, preferred_model)
    VALUES (
      ${data.name},
      ${data.description ?? null},
      ${data.capabilities ? JSON.stringify(data.capabilities) : null},
      ${data.policy ? JSON.stringify(data.policy) : null},
      ${data.preferredModel ?? null}
    )
    RETURNING id, name, description, capabilities, policy,
              preferred_model AS "preferredModel",
              created_at AS "createdAt",
              updated_at AS "updatedAt"
  `

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
  const sql = getSql()

  const updates: Record<string, unknown> = { updated_at: new Date() }
  if (data.name !== undefined) updates.name = data.name
  if (data.description !== undefined) updates.description = data.description
  if (data.capabilities !== undefined) updates.capabilities = JSON.stringify(data.capabilities)
  if (data.policy !== undefined) updates.policy = JSON.stringify(data.policy)
  if (data.preferredModel !== undefined) updates.preferred_model = data.preferredModel

  const [agent] = await sql<Agent[]>`
    UPDATE agents SET ${sql(updates, ...Object.keys(updates) as (keyof typeof updates)[])}
    WHERE id = ${id}
    RETURNING id, name, description, capabilities, policy,
              preferred_model AS "preferredModel",
              created_at AS "createdAt",
              updated_at AS "updatedAt"
  `

  revalidatePath('/agents')
  return agent
}

export async function deleteAgent(id: string) {
  const sql = getSql()

  await sql`DELETE FROM agents WHERE id = ${id}`

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
  const sql = getSql()

  const [step] = await sql<Step[]>`
    INSERT INTO steps (agent_id, name, step_type, step_order, config)
    VALUES (
      ${agentId},
      ${data.name},
      ${data.stepType},
      ${data.stepOrder},
      ${data.config ? JSON.stringify(data.config) : null}
    )
    RETURNING id, agent_id AS "agentId", name,
              step_type AS "stepType",
              step_order AS "stepOrder",
              config,
              created_at AS "createdAt",
              updated_at AS "updatedAt"
  `

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
  const sql = getSql()

  const updates: Record<string, unknown> = { updated_at: new Date() }
  if (data.name !== undefined) updates.name = data.name
  if (data.stepType !== undefined) updates.step_type = data.stepType
  if (data.stepOrder !== undefined) updates.step_order = data.stepOrder
  if (data.config !== undefined) updates.config = JSON.stringify(data.config)

  const [step] = await sql<Step[]>`
    UPDATE steps SET ${sql(updates, ...Object.keys(updates) as (keyof typeof updates)[])}
    WHERE id = ${id}
    RETURNING id, agent_id AS "agentId", name,
              step_type AS "stepType",
              step_order AS "stepOrder",
              config,
              created_at AS "createdAt",
              updated_at AS "updatedAt"
  `

  revalidatePath('/agents')
  return step
}

export async function deleteStep(id: string) {
  const sql = getSql()

  await sql`DELETE FROM steps WHERE id = ${id}`

  revalidatePath('/agents')
}
