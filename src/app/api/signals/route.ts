import { NextRequest, NextResponse } from 'next/server'
import { getSql } from '@/lib/db/client'

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

    const sql = getSql()
    const key = idempotencyKey || crypto.randomUUID()

    const [existing] = await sql<[{ id: string; status: string }?]>`
      SELECT id, status FROM signals WHERE idempotency_key = ${key}
    `

    if (existing) {
      return NextResponse.json(
        { id: existing.id, status: existing.status },
        { status: 200 },
      )
    }

    const [inserted] = await sql<[{ id: string }]>`
      INSERT INTO signals (signal_type, source, agent_id, payload, source_metadata, idempotency_key)
      VALUES (${signalType}, ${source}, ${agentId}, ${payload ?? null}, ${sourceMetadata ?? null}, ${key})
      RETURNING id
    `

    return NextResponse.json({ id: inserted.id, status: 'pending' }, { status: 201 })
  } catch (error) {
    console.error('[api/signals] Error creating signal:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
