import supabase from "./supabase";

export async function getUserId() {
  const {
    data: { user },
    error,
  } = await supabase.auth.getUser();
  if (error) throw error;
  if (!user) throw new Error("User must be logged in");
  return user.id;
}
