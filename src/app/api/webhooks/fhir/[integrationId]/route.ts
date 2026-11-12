import { NextRequest, NextResponse } from 'next/server'
import { fhirMessageId, ingestForIntegration, ingestResponse } from '@/lib/signals/ingest'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ integrationId: string }> },
) {
  try {
    const { integrationId } = await params
    const body = await request.json()

    const result = await ingestForIntegration(request, integrationId, 'rest-webhook', {
      source: 'fhir-webhook',
      messageId: fhirMessageId(body ?? {}),
      payload: body,
      sourceMetadata: {
        resourceType: body?.resourceType ?? 'Unknown',
        bundleType: body?.type ?? null,
        entryCount: Array.isArray(body?.entry) ? body.entry.length : 0,
      },
    })
    return ingestResponse(result)
  } catch (error) {
    console.error('[api/webhooks/fhir] Error processing FHIR payload:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
