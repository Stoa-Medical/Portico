import { NextRequest, NextResponse } from 'next/server'
import { getSql } from '@/lib/db/client'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()

    const resourceType = body?.resourceType ?? 'Unknown'
    const bundleType = body?.type ?? null
    const entryCount = Array.isArray(body?.entry) ? body.entry.length : 0

    const sql = getSql()

    const [inserted] = await sql<[{ id: string }]>`
      INSERT INTO signals (signal_type, source, idempotency_key, payload, source_metadata)
      VALUES ('run', 'fhir-webhook', ${crypto.randomUUID()}, ${JSON.stringify(body)}, ${JSON.stringify({ resourceType, bundleType, entryCount })})
      RETURNING id
    `

    return NextResponse.json({ accepted: true, signalId: inserted.id })
  } catch (error) {
    console.error('[api/webhooks/fhir] Error processing FHIR bundle:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
