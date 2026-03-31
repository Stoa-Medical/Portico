export default async function MappingEditorPage({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = await params

  return (
    <div>
      <h1 className="text-2xl font-bold text-white">Mapping Editor</h1>
      <p className="mt-2 text-sm text-zinc-400">Mapping {id}</p>
    </div>
  )
}
