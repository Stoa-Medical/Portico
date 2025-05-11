import supabase from "./supabase";
import { get } from "svelte/store";
import { enforceAgentOwnership } from "./stores/configStore";

/**
 * Result type for functional error handling
 * A monadic container for handling success/failure outcomes
 */
type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

/**
 * Pure function to retrieve user from Supabase auth
 * @returns Result containing either the user ID or an error
 */
const fetchCurrentUser = async (): Promise<Result<string, Error>> => {
  try {
    const { data, error } = await supabase.auth.getUser();

    if (error) return { ok: false, error: new Error(error.message) };
    if (!data.user)
      return { ok: false, error: new Error("User must be logged in") };

    return { ok: true, value: data.user.id };
  } catch (err) {
    return {
      ok: false,
      error: err instanceof Error ? err : new Error(String(err)),
    };
  }
};

/**
 * Gets the current user ID if agent ownership filtering is enabled
 * @returns User ID if ownership filtering is enabled, null otherwise
 */
export async function getUserIdIfEnforced(): Promise<string | null> {
  // If agent ownership enforcement is disabled, return null
  if (!get(enforceAgentOwnership)) {
    return null;
  }

  // Otherwise, return the current user ID
  const result = await fetchCurrentUser();

  if (!result.ok) throw result.error;
  return result.value;
}

/**
 * Gets the current user ID regardless of settings
 * @returns User ID
 */
export async function getUserId(): Promise<string> {
  const result = await fetchCurrentUser();

  if (!result.ok) throw result.error;
  return result.value;
}
