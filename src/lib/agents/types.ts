// ─── Enum types (mirror Drizzle pgEnum values) ─────────────────────────────

export type StepType = 'python' | 'llm' | 'transform' | 'validate' | 'fhir'

export type SignalType = 'run' | 'sync' | 'fyi'

export type SignalStatus = 'pending' | 'dispatched' | 'completed' | 'failed'

export type RunningStatus = 'waiting' | 'running' | 'completed' | 'failed' | 'cancelled'

export type ConnectionType =
  | 'hl7v2-mllp'
  | 'hl7v2-http'
  | 'fhir-subscription'
  | 'rest-webhook'
  | 'file-drop'

export type ConnectionDirection = 'inbound' | 'outbound'

export type IntegrationStatus = 'active' | 'inactive' | 'error'

// ─── Entity types (plain TS mirrors of Drizzle schema for use in components) ─

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
  stepType: StepType
  stepOrder: number
  config: unknown
  createdAt: Date
  updatedAt: Date
}

export interface Signal {
  id: string
  signalType: SignalType
  source: string
  idempotencyKey: string
  agentId: string | null
  payload: unknown
  responseData: unknown
  runtimeSessionId: string | null
  sourceMetadata: unknown
  status: SignalStatus
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
  status: RunningStatus
  stepResults: unknown
  stepExecutionMs: unknown
  totalExecutionMs: number | null
  dataQualityScore: number | null
  createdAt: Date
  completedAt: Date | null
}

export interface DataMapping {
  id: string
  name: string
  sourceSchema: unknown
  targetSchema: unknown
  fieldMappings: unknown
  aiGenerated: boolean | null
  agentId: string | null
  createdAt: Date
  updatedAt: Date
}

export interface Integration {
  id: string
  name: string
  connectionType: ConnectionType
  direction: ConnectionDirection
  config: unknown
  agentId: string | null
  status: IntegrationStatus
  lastSeenAt: Date | null
  createdAt: Date
  updatedAt: Date
}

export interface AuditLogEntry {
  id: number
  actor: string
  action: string
  resourceType: string
  resourceId: string | null
  details: unknown
  createdAt: Date
}
