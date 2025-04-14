<!-- TODO: Below is AI boilerplate code, think through actual design -->

<script>
  import PageHeader from "../lib/components/PageHeader.svelte";
  import { Card, Button, Heading } from "flowbite-svelte";
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <PageHeader title="Welcome" breadcrumbs={[{ label: "Home", url: "/" }]} />

  <!-- Main Content -->
  <div class="grid grid-cols-1 gap-6">
    <Card class="overflow-x-auto max-w-full bg-card">
      <div class="p-4">
        <Heading tag="h2" class="mb-4">Hi there, Howard!</Heading>
        <Heading tag="h2" class="mb-4">Let's create a new project!</Heading>

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
