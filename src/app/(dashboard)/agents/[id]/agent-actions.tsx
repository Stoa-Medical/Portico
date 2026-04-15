"use client"

import { useTransition } from "react"
import { useRouter } from "next/navigation"
import { Trash2 } from "lucide-react"
import { toast } from "sonner"

import { deleteAgent } from "@/lib/agents/actions"
import { Button } from "@/components/ui/button"

export function DeleteAgentButton({ id }: { id: string }) {
  const router = useRouter()
  const [pending, startTransition] = useTransition()

  function handleDelete() {
    if (!confirm("Delete this agent? This cannot be undone.")) return
    startTransition(async () => {
      try {
        await deleteAgent(id)
        toast.success("Agent deleted")
        router.push("/agents")
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "Failed to delete")
      }
    })
  }

  return (
    <Button variant="outline" size="sm" onClick={handleDelete} disabled={pending}>
      <Trash2 className="mr-2 h-4 w-4" />
      {pending ? "Deleting..." : "Delete"}
    </Button>
  )
}
