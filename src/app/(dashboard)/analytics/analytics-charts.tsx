"use client"

import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

const STATUS_COLORS: Record<string, string> = {
  pending: "var(--chart-3)",
  dispatched: "var(--chart-2)",
  completed: "var(--chart-1)",
  failed: "var(--destructive)",
}

export type VolumePoint = {
  date: string
  pending: number
  dispatched: number
  completed: number
  failed: number
}

export type PerformancePoint = {
  agent: string
  successRate: number
  total: number
}

export type StatusSlice = {
  status: string
  count: number
}

export function VolumeChart({ data }: { data: VolumePoint[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Signal Volume</CardTitle>
        <CardDescription>Daily signal counts by status</CardDescription>
      </CardHeader>
      <CardContent>
        {data.length === 0 ? (
          <p className="py-12 text-center text-sm text-muted-foreground">No data yet</p>
        ) : (
          <ResponsiveContainer width="100%" height={280}>
            <AreaChart data={data}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
              <XAxis dataKey="date" stroke="var(--muted-foreground)" fontSize={12} />
              <YAxis stroke="var(--muted-foreground)" fontSize={12} />
              <Tooltip
                contentStyle={{
                  backgroundColor: "var(--popover)",
                  border: "1px solid var(--border)",
                  borderRadius: 8,
                }}
              />
              <Legend />
              <Area type="monotone" dataKey="completed" stackId="1" stroke={STATUS_COLORS.completed} fill={STATUS_COLORS.completed} />
              <Area type="monotone" dataKey="dispatched" stackId="1" stroke={STATUS_COLORS.dispatched} fill={STATUS_COLORS.dispatched} />
              <Area type="monotone" dataKey="pending" stackId="1" stroke={STATUS_COLORS.pending} fill={STATUS_COLORS.pending} />
              <Area type="monotone" dataKey="failed" stackId="1" stroke={STATUS_COLORS.failed} fill={STATUS_COLORS.failed} />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  )
}

export function PerformanceChart({ data }: { data: PerformancePoint[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Agent Success Rate</CardTitle>
        <CardDescription>Completed sessions ÷ total sessions</CardDescription>
      </CardHeader>
      <CardContent>
        {data.length === 0 ? (
          <p className="py-12 text-center text-sm text-muted-foreground">No data yet</p>
        ) : (
          <ResponsiveContainer width="100%" height={280}>
            <BarChart data={data}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
              <XAxis dataKey="agent" stroke="var(--muted-foreground)" fontSize={12} />
              <YAxis stroke="var(--muted-foreground)" fontSize={12} domain={[0, 1]} tickFormatter={(v) => `${Math.round(v * 100)}%`} />
              <Tooltip
                contentStyle={{
                  backgroundColor: "var(--popover)",
                  border: "1px solid var(--border)",
                  borderRadius: 8,
                }}
                formatter={(value: number) => `${Math.round(value * 100)}%`}
              />
              <Bar dataKey="successRate" fill="var(--chart-1)" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  )
}

export function StatusPie({ data }: { data: StatusSlice[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Signal Status Distribution</CardTitle>
        <CardDescription>Last 7 days</CardDescription>
      </CardHeader>
      <CardContent>
        {data.length === 0 ? (
          <p className="py-12 text-center text-sm text-muted-foreground">No data yet</p>
        ) : (
          <ResponsiveContainer width="100%" height={280}>
            <PieChart>
              <Pie
                data={data}
                dataKey="count"
                nameKey="status"
                cx="50%"
                cy="50%"
                outerRadius={90}
                label={(entry: { status: string; count: number }) => `${entry.status}: ${entry.count}`}
              >
                {data.map((entry) => (
                  <Cell key={entry.status} fill={STATUS_COLORS[entry.status] ?? "var(--chart-5)"} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{
                  backgroundColor: "var(--popover)",
                  border: "1px solid var(--border)",
                  borderRadius: 8,
                }}
              />
            </PieChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  )
}
