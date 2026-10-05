import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { api } from "./api";
import type { CloudSaveStatus, SaveBackupRecord } from "../types";

const SUPABASE_URL = (import.meta.env.VITE_SUPABASE_URL as string | undefined)?.trim() || "";
const SUPABASE_KEY =
  (import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY as string | undefined)?.trim() || "";

let client: SupabaseClient | null = null;
let authPromise: Promise<{ client: SupabaseClient; userId: string; accessToken: string }> | null = null;

function configured() {
  return Boolean(SUPABASE_URL && SUPABASE_KEY);
}

function getClient() {
  if (!configured()) return null;
  if (!client) {
    client = createClient(SUPABASE_URL, SUPABASE_KEY, {
      auth: {
        persistSession: true,
        autoRefreshToken: true,
        detectSessionInUrl: false,
        storageKey: "dusk-supabase-auth",
      },
    });
  }
  return client;
}

async function ensureCloudIdentity() {
  if (authPromise) return authPromise;

  authPromise = (async () => {
    const supabase = getClient();
    if (!supabase) {
      throw new Error("Supabase cloud saves are not configured for this Dusk build.");
    }

    const existing = await supabase.auth.getSession();
    if (existing.error) throw existing.error;

    let session = existing.data.session;
    if (!session) {
      const created = await supabase.auth.signInAnonymously();
      if (created.error) throw created.error;
      session = created.data.session;
    }

    if (!session?.user?.id || !session.access_token) {
      throw new Error("Supabase did not return a usable cloud-save session.");
    }

    return {
      client: supabase,
      userId: session.user.id,
      accessToken: session.access_token,
    };
  })();

  try {
    return await authPromise;
  } finally {
    authPromise = null;
  }
}

export async function cloudSaveStatus(): Promise<CloudSaveStatus> {
  if (!configured()) {
    return {
      configured: false,
      authenticated: false,
      userId: null,
      message: "Cloud saves are ready in Dusk, but this build is missing the Supabase project URL/publishable key.",
    };
  }

  try {
    const identity = await ensureCloudIdentity();
    return {
      configured: true,
      authenticated: true,
      userId: identity.userId,
      message: "Supabase cloud save storage is connected.",
    };
  } catch (error) {
    return {
      configured: true,
      authenticated: false,
      userId: null,
      message: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function syncBackupToSupabase(backup: SaveBackupRecord) {
  const identity = await ensureCloudIdentity();
  return api.uploadSaveBackupToCloud({
    backupId: backup.id,
    supabaseUrl: SUPABASE_URL,
    publishableKey: SUPABASE_KEY,
    accessToken: identity.accessToken,
    authUserId: identity.userId,
  });
}

export async function syncCloudManifest() {
  const identity = await ensureCloudIdentity();
  return api.uploadCloudManifest({
    supabaseUrl: SUPABASE_URL,
    publishableKey: SUPABASE_KEY,
    accessToken: identity.accessToken,
    authUserId: identity.userId,
  });
}
