<script lang="ts">
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";

  let email = $state("");
  let password = $state("");
  let confirmPassword = $state("");
  let error = $state("");
  let success = $state("");
  let showPassword = $state(false);

  async function register() {
    error = "";
    success = "";

    if (password !== confirmPassword) {
      error = "Passwords do not match.";
      return;
    }

    const { data, error: authError } = await supabase.auth.signUp({
      email,
      password,
    });

    if (authError) {
      if (
        authError.message.includes("User already registered") ||
        authError.message.includes("already exists")
      ) {
        error =
          "This email address is already in use. Try logging in or use a different email.";
      } else {
        error = authError.message;
      }
    } else if (data.user && !data.session) {
      // This case usually means email confirmation is required and has been sent,
      // or it's an existing user being re-sent a confirmation (Supabase's default for confirmed users with email confirmation enabled).
      success =
        "Registration initiated! Please check your email to confirm your account. If you've registered before, this will re-send the confirmation.";
    } else if (data.user && data.session) {
      // This case means email confirmation might be off, or it's an OAuth/magic link scenario where session is immediate.
      success = "Registration successful! You are now logged in.";
      // Optionally, redirect to a different page if login is immediate
      // await goto('/');
    } else {
      // Fallback, should ideally not be reached if data.user is the primary indicator
      error =
        "An unexpected issue occurred during registration. Please try again.";
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
        type={showPassword ? "text" : "password"}
        bind:value={password}
        class="border w-full p-2 rounded text-black"
        required
      />
    </div>

    <div>
      <label for="confirmPassword">Confirm Password</label>
      <input
        name="confirmPassword"
        type={showPassword ? "text" : "password"}
        bind:value={confirmPassword}
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
      <label for="showPasswordCheckbox" class="text-sm">Show Passwords</label>
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
    <a href="/login" class="text-blue-500 underline">Login</a>
  </p>
</main>
