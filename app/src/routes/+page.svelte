<script>
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { Card, Button } from "flowbite-svelte";
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <PageHeader title="Welcome" breadcrumbs={[{ label: "Home", url: "/" }]} />

  <!-- Main Content -->
  <div class="grid grid-cols-1 gap-6">
    <Card class="overflow-x-auto max-w-full bg-card">
      <div class="p-4">
        <h2 class="mb-4 text-4xl">Hi there!</h2>
        <br />
        <h2 class="mb-4 text-4xl">Let's create a new project!</h2>

        <p class="mt-2 mb-6">
          Create a project from scratch or start from a template
        </p>

        <div class="flex flex-col sm:flex-row sm:space-y-0 sm:space-x-4 w-full">
          <Button href="/agents" size="xl" class="bg-sea text-black"
            >Agents</Button
          >
          <Button href="/steps" size="xl" class="bg-sea text-black"
            >Steps</Button
          >
          <Button href="/analytics" size="xl" class="bg-sea text-black"
            >Analytics</Button
          >
        </div>
      </div>
    </Card>
  </div>
</main>

<style>
</style>
