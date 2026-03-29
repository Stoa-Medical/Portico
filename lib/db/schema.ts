import { relations } from 'drizzle-orm'
import {
  boolean,
  integer,
  jsonb,
  pgEnum,
  pgTable,
  real,
  serial,
  text,
  timestamp,
  uuid,
} from 'drizzle-orm/pg-core'

// ─── Enums ───────────────────────────────────────────────────────────────────

export const stepTypeEnum = pgEnum('step_type', [
  'python',
  'llm',
  'transform',
  'validate',
  'fhir',
])

export const signalTypeEnum = pgEnum('signal_type', [
  'run',
  'sync',
  'fyi',
])

export const signalStatusEnum = pgEnum('signal_status', [
  'pending',
  'dispatched',
  'completed',
  'failed',
])

export const runningStatusEnum = pgEnum('running_status', [
  'waiting',
  'running',
  'completed',
  'failed',
  'cancelled',
])

export const connectionTypeEnum = pgEnum('connection_type', [
  'hl7v2-mllp',
  'hl7v2-http',
  'fhir-subscription',
  'rest-webhook',
  'file-drop',
])

export const connectionDirectionEnum = pgEnum('connection_direction', [
  'inbound',
  'outbound',
])

export const integrationStatusEnum = pgEnum('integration_status', [
  'active',
  'inactive',
  'error',
])

// ─── Tables ──────────────────────────────────────────────────────────────────

export const agents = pgTable('agents', {
  id: uuid('id').primaryKey().defaultRandom(),
  name: text('name').notNull(),
  description: text('description'),
  capabilities: jsonb('capabilities'),
  policy: jsonb('policy'),
  preferredModel: text('preferred_model'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const steps = pgTable('steps', {
  id: uuid('id').primaryKey().defaultRandom(),
  agentId: uuid('agent_id').references(() => agents.id),
  name: text('name').notNull(),
  stepType: stepTypeEnum('step_type').notNull(),
  stepOrder: integer('step_order').notNull(),
  config: jsonb('config'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const runtimeSessions = pgTable('runtime_sessions', {
  id: uuid('id').primaryKey().defaultRandom(),
  agentId: uuid('agent_id').references(() => agents.id),
  signalId: uuid('signal_id'), // No DB FK — circular with signals.runtimeSessionId. Linked via Drizzle relations.
  status: runningStatusEnum('status').notNull().default('waiting'),
  stepResults: jsonb('step_results'),
  stepExecutionMs: jsonb('step_execution_ms'),
  totalExecutionMs: integer('total_execution_ms'),
  dataQualityScore: real('data_quality_score'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  completedAt: timestamp('completed_at'),
})

export const signals = pgTable('signals', {
  id: uuid('id').primaryKey().defaultRandom(),
  signalType: signalTypeEnum('signal_type').notNull(),
  source: text('source').notNull(),
  idempotencyKey: text('idempotency_key').notNull().unique(),
  agentId: uuid('agent_id').references(() => agents.id),
  payload: jsonb('payload'),
  responseData: jsonb('response_data'),
  runtimeSessionId: uuid('runtime_session_id').references(() => runtimeSessions.id),
  sourceMetadata: jsonb('source_metadata'),
  status: signalStatusEnum('status').notNull().default('pending'),
  errorMessage: text('error_message'),
  leasedAt: timestamp('leased_at'),
  leaseExpiresAt: timestamp('lease_expires_at'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const dataMappings = pgTable('data_mappings', {
  id: uuid('id').primaryKey().defaultRandom(),
  name: text('name').notNull(),
  sourceSchema: jsonb('source_schema'),
  targetSchema: jsonb('target_schema'),
  fieldMappings: jsonb('field_mappings'),
  aiGenerated: boolean('ai_generated').default(false),
  agentId: uuid('agent_id').references(() => agents.id),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const integrations = pgTable('integrations', {
  id: uuid('id').primaryKey().defaultRandom(),
  name: text('name').notNull(),
  connectionType: connectionTypeEnum('connection_type').notNull(),
  direction: connectionDirectionEnum('direction').notNull(),
  config: jsonb('config'),
  agentId: uuid('agent_id').references(() => agents.id),
  status: integrationStatusEnum('status').notNull().default('active'),
  lastSeenAt: timestamp('last_seen_at'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const deadLetterSignals = pgTable('dead_letter_signals', {
  id: uuid('id').primaryKey().defaultRandom(),
  signalId: uuid('signal_id').references(() => signals.id),
  agentId: uuid('agent_id').references(() => agents.id),
  failureReason: text('failure_reason'),
  lastStepIndex: integer('last_step_index'),
  lastError: jsonb('last_error'),
  originalPayload: jsonb('original_payload'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  resolvedAt: timestamp('resolved_at'),
  resolvedBy: text('resolved_by'),
})

export const outboxEvents = pgTable('outbox_events', {
  id: uuid('id').primaryKey().defaultRandom(),
  signalId: uuid('signal_id').references(() => signals.id),
  stepIndex: integer('step_index'),
  eventType: text('event_type').notNull(),
  payload: jsonb('payload'),
  status: text('status').notNull().default('pending'),
  medplumResponse: jsonb('medplum_response'),
  retryCount: integer('retry_count').default(0),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  sentAt: timestamp('sent_at'),
})

export const engineState = pgTable('engine_state', {
  id: integer('id').primaryKey().default(1),
  lastRedisStreamId: text('last_redis_stream_id'),
  lastHeartbeat: timestamp('last_heartbeat'),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const auditLog = pgTable('audit_log', {
  id: serial('id').primaryKey(),
  actor: text('actor').notNull(),
  action: text('action').notNull(),
  resourceType: text('resource_type').notNull(),
  resourceId: uuid('resource_id'),
  details: jsonb('details'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
})

// ─── Relations ───────────────────────────────────────────────────────────────

export const agentsRelations = relations(agents, ({ many }) => ({
  steps: many(steps),
  signals: many(signals),
  runtimeSessions: many(runtimeSessions),
  dataMappings: many(dataMappings),
  integrations: many(integrations),
  deadLetterSignals: many(deadLetterSignals),
}))

export const stepsRelations = relations(steps, ({ one }) => ({
  agent: one(agents, {
    fields: [steps.agentId],
    references: [agents.id],
  }),
}))

export const signalsRelations = relations(signals, ({ one, many }) => ({
  agent: one(agents, {
    fields: [signals.agentId],
    references: [agents.id],
  }),
  runtimeSession: one(runtimeSessions, {
    fields: [signals.runtimeSessionId],
    references: [runtimeSessions.id],
  }),
  deadLetterSignals: many(deadLetterSignals),
  outboxEvents: many(outboxEvents),
}))

export const runtimeSessionsRelations = relations(runtimeSessions, ({ one, many }) => ({
  agent: one(agents, {
    fields: [runtimeSessions.agentId],
    references: [agents.id],
  }),
  signals: many(signals),
}))

export const dataMappingsRelations = relations(dataMappings, ({ one }) => ({
  agent: one(agents, {
    fields: [dataMappings.agentId],
    references: [agents.id],
  }),
}))

export const integrationsRelations = relations(integrations, ({ one }) => ({
  agent: one(agents, {
    fields: [integrations.agentId],
    references: [agents.id],
  }),
}))

export const deadLetterSignalsRelations = relations(deadLetterSignals, ({ one }) => ({
  signal: one(signals, {
    fields: [deadLetterSignals.signalId],
    references: [signals.id],
  }),
  agent: one(agents, {
    fields: [deadLetterSignals.agentId],
    references: [agents.id],
  }),
}))

export const outboxEventsRelations = relations(outboxEvents, ({ one }) => ({
  signal: one(signals, {
    fields: [outboxEvents.signalId],
    references: [signals.id],
  }),
}))
