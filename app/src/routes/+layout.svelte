<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { derived } from "svelte/store";
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";

  const currentPath = derived(page, ($page) => $page.url.pathname);

  const links = [
    { href: "/", icon: "/home-icon.svg", label: "Home" },
    { href: "/agents", icon: "/folder-icon.svg", label: "Agents" },
    { href: "/analytics", icon: "/donut-icon.svg", label: "Analytics" },
    { href: "/steps", icon: "/tree-icon.svg", label: "Steps" },
  ];

  let checkingAuth = true;
  let user = undefined;

  onMount(async () => {
    document.documentElement.classList.add("dark");

    const { data } = await supabase.auth.getSession();
    user = data.session?.user;

    if (!user && !window.location.pathname.startsWith("/login")) {
      goto("/login");
    }

    supabase.auth.onAuthStateChange(async (_event, session) => {
      if (!session && !window.location.pathname.startsWith("/login")) {
        user = null;
        goto("/login");
      } else {
        user = session?.user;
      }
    });

    checkingAuth = false;
  });
</script>

{#if checkingAuth}
  <div class="h-screen flex items-center justify-center text-white bg-gray-900">
    <p>Checking auth...</p>
  </div>
{:else}
  <!-- Main layout content -->
  <div class="flex h-screen bg-gray-900 text-white app">
    <!-- Sidebar and main content go here -->
    {#if user}
      <aside class="w-16 bg-gray-800 flex flex-col items-center py-4 space-y-6">
        <!-- Logo -->
        <button class="w-10 h-12 mt-4">
          <img src="/logo-icon.svg" class="w-full h-full" />
        </button>

        <div class="flex flex-col space-y-6">
          {#each links as { href, icon, label }}
            <a
              {href}
              class={`w-10 h-10 p-2 rounded-full flex items-center justify-center transition 
      hover:scale-110
      ${
        $currentPath === href
          ? "bg-white/10 border border-cyan-200/20 shadow-sm backdrop-blur-sm"
          : "bg-transparent"
      }`}
            >
              <img src={icon} alt={label} title={label} class="w-full h-full" />
            </a>
          {/each}
        </div>

        <div class="flex-grow"></div>

        <!-- Avatar Placeholder -->
        <div
          class="w-8 h-8 bg-gray-500 mb-2 rounded-full"
          on:click={() => supabase.auth.signOut()}
        ></div>
      </aside>
    {/if}

    <!-- Main content -->
    <div class="flex-1 overflow-auto">
      <main class="p-6">
        <slot />
      </main>
    </div>
  </div>
{/if}
