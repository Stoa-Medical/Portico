import { NextRequest, NextResponse } from 'next/server'
import { eq } from 'drizzle-orm'
import { getDb } from '@/lib/db/client'
import { signals } from '@/lib/db/schema'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { signalType, source, agentId, payload, sourceMetadata, idempotencyKey } = body

    if (!signalType || !source || !agentId) {
      return NextResponse.json(
        { error: 'signalType, source, and agentId are required' },
        { status: 400 },
      )
    }

    const db = getDb()
    const key = idempotencyKey ?? crypto.randomUUID()

    // Check for existing signal with this idempotency key
    const [existing] = await db
      .select({ id: signals.id, status: signals.status })
      .from(signals)
      .where(eq(signals.idempotencyKey, key))

    if (existing) {
      return NextResponse.json(
        { id: existing.id, status: existing.status },
        { status: 200 },
      )
    }

    const [inserted] = await db
      .insert(signals)
      .values({
        signalType,
        source,
        agentId,
        payload: payload ?? null,
        sourceMetadata: sourceMetadata ?? null,
        idempotencyKey: key,
      })
      .returning({ id: signals.id })

    return NextResponse.json({ id: inserted.id, status: 'pending' }, { status: 201 })
  } catch (error) {
    console.error('[api/signals] Error creating signal:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
