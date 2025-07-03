<script lang="ts">
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";

  let email = $state("");
  let password = $state("");
  let error = $state("");
  let showPassword = $state(false);

  async function login() {
    error = "";
    const { error: authError } = await supabase.auth.signInWithPassword({
      email,
      password,
    });

    if (authError) {
      error = authError.message;
    } else {
      goto("/");
    }
  }
</script>

<main
  class="min-h-screen flex flex-col items-center justify-center p-6 max-w-md mx-auto"
>
  <h1 class="w-full text-7xl text-center font-eb-garamond-medium">Portico</h1>
  <svg
    class="mx-auto my-12 shrink min-w-32 min-h-32"
    width="170"
    height="310"
    viewBox="0 0 17 31"
    xmlns="http://www.w3.org/2000/svg"
  >
    <defs>
      <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stop-color="#a7e7df" />
        <stop offset="90%" stop-color="#64a9a0" />
      </linearGradient>
    </defs>

    <path
      d="M15.676 4.01852H15.0125C14.6477 4.01852 14.3521 3.71866 14.3521 3.34877C14.3521 2.97887 14.0564 2.67901 13.6916 2.67901H4.39212C3.88426 2.67901 3.5184 3.17316 3.66008 3.66774L6.74378 14.4327C6.82068 14.7011 6.82437 14.9857 6.75447 15.2561L3.62739 27.3519C3.50071 27.8419 3.86524 28.321 4.36478 28.321H7.79621C8.63823 28.321 9.32082 27.6288 9.32082 26.7749V9.35411C9.32082 8.92716 9.66212 8.58105 10.0831 8.58105H11.2065C11.6275 8.58105 11.9688 8.92716 11.9688 9.35412V29.4539C11.9688 30.3078 11.2862 31 10.4441 31H1.52545C0.524382 31 -0.205031 30.0382 0.0518973 29.057L3.66634 15.2538C3.73677 14.9848 3.73432 14.7016 3.65925 14.434L0.163203 1.96913C-0.113707 0.981832 0.617483 0 1.62965 0H15.4754C16.3174 0 17 0.692229 17 1.54614V2.67587C17 3.41739 16.4072 4.01852 15.676 4.01852Z"
      fill="url(#logoGradient)"
    />

    <path
      d="M14.3319 9.35412C14.3319 8.92716 14.6732 8.58105 15.0942 8.58105H16.2377C16.6587 8.58105 17 8.92716 17 9.35411V30.2269C17 30.6539 16.6587 31 16.2377 31H15.0942C14.6732 31 14.3319 30.6539 14.3319 30.2269V9.35412Z"
      fill="url(#logoGradient)"
    />
  </svg>

  <form on:submit|preventDefault={login} class="space-y-4">
    <div>
      <label for="email">Email</label>
      <input
        name="email"
        type="email"
        bind:value={email}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    <div>
      <label for="password">Password</label>
      <input
        name="password"
        type={showPassword ? "text" : "password"}
        bind:value={password}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    <div class="flex items-center">
      <input
        type="checkbox"
        id="showPasswordCheckbox"
        bind:checked={showPassword}
        class="mr-2"
      />
      <label for="showPasswordCheckbox" class="text-sm">Show Password</label>
    </div>

    {#if error}
      <p class="text-red-500">{error}</p>
    {/if}

    <div class="flex items-center justify-between">
      <button type="submit" class="bg-sea text-black px-4 py-2 rounded">
        Login
      </button>

      <a href="/register" class="text-sea underline text-sm ml-4"> Register </a>
    </div>
  </form>
</main>
