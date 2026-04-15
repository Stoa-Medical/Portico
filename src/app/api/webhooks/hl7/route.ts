import { NextRequest, NextResponse } from 'next/server'
import { getSql } from '@/lib/db/client'

export async function POST(request: NextRequest) {
  try {
    const rawBody = await request.text()

    const lines = rawBody.split('\r').filter(Boolean)
    const mshSegment = lines.find((line) => line.startsWith('MSH'))

    let messageType = 'unknown'
    if (mshSegment) {
      const fields = mshSegment.split('|')
      if (fields.length > 8) {
        messageType = fields[8]
      }
    }

    const sql = getSql()

    const [inserted] = await sql<[{ id: string }]>`
      INSERT INTO signals (signal_type, source, idempotency_key, payload, source_metadata)
      VALUES ('run', 'hl7v2-http', ${crypto.randomUUID()}, ${JSON.stringify({ raw: rawBody })}, ${JSON.stringify({ messageType, segmentCount: lines.length })})
      RETURNING id
    `

    return NextResponse.json({ accepted: true, signalId: inserted.id })
  } catch (error) {
    console.error('[api/webhooks/hl7] Error processing HL7 message:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
