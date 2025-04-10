<script>
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  import {
    Button,
    Heading,
    Breadcrumb,
    BreadcrumbItem,
    Badge,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    TrashBinOutline,
    PlayOutline,
  } from "flowbite-svelte-icons";
  import StepConfig from "$lib/components/StepConfig.svelte";
  import { saveStep, deleteStep } from "../../agents/api";

  // Stores
  $: stepId = $page.params.slug;
  $: searchParams = $page.url.searchParams;

  // Reactive flags
  $: isNewStep = stepId === "new";

  // Reactively assign step
  let step;
  $: {
    if (isNewStep) {
      step = {
        id: "new",
        name: "",
        type: "Prompt",
        agentId: parseInt(searchParams.get("agentId") || "1"),
        agentName: searchParams.get("agentName") ?? "",
        content: "",
        isActive: true,
        lastEdited: "Just now",
        createdAt: new Date().toISOString().split("T")[0],
      };
    } else {
      step = getStepData(stepId);
    }
  }

  const agents = [
    { id: 1, name: "Agent Smith" },
    { id: 2, name: "Agent Johnson" },
    { id: 3, name: "Agent Brown" },
  ];

  const stepTypes = ["Prompt", "Python"];

  async function clickSaveStep() {
    await saveStep(step);
    goBack();
  }

  async function clickDeleteStep() {
    if (confirm("Are you sure you want to delete this step?")) {
      await deleteStep(step.id);
      goBack();
    }
  }

  function goBack() {
    if (isNewStep) {
      goto("/agents/");
    } else {
      goto("/steps");
    }
  }

  // TODO: Add back execution features
  // function clickExecuteStep() {
  //   alert(`Executing ${step.type} step: ${step.name}`);
  // }

  function getStepData(id) {
    return {
      id: parseInt(id),
      name: `Step ${id}`,
      type: id % 2 === 0 ? "Prompt" : "Python",
      agentId: 1,
      agentName: "Agent Smith",
      content:
        id % 2 === 0
          ? `You are an AI assistant that helps with data analysis.\n{{data}}`
          : `import pandas as pd\n...`,
      isActive: true,
      lastEdited: "2 hours ago",
      createdAt: "2023-10-15",
    };
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem href="/steps">Steps</BreadcrumbItem>
      <BreadcrumbItem>Step {stepId}</BreadcrumbItem>
    </Breadcrumb>

    <div class="flex flex-col sm:flex-row gap-4 mb-4">
      <div class="flex items-center gap-3">
        <Button color="light" size="sm" on:click={goBack}>
          <ArrowLeftOutline class="mr-2 h-4 w-4" />
          Back
        </Button>
        <Heading tag="h1" class="text-2xl font-bold">{step.name}</Heading>
        <Badge color={step.type === "Python" ? "blue" : "purple"}>
          {step.type}
        </Badge>
      </div>
    </div>
    <div class="flex flex-wrap gap-2 mb-6">
      <!-- <Button color="green" on:click={clickExecuteStep}>
        <PlayOutline class="mr-2 h-5 w-5" />
        Execute
      </Button> -->
      <Button color="red" on:click={clickDeleteStep}>
        <TrashBinOutline class="mr-2 h-5 w-5" />
        Delete
      </Button>
      <Button color="blue" on:click={clickSaveStep}>Save</Button>
    </div>
  </div>

  <!-- Step Configuration -->
  <StepConfig bind:step {stepTypes} {agents} />
</main>
