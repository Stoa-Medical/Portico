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
  ];

  let checkingAuth = $state(true);
  let user = $state<any>(undefined); // Using any for user type, adjust if you have a specific user type
  let showLogoutTooltip = $state(false);

  const allowedPages = ["/login", "/register"];

  const isOnAllowedPage = () =>
    allowedPages.some((path) => window.location.pathname.startsWith(path));

  onMount(async () => {
    document.documentElement.classList.add("dark");

    const { data } = await supabase.auth.getSession();
    user = data.session?.user;

    if (!user && !isOnAllowedPage()) {
      goto("/login");
    }

    supabase.auth.onAuthStateChange(async (_event, session) => {
      if (!session && !isOnAllowedPage()) {
        user = null;
        goto("/login");
      } else {
        user = session?.user;
      }
    });

    // Watch route changes and redirect if not logged in
    page.subscribe(() => {
      if (!user && !isOnAllowedPage()) {
        goto("/login");
      }
    });

    checkingAuth = false;
  });

  // Access default slot content in Svelte 5
  let { children } = $props();
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
        <button aria-label="Home" class="w-10 h-12 mt-4">
          <img src="/logo-icon.svg" alt="Portico logo" class="w-full h-full" />
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
        <div class="relative">
          <button
            type="button"
            aria-label="Logout"
            class="w-8 h-8 bg-cyan-800 text-white flex items-center justify-center mb-2 rounded-full cursor-pointer uppercase font-semibold"
            onclick={() => supabase.auth.signOut()}
            onmouseenter={() => (showLogoutTooltip = true)}
            onmouseleave={() => (showLogoutTooltip = false)}
          >
            {user?.email?.[0] ?? "?"}
          </button>
          {#if showLogoutTooltip}
            <div
              class="absolute left-full top-1/2 -translate-y-1/2 ml-2 px-2 py-1 bg-gray-700 text-white text-xs rounded shadow-md whitespace-nowrap z-10"
            >
              Click to logout
            </div>
          {/if}
        </div>
      </aside>
    {/if}

    <!-- Main content -->
    <div class="flex-1 overflow-auto">
      <main class="p-6 h-full flex flex-col">
        {@render children?.()}
      </main>
    </div>
  </div>
{/if}
