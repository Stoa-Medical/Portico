import Link from "next/link"
import { Radio } from "lucide-react"
import { formatDistanceToNow } from "date-fns"

import { getSql } from "@/lib/db/client"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

type IntegrationRow = {
  id: string
  name: string
  connectionType: string
  direction: "inbound" | "outbound"
  status: "active" | "inactive" | "error"
  lastSeenAt: Date | null
}

const statusVariant: Record<IntegrationRow["status"], "success" | "warning" | "destructive"> = {
  active: "success",
  inactive: "warning",
  error: "destructive",
}

export default async function IntegrationsPage() {
  const sql = getSql()

  const integrations = await sql<IntegrationRow[]>`
    SELECT
      id, name,
      connection_type AS "connectionType",
      direction,
      status,
      last_seen_at AS "lastSeenAt"
    FROM integrations
    ORDER BY created_at DESC
  `

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">Integrations</h1>
        <p className="text-muted-foreground">
          Inbound and outbound connections to external healthcare systems
        </p>
      </div>

      {integrations.length === 0 ? (
        <Card>
          <CardContent className="py-16 text-center">
            <Radio className="mx-auto mb-4 h-10 w-10 text-muted-foreground" />
            <p className="text-lg font-medium">No integrations configured</p>
            <p className="mt-2 text-sm text-muted-foreground">
              Add an integration row via SQL or the admin API to get started.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {integrations.map((i) => {
            const tone = statusVariant[i.status]
            return (
              <Link key={i.id} href={`/integrations/${i.id}`}>
                <Card className="h-full cursor-pointer transition-colors hover:border-primary/50">
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-sky/20">
                          <Radio className="h-5 w-5 text-sky" />
                        </div>
                        <div>
                          <CardTitle className="text-base">{i.name}</CardTitle>
                          <CardDescription className="text-xs">
                            {i.connectionType} · {i.direction}
                          </CardDescription>
                        </div>
                      </div>
                      <div
                        className={`h-2 w-2 rounded-full ${
                          tone === "success"
                            ? "bg-success"
                            : tone === "warning"
                              ? "bg-warning"
                              : "bg-destructive"
                        }`}
                      />
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="flex items-center justify-between">
                      <Badge variant="secondary" className="text-xs">
                        {i.status}
                      </Badge>
                      <span className="text-xs text-muted-foreground">
                        {i.lastSeenAt
                          ? `Last seen ${formatDistanceToNow(new Date(i.lastSeenAt), { addSuffix: true })}`
                          : "Never connected"}
                      </span>
                    </div>
                  </CardContent>
                </Card>
              </Link>
            )
          })}
        </div>
      )}
    </div>
  )
}
