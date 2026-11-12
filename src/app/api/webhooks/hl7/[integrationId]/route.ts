import { NextRequest, NextResponse } from 'next/server'
import { ingestForIntegration, ingestResponse } from '@/lib/signals/ingest'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ integrationId: string }> },
) {
  try {
    const { integrationId } = await params
    const raw = await request.text()

    const segments = raw.split(/\r\n|\r|\n/).filter(Boolean)
    const msh = segments.find((segment) => segment.startsWith('MSH'))?.split('|') ?? []
    // MSH-1 is the field separator itself, so MSH-n sits at index n-1.
    const messageType = msh[8] || 'unknown'
    const controlId = msh[9] || null

    const result = await ingestForIntegration(request, integrationId, 'hl7v2-http', {
      source: 'hl7v2-http',
      messageId: controlId,
      payload: { raw },
      sourceMetadata: { messageType, controlId, segmentCount: segments.length },
    })
    return ingestResponse(result)
  } catch (error) {
    console.error('[api/webhooks/hl7] Error processing HL7 message:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
