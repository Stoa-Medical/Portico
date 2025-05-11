/**
 * Result type for functional error handling
 * A monadic container for handling success/failure outcomes
 */
export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };
