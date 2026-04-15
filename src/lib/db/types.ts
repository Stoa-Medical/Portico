export interface Agent {
  id: string
  name: string
  description: string | null
  capabilities: unknown
  policy: unknown
  preferredModel: string | null
  createdAt: Date
  updatedAt: Date
}

export interface Step {
  id: string
  agentId: string | null
  name: string
  stepType: 'python' | 'llm' | 'transform' | 'validate' | 'fhir'
  stepOrder: number
  config: unknown
  createdAt: Date
  updatedAt: Date
}

export interface Signal {
  id: string
  signalType: 'run' | 'sync' | 'fyi'
  source: string
  idempotencyKey: string
  agentId: string | null
  payload: unknown
  responseData: unknown
  runtimeSessionId: string | null
  sourceMetadata: unknown
  status: 'pending' | 'dispatched' | 'completed' | 'failed'
  errorMessage: string | null
  leasedAt: Date | null
  leaseExpiresAt: Date | null
  createdAt: Date
  updatedAt: Date
}

export interface RuntimeSession {
  id: string
  agentId: string | null
  signalId: string | null
  status: 'waiting' | 'running' | 'completed' | 'failed' | 'cancelled'
  stepResults: unknown
  stepExecutionMs: unknown
  totalExecutionMs: number | null
  dataQualityScore: number | null
  createdAt: Date
  completedAt: Date | null
}

export interface SignalCountRow {
  status: string
  count: number
}

export interface AgentPerformanceRow {
  agentId: string
  agentName: string | null
  avgExecutionMs: number | null
  successCount: number
  totalCount: number
}

export interface RecentSessionRow {
  id: string
  agentId: string | null
  agentName: string | null
  status: string
  totalExecutionMs: number | null
  dataQualityScore: number | null
  createdAt: Date
  completedAt: Date | null
}
