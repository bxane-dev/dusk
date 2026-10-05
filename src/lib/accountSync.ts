import { api } from "./api";
import { getSupabaseClient } from "./auth";

function hasCloudState(value: unknown) {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return Array.isArray(record.profiles) && record.profiles.length > 0;
}

export async function syncAccountState() {
  const supabase = getSupabaseClient();
  if (!supabase) throw new Error("Supabase is not configured.");

  const { data: authData, error: authError } = await supabase.auth.getSession();
  if (authError) throw authError;
  const user = authData.session?.user;
  if (!user) throw new Error("Sign in to sync Dusk.");

  const state = await api.exportAccountState();
  const { error } = await supabase.from("dusk_account_state").upsert(
    {
      user_id: user.id,
      state,
      state_version: 1,
      updated_at: new Date().toISOString(),
    },
    { onConflict: "user_id" },
  );
  if (error) throw error;
}

export async function hydrateAccountState() {
  const supabase = getSupabaseClient();
  if (!supabase) return false;

  const { data: authData, error: authError } = await supabase.auth.getSession();
  if (authError) throw authError;
  const user = authData.session?.user;
  if (!user) return false;

  const { data, error } = await supabase
    .from("dusk_account_state")
    .select("state")
    .eq("user_id", user.id)
    .maybeSingle();
  if (error) throw error;

  if (hasCloudState(data?.state)) {
    await api.importAccountState(data!.state as Record<string, unknown>);
    return true;
  }

  await syncAccountState();
  return false;
}
