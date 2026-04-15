import Link from "next/link"
import { notFound } from "next/navigation"
import { ArrowLeft, Database, Sparkles } from "lucide-react"

import { getSql } from "@/lib/db/client"
import type { DataMapping } from "@/lib/agents/types"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export default async function MappingDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params
  const sql = getSql()

  const [mapping] = await sql<DataMapping[]>`
    SELECT
      id, name,
      source_schema AS "sourceSchema",
      target_schema AS "targetSchema",
      field_mappings AS "fieldMappings",
      ai_generated AS "aiGenerated",
      agent_id AS "agentId",
      created_at AS "createdAt",
      updated_at AS "updatedAt"
    FROM data_mappings WHERE id = ${id}
  `

  if (!mapping) notFound()

  return (
    <div className="p-6">
      <Link
        href="/mappings"
        className="mb-4 inline-flex items-center text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ArrowLeft className="mr-1 h-4 w-4" />
        Back to Mappings
      </Link>

      <div className="mb-8 flex items-start justify-between gap-3">
        <div className="flex items-start gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-papyrus/20">
            <Database className="h-6 w-6 text-papyrus" />
          </div>
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">{mapping.name}</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Created {new Date(mapping.createdAt).toLocaleDateString()}
            </p>
          </div>
        </div>
        {mapping.aiGenerated && (
          <Badge variant="secondary">
            <Sparkles className="mr-1 h-3 w-3" />
            AI Generated
          </Badge>
        )}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Source Schema</CardTitle>
          </CardHeader>
          <CardContent>
            {mapping.sourceSchema ? (
              <pre className="max-h-96 overflow-auto rounded bg-muted p-4 font-mono text-xs">
                {JSON.stringify(mapping.sourceSchema, null, 2)}
              </pre>
            ) : (
              <p className="text-sm text-muted-foreground">Not set</p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Target Schema</CardTitle>
          </CardHeader>
          <CardContent>
            {mapping.targetSchema ? (
              <pre className="max-h-96 overflow-auto rounded bg-muted p-4 font-mono text-xs">
                {JSON.stringify(mapping.targetSchema, null, 2)}
              </pre>
            ) : (
              <p className="text-sm text-muted-foreground">Not set</p>
            )}
          </CardContent>
        </Card>
      </div>

      <Card className="mt-4">
        <CardHeader>
          <CardTitle className="text-base">Field Mappings</CardTitle>
        </CardHeader>
        <CardContent>
          {mapping.fieldMappings ? (
            <pre className="max-h-96 overflow-auto rounded bg-muted p-4 font-mono text-xs">
              {JSON.stringify(mapping.fieldMappings, null, 2)}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground">No field mappings defined</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
