"use client"

import { useState, useTransition } from "react"
import { useRouter } from "next/navigation"
import { ArrowDown, ArrowUp, Plus, Trash2 } from "lucide-react"
import { toast } from "sonner"

import { createStep, deleteStep, updateStep } from "@/lib/agents/actions"
import type { Step } from "@/lib/db/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"

const STEP_TYPES = ["python", "llm", "transform", "validate", "fhir"] as const
type StepType = (typeof STEP_TYPES)[number]

export function StepEditor({ agentId, steps }: { agentId: string; steps: Step[] }) {
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [stepType, setStepType] = useState<StepType>("python")
  const [configText, setConfigText] = useState("")
  const [pending, startTransition] = useTransition()

  function handleAdd() {
    if (!name.trim()) {
      toast.error("Name is required")
      return
    }
    let config: unknown = null
    if (configText.trim()) {
      try {
        config = JSON.parse(configText)
      } catch {
        toast.error("Config must be valid JSON")
        return
      }
    }
    startTransition(async () => {
      try {
        await createStep(agentId, {
          name: name.trim(),
          stepType,
          stepOrder: steps.length,
          config: config ?? undefined,
        })
        toast.success("Step added")
        setOpen(false)
        setName("")
        setStepType("python")
        setConfigText("")
        router.refresh()
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Failed to add step")
      }
    })
  }

  function handleDelete(stepId: string) {
    if (!confirm("Delete this step?")) return
    startTransition(async () => {
      try {
        await deleteStep(stepId)
        toast.success("Step deleted")
        router.refresh()
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Failed to delete")
      }
    })
  }

  function handleMove(index: number, direction: -1 | 1) {
    const target = index + direction
    if (target < 0 || target >= steps.length) return
    const a = steps[index]
    const b = steps[target]
    startTransition(async () => {
      try {
        await Promise.all([
          updateStep(a.id, { stepOrder: b.stepOrder }),
          updateStep(b.id, { stepOrder: a.stepOrder }),
        ])
        router.refresh()
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Failed to reorder")
      }
    })
  }

  return (
    <div>
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-lg font-semibold">Steps</h2>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button size="sm">
              <Plus className="mr-2 h-4 w-4" />
              Add Step
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add Step</DialogTitle>
              <DialogDescription>Define a new step for this workflow</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="step-name">Name</Label>
                <Input
                  id="step-name"
                  placeholder="e.g., Parse FHIR payload"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
              </div>
              <div className="grid gap-2">
                <Label>Type</Label>
                <Select value={stepType} onValueChange={(v) => setStepType(v as StepType)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {STEP_TYPES.map((t) => (
                      <SelectItem key={t} value={t}>
                        {t}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="step-config">Config (JSON, optional)</Label>
                <Textarea
                  id="step-config"
                  rows={4}
                  placeholder='{"prompt": "..."}'
                  value={configText}
                  onChange={(e) => setConfigText(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setOpen(false)} disabled={pending}>
                Cancel
              </Button>
              <Button onClick={handleAdd} disabled={pending}>
                {pending ? "Adding..." : "Add Step"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {steps.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center text-sm text-muted-foreground">
            No steps yet. Add the first step to define this workflow.
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          {steps.map((step, idx) => (
            <Card key={step.id}>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-3">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-secondary text-xs font-semibold">
                      {idx + 1}
                    </div>
                    <div>
                      <CardTitle className="text-base">{step.name}</CardTitle>
                      <Badge variant="secondary" className="mt-1 text-xs">
                        {step.stepType}
                      </Badge>
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      disabled={idx === 0 || pending}
                      onClick={() => handleMove(idx, -1)}
                    >
                      <ArrowUp className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      disabled={idx === steps.length - 1 || pending}
                      onClick={() => handleMove(idx, 1)}
                    >
                      <ArrowDown className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 text-destructive"
                      disabled={pending}
                      onClick={() => handleDelete(step.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </CardHeader>
              {step.config != null && (
                <CardContent>
                  <pre className="max-h-32 overflow-auto rounded bg-muted p-3 font-mono text-xs text-muted-foreground">
                    {JSON.stringify(step.config, null, 2)}
                  </pre>
                </CardContent>
              )}
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
