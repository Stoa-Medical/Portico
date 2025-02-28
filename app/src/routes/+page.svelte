<!-- TODO: Below is AI boilerplate code, think through actual design -->

<script>
  import { 
    Card, 
    Button, 
    Heading, 
    Breadcrumb, 
    BreadcrumbItem,
  } from 'flowbite-svelte';
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex justify-between items-center">
      <Heading tag="h1" class="text-2xl font-bold">Welcome</Heading>
    </div>
  </div>
  
  <!-- Main Content -->
  <div class="grid grid-cols-1 gap-6">
    <Card>
      <Heading tag="h2" class="mb-4">Navigation</Heading>
      <p class="mb-4">Welcome to the application. Use the links below to navigate:</p>
      
      <div class="flex flex-col space-y-4 sm:flex-row sm:space-y-0 sm:space-x-4">
        <Button href="/agents" color="blue">
          View Agents
        </Button>
        <Button href="/steps" color="green">
          View Steps
        </Button>
        <Button href="/analytics" color="purple">
          Analytics
        </Button>
        <Button href="/" color="light">
          Home
        </Button>
      </div>
    </Card>
  </div>
</main>

<style>

</style>
