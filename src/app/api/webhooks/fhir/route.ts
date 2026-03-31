import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db/client'
import { signals } from '@/lib/db/schema'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()

    const resourceType = body?.resourceType ?? 'Unknown'
    const bundleType = body?.type ?? null
    const entryCount = Array.isArray(body?.entry) ? body.entry.length : 0

    const db = getDb()

    const [inserted] = await db
      .insert(signals)
      .values({
        signalType: 'run',
        source: 'fhir-webhook',
        idempotencyKey: crypto.randomUUID(),
        payload: body,
        sourceMetadata: { resourceType, bundleType, entryCount },
      })
      .returning({ id: signals.id })

    return NextResponse.json({ accepted: true, signalId: inserted.id })
  } catch (error) {
    console.error('[api/webhooks/fhir] Error processing FHIR bundle:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
