import { NextRequest, NextResponse } from 'next/server'

export async function GET(request: NextRequest) {
  const authHeader = request.headers.get('authorization')

  if (authHeader !== `Bearer ${process.env.CRON_SECRET}`) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 })
  }

  // TODO: Implement dead letter signal cleanup
  // - Move stale signals (lease expired, retries exhausted) to dead_letter_signals
  // - Reconcile outbox events that were sent but not acknowledged
  // - Clean up completed runtime sessions older than retention period

  console.log('[cron/cleanup] Cleanup job executed')

  return NextResponse.json({ ok: true })
}
