<script lang="ts">
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";

  let email = "";
  let password = "";
  let error = "";

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

<main class="p-6 max-w-md mx-auto">
  <h1 class="text-2xl font-bold mb-4">Login</h1>

  <form on:submit|preventDefault={login} class="space-y-4">
    <div>
      <label>Email</label>
      <input
        type="email"
        bind:value={email}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    <div>
      <label>Password</label>
      <input
        type="password"
        bind:value={password}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    {#if error}
      <p class="text-red-500">{error}</p>
    {/if}

    <button type="submit" class="bg-sea text-black px-4 py-2 rounded"
      >Login</button
    >
  </form>
</main>
