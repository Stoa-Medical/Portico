import Link from "next/link"
import { Database, Sparkles } from "lucide-react"
import { formatDistanceToNow } from "date-fns"

import { getSql } from "@/lib/db/client"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

type MappingRow = {
  id: string
  name: string
  aiGenerated: boolean | null
  agentId: string | null
  createdAt: Date
}

export default async function MappingsPage() {
  const sql = getSql()

  const mappings = await sql<MappingRow[]>`
    SELECT
      id, name,
      ai_generated AS "aiGenerated",
      agent_id AS "agentId",
      created_at AS "createdAt"
    FROM data_mappings
    ORDER BY created_at DESC
  `

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">Data Mappings</h1>
        <p className="text-muted-foreground">
          Schema-to-schema field mappings used by transform steps
        </p>
      </div>

      {mappings.length === 0 ? (
        <Card>
          <CardContent className="py-16 text-center">
            <Database className="mx-auto mb-4 h-10 w-10 text-muted-foreground" />
            <p className="text-lg font-medium">No mappings yet</p>
            <p className="mt-2 text-sm text-muted-foreground">
              Mappings are produced by transform workflows or inserted directly.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {mappings.map((m) => (
            <Link key={m.id} href={`/mappings/${m.id}`}>
              <Card className="h-full cursor-pointer transition-colors hover:border-primary/50">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-papyrus/20">
                        <Database className="h-5 w-5 text-papyrus" />
                      </div>
                      <div>
                        <CardTitle className="text-base">{m.name}</CardTitle>
                        <CardDescription className="text-xs">
                          {formatDistanceToNow(new Date(m.createdAt), { addSuffix: true })}
                        </CardDescription>
                      </div>
                    </div>
                    {m.aiGenerated && (
                      <Badge variant="secondary" className="text-xs">
                        <Sparkles className="mr-1 h-3 w-3" />
                        AI
                      </Badge>
                    )}
                  </div>
                </CardHeader>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
