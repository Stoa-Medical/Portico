const ENGINE_URL = process.env.ENGINE_GRPC_URL || 'localhost:50051'

export async function dispatchSignal(signalId: string): Promise<void> {
  // TODO: Implement gRPC client to Rust engine
  console.log(`[engine] Would dispatch signal ${signalId} to ${ENGINE_URL}`)
}
