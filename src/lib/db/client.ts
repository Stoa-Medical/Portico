import postgres from 'postgres'

let _sql: ReturnType<typeof postgres> | null = null

export function getSql() {
  if (!_sql) _sql = postgres(process.env.DATABASE_URL!, { prepare: false })
  return _sql
}
