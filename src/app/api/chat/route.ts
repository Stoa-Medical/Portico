import { NextResponse } from 'next/server'
// import { streamText } from 'ai'

export async function POST() {
  // TODO: Configure AI Gateway model routing
  // The AI Gateway will provide access to the preferred model (e.g., anthropic/claude-sonnet-4.6)
  // via a unified endpoint. Once configured, uncomment the streamText import and implement:
  //
  // const result = streamText({
  //   model: gateway('anthropic/claude-sonnet-4.6'),
  //   messages,
  // })
  // return result.toDataStreamResponse()

  return NextResponse.json(
    { error: 'Chat endpoint not yet configured. AI Gateway model pending.' },
    { status: 501 },
  )
}
