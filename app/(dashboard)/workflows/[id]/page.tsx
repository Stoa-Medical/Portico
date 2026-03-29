export default async function WorkflowDetailPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params

  return (
    <div>
      <h1 className="text-2xl font-bold text-white">Workflow Detail</h1>
      <p className="mt-2 text-sm text-zinc-400">Workflow {id}</p>
    </div>
  )
}
