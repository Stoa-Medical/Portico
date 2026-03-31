import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db/client'
import { signals } from '@/lib/db/schema'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()

    // Medplum subscription notifications follow the FHIR SubscriptionStatus pattern
    const subscriptionId = body?.subscription?.reference ?? body?.id ?? null
    const notificationType = body?.type ?? body?.resourceType ?? 'unknown'

    const db = getDb()

    await db.insert(signals).values({
      signalType: 'run',
      source: 'medplum-subscription',
      idempotencyKey: crypto.randomUUID(),
      payload: body,
      sourceMetadata: { subscriptionId, notificationType },
    })

    return NextResponse.json({ accepted: true })
  } catch (error) {
    console.error('[api/webhooks/medplum] Error processing notification:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
