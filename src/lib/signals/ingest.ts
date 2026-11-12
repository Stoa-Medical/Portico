import { createHash, timingSafeEqual } from 'node:crypto'
import { NextResponse } from 'next/server'
import type { JSONValue } from 'postgres'
import { getSql } from '@/lib/db/client'

export type WebhookConnectionType = 'hl7v2-http' | 'rest-webhook' | 'fhir-subscription'

export interface InboundMessage {
  source: string
  /** The sender's own id for this message. A retry with the same id returns the first signal. */
  messageId: string | null
  payload: JSONValue
  sourceMetadata: JSONValue
}

type IngestResult =
  | { ok: true; signalId: string; duplicate: boolean }
  | { ok: false; status: 401 | 404 | 409; error: string }

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function bearerToken(request: Request): string | null {
  const match = /^Bearer\s+(.+)$/i.exec(request.headers.get('authorization') ?? '')
  return match ? match[1].trim() : null
}

function secretMatches(presented: string | null, expectedSha256Hex: unknown): boolean {
  if (!presented || typeof expectedSha256Hex !== 'string') return false
  const expected = Buffer.from(expectedSha256Hex, 'hex')
  const actual = createHash('sha256').update(presented).digest()
  return expected.length === actual.length && timingSafeEqual(expected, actual)
}

/**
 * Turns a webhook request into a signal for the agent its integration points at.
 * The URL names the integration and the bearer token proves the sender is that
 * integration, whose config stores only the token's SHA-256.
 */
export async function ingestForIntegration(
  request: Request,
  integrationId: string,
  connectionType: WebhookConnectionType,
  message: InboundMessage,
): Promise<IngestResult> {
  if (!UUID.test(integrationId)) {
    return { ok: false, status: 404, error: 'Unknown integration' }
  }

  const sql = getSql()
  const [integration] = await sql<
    {
      agentId: string | null
      status: string
      direction: string
      secretSha256: string | null
    }[]
  >`
    SELECT
      agent_id AS "agentId",
      status,
      direction,
      config->>'webhook_secret_sha256' AS "secretSha256"
    FROM integrations
    WHERE id = ${integrationId} AND connection_type = ${connectionType}
  `

  if (!integration) {
    return { ok: false, status: 404, error: 'Unknown integration' }
  }
  if (!secretMatches(bearerToken(request), integration.secretSha256)) {
    return { ok: false, status: 401, error: 'Invalid or missing bearer token' }
  }
  if (integration.direction !== 'inbound' || integration.status !== 'active') {
    return { ok: false, status: 409, error: 'Integration is not an active inbound connection' }
  }
  if (!integration.agentId) {
    return { ok: false, status: 409, error: 'Integration has no linked agent' }
  }

  const idempotencyKey = `${integrationId}:${message.messageId ?? crypto.randomUUID()}`

  const [inserted] = await sql<{ id: string }[]>`
    INSERT INTO signals (signal_type, source, agent_id, idempotency_key, payload, source_metadata)
    VALUES (
      'run', ${message.source}, ${integration.agentId}, ${idempotencyKey},
      ${sql.json(message.payload)}, ${sql.json(message.sourceMetadata)}
    )
    ON CONFLICT (idempotency_key) DO NOTHING
    RETURNING id
  `

  await sql`UPDATE integrations SET last_seen_at = now() WHERE id = ${integrationId}`

  if (inserted) {
    return { ok: true, signalId: inserted.id, duplicate: false }
  }

  const [existing] = await sql<{ id: string }[]>`
    SELECT id FROM signals WHERE idempotency_key = ${idempotencyKey}
  `
  return { ok: true, signalId: existing.id, duplicate: true }
}

export function ingestResponse(result: IngestResult): NextResponse {
  if (!result.ok) {
    return NextResponse.json({ error: result.error }, { status: result.status })
  }
  return NextResponse.json({
    accepted: true,
    signalId: result.signalId,
    duplicate: result.duplicate,
  })
}

/**
 * A FHIR resource names one message only when it carries a version. Without
 * one, two different updates to the same resource would look like a retry.
 */
export function fhirMessageId(resource: {
  resourceType?: unknown
  id?: unknown
  meta?: { versionId?: unknown }
}): string | null {
  const { resourceType, id } = resource
  const versionId = resource.meta?.versionId
  if (typeof resourceType !== 'string' || typeof id !== 'string' || typeof versionId !== 'string') {
    return null
  }
  return `${resourceType}/${id}/_history/${versionId}`
}
