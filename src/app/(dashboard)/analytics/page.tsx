import { formatDistanceToNow } from "date-fns"

import {
  getAgentPerformance,
  getRecentSessions,
  getSignalCounts,
  getSignalVolumeByDay,
} from "@/lib/analytics/queries"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  PerformanceChart,
  StatusPie,
  VolumeChart,
  type PerformancePoint,
  type StatusSlice,
  type VolumePoint,
} from "./analytics-charts"

const SEVEN_DAYS_MS = 7 * 24 * 60 * 60 * 1000

export default async function AnalyticsPage() {
  const since = new Date(Date.now() - SEVEN_DAYS_MS)

  const [signalCounts, volumeRows, performance, recentSessions] = await Promise.all([
    getSignalCounts(since),
    getSignalVolumeByDay(since),
    getAgentPerformance(since),
    getRecentSessions(20),
  ])

  const statusSlices: StatusSlice[] = signalCounts.map((r) => ({
    status: r.status,
    count: r.count,
  }))

  const volumeMap = new Map<string, VolumePoint>()
  for (const row of volumeRows) {
    const key = new Date(row.bucket).toISOString().slice(0, 10)
    const existing =
      volumeMap.get(key) ??
      ({
        date: key,
        pending: 0,
        dispatched: 0,
        completed: 0,
        failed: 0,
      } as VolumePoint)
    if (
      row.status === "pending" ||
      row.status === "dispatched" ||
      row.status === "completed" ||
      row.status === "failed"
    ) {
      existing[row.status] = row.count
    }
    volumeMap.set(key, existing)
  }
  const volume: VolumePoint[] = Array.from(volumeMap.values()).sort((a, b) =>
    a.date.localeCompare(b.date),
  )

  const performanceData: PerformancePoint[] = performance
    .filter((p) => p.agentName)
    .map((p) => ({
      agent: p.agentName!,
      successRate: p.successRate,
      total: p.totalCount,
    }))

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight">Analytics</h1>
        <p className="text-muted-foreground">Signal throughput and agent performance (last 7 days)</p>
      </div>

      <div className="mb-6 grid gap-6 lg:grid-cols-2">
        <VolumeChart data={volume} />
        <StatusPie data={statusSlices} />
      </div>

      <div className="mb-6">
        <PerformanceChart data={performanceData} />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Sessions</CardTitle>
          <CardDescription>Latest 20 runtime sessions</CardDescription>
        </CardHeader>
        <CardContent>
          {recentSessions.length === 0 ? (
            <p className="py-6 text-center text-sm text-muted-foreground">No sessions yet</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Agent</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Duration</TableHead>
                  <TableHead>Quality</TableHead>
                  <TableHead>When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentSessions.map((s) => (
                  <TableRow key={s.id}>
                    <TableCell>{s.agentName ?? "—"}</TableCell>
                    <TableCell>
                      <Badge variant={s.status === "completed" ? "default" : "secondary"} className="text-xs">
                        {s.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-sm">
                      {s.totalExecutionMs != null ? `${s.totalExecutionMs}ms` : "—"}
                    </TableCell>
                    <TableCell className="text-sm">
                      {s.dataQualityScore != null ? s.dataQualityScore.toFixed(2) : "—"}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {formatDistanceToNow(new Date(s.createdAt), { addSuffix: true })}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
