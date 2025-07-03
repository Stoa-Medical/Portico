<script lang="ts">
  import supabase from "$lib/supabase";
  import { goto } from "$app/navigation";
  import { fade } from "svelte/transition";

  let email = $state("");
  let password = $state("");
  let confirmPassword = $state("");
  let error = $state("");
  let success = $state("");
  let showPassword = $state(false);
  let redirecting = $state(false);

  // Regex for basic RFC5322-compliant email structure (simplified)
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  const emailErrorMessage =
    "Invalid email format. Please use a valid address like name@example.com.";
  // Password: min 8 chars, at least 1 upper, 1 lower, 1 digit, 1 special
  const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[\W_]).{8,}$/;
  const passwordRequirementMessage = "Password must meet all of the following:";

  // Derived validation flags (single object)
  const passwordCriteria = $derived(() => ({
    length: password.length >= 8,
    uppercase: /[A-Z]/.test(password),
    lowercase: /[a-z]/.test(password),
    number: /\d/.test(password),
    special: /[\W_]/.test(password),
  }));

  async function register() {
    error = "";
    success = "";

    // Client-side validation
    if (!emailRegex.test(email)) {
      error = emailErrorMessage;
      return;
    }

    if (!passwordRegex.test(password)) {
      error = passwordRequirementMessage;
      return;
    }

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
      // Show success screen for 0.24 seconds, then navigate to home
      redirecting = true;
      setTimeout(() => {
        goto("/");
      }, 240);
    } else {
      // Fallback, should ideally not be reached if data.user is the primary indicator
      error =
        "An unexpected issue occurred during registration. Please try again.";
    }
  }
</script>

<main class="p-6 max-w-md mx-auto" in:fade>
  {#if redirecting}
    <h1 class="text-2xl font-bold mb-4">Registration successful!</h1>
    <p>Redirecting...</p>
  {:else}
    <h1 class="text-2xl font-bold mb-4">Register</h1>

    <form on:submit|preventDefault={register} class="space-y-4">
      <div>
        <label for="email">Email</label>
        <input
          name="email"
          type="email"
          bind:value={email}
          class="border w-full p-2 rounded text-black"
          pattern="[^\s@]+@[^\s@]+\.[^\s@]+"
          required
        />
        <p class="text-xs text-gray-400 mt-1">
          Example: <code>name@example.com</code>
        </p>
      </div>

      <div>
        <label for="password">Password</label>
        <input
          name="password"
          type={showPassword ? "text" : "password"}
          bind:value={password}
          class="border w-full p-2 rounded text-black"
          minlength="8"
          required
        />
        <p class="text-xs text-gray-400 mt-1 mb-1 font-medium">
          {passwordRequirementMessage}
        </p>
        <ul class="text-xs space-y-1">
          <li
            class={passwordCriteria().length
              ? "text-green-400"
              : "text-gray-400"}
          >
            • Minimum 8 characters
          </li>
          <li
            class={passwordCriteria().uppercase
              ? "text-green-400"
              : "text-gray-400"}
          >
            • At least one uppercase letter (A-Z)
          </li>
          <li
            class={passwordCriteria().lowercase
              ? "text-green-400"
              : "text-gray-400"}
          >
            • At least one lowercase letter (a-z)
          </li>
          <li
            class={passwordCriteria().number
              ? "text-green-400"
              : "text-gray-400"}
          >
            • At least one number (0-9)
          </li>
          <li
            class={passwordCriteria().special
              ? "text-green-400"
              : "text-gray-400"}
          >
            • At least one special character (!@#$…)
          </li>
        </ul>
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
  {/if}
</main>
