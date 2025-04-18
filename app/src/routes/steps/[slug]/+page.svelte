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
  import { saveStep, deleteStep, getStep, getAgents } from "../../agents/api";

  // Stores
  $: stepId = $page.params.slug;
  $: searchParams = $page.url.searchParams;
  $: isNewStep = stepId === "new";

  let step = null;
  let agents = [];
  let isLoading = true;
  const stepTypes = ["prompt", "python"];

  // Load agents + step data
  async function loadData() {
    isLoading = true;
    try {
      agents = await getAgents();

      if (isNewStep) {
        // TODO: Must change init here when schema changes, make this more maintainable.
        const agentName = searchParams.get("agentName");
        step = {
          id: "new",
          step_type: "prompt",
          agent_id: parseInt(searchParams.get("agentId") || "1"),
          name: agentName ? agentName + " Step" : "",
          step_content: "",
          // isActive: true,
          // lastEdited: "Just now",
          created_timestamp: new Date().toISOString().split("T")[0],
        };
      } else {
        step = await getStep(stepId);
      }
    } catch (err) {
      console.error("Failed to load data:", err);
    } finally {
      isLoading = false;
    }
  }

  loadData();

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
    const agentId = parseInt(searchParams.get("agentId") || "");
    goto(isNewStep ? `/agents?agentId=${agentId}&tab=Steps` : "/steps");
  }
</script>

<main class="container mx-auto p-4">
  {#if isLoading}
    <div class="flex items-center justify-center py-32"></div>
  {:else}
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
          <Heading tag="h1" class="text-2xl font-bold"
            >{step?.name || "New Step"}</Heading
          >
          <Badge color={step.step_type === "Python" ? "blue" : "purple"}>
            {step.step_type}
          </Badge>
        </div>
      </div>
      <div class="flex flex-wrap gap-2 mb-6">
        <!-- <Button color="green" on:click={clickExecuteStep}>
          <PlayOutline class="mr-2 h-5 w-5" />
          Execute
        </Button> -->
        {#if !isNewStep}
          <Button class="bg-[#CE5A5A]" on:click={clickDeleteStep}>
            <TrashBinOutline class="mr-2 h-5 w-5" />
            Delete
          </Button>
        {/if}
        <Button class="bg-sea text-black" on:click={clickSaveStep}>Save</Button>
      </div>
    </div>

    <!-- Step Configuration -->
    <StepConfig bind:step {stepTypes} {agents} />
  {/if}
</main>
