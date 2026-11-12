import { NextRequest, NextResponse } from 'next/server'
import { fhirMessageId, ingestForIntegration, ingestResponse } from '@/lib/signals/ingest'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ integrationId: string }> },
) {
  try {
    const { integrationId } = await params
    const body = await request.json()

    const result = await ingestForIntegration(request, integrationId, 'fhir-subscription', {
      source: 'medplum-subscription',
      messageId: fhirMessageId(body ?? {}),
      payload: body,
      sourceMetadata: {
        subscriptionId: body?.subscription?.reference ?? null,
        resourceType: body?.resourceType ?? 'unknown',
      },
    })
    return ingestResponse(result)
  } catch (error) {
    console.error('[api/webhooks/medplum] Error processing notification:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
