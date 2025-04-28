<script lang="ts">
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";

  let email = "";
  let password = "";
  let error = "";
  let success = "";

  async function register() {
    error = "";
    success = "";

    const { error: authError } = await supabase.auth.signUp({
      email,
      password,
    });

    if (authError) {
      error = authError.message;
    } else {
      success = "Registration successful! Please check your email to confirm.";
    }
  }
</script>

<main class="p-6 max-w-md mx-auto">
  <h1 class="text-2xl font-bold mb-4">Register</h1>

  <form on:submit|preventDefault={register} class="space-y-4">
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
        type="password"
        bind:value={password}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    {#if error}
      <p class="text-red-500">{error}</p>
    {/if}

    {#if success}
      <p class="text-green-500">{success}</p>
    {/if}

    <button type="submit" class="bg-sea text-black px-4 py-2 rounded">
      Register
    </button>
  </form>

  <p class="mt-4">
    Already have an account?
    <a href="/login" class="text-sea underline">Login</a>
  </p>
</main>
