import { NextRequest, NextResponse } from 'next/server'
import { getDb } from '@/lib/db/client'
import { signals } from '@/lib/db/schema'

export async function POST(request: NextRequest) {
  try {
    const rawBody = await request.text()

    // Parse MSH segment to extract message type
    const lines = rawBody.split('\r').filter(Boolean)
    const mshSegment = lines.find((line) => line.startsWith('MSH'))

    let messageType = 'unknown'
    if (mshSegment) {
      const fields = mshSegment.split('|')
      // MSH-9 is the message type field (index 8 in zero-based after split)
      if (fields.length > 8) {
        messageType = fields[8]
      }
    }

    const db = getDb()

    const [inserted] = await db
      .insert(signals)
      .values({
        signalType: 'run',
        source: 'hl7v2-http',
        idempotencyKey: crypto.randomUUID(),
        payload: { raw: rawBody },
        sourceMetadata: { messageType, segmentCount: lines.length },
      })
      .returning({ id: signals.id })

    return NextResponse.json({ accepted: true, signalId: inserted.id })
  } catch (error) {
    console.error('[api/webhooks/hl7] Error processing HL7 message:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
