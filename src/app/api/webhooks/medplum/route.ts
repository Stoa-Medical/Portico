import { NextRequest, NextResponse } from 'next/server'
import { getSql } from '@/lib/db/client'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()

    const subscriptionId = body?.subscription?.reference ?? body?.id ?? null
    const notificationType = body?.type ?? body?.resourceType ?? 'unknown'

    const sql = getSql()

    await sql`
      INSERT INTO signals (signal_type, source, idempotency_key, payload, source_metadata)
      VALUES ('run', 'medplum-subscription', ${crypto.randomUUID()}, ${JSON.stringify(body)}, ${JSON.stringify({ subscriptionId, notificationType })})
    `

    return NextResponse.json({ accepted: true })
  } catch (error) {
    console.error('[api/webhooks/medplum] Error processing notification:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
