import { createClient, type Session, type SupabaseClient, type User } from "@supabase/supabase-js";

const DUSK_SUPABASE_URL = "https://cwfmizfkysdrsesuhkym.supabase.co";
const DUSK_SUPABASE_PUBLISHABLE_KEY = "sb_publishable_yB9lh3iBKpH-tBHFA9neNA_bcNnxbK4";

export const SUPABASE_URL =
  (import.meta.env.VITE_SUPABASE_URL as string | undefined)?.trim() || DUSK_SUPABASE_URL;
export const SUPABASE_KEY =
  (import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY as string | undefined)?.trim() ||
  DUSK_SUPABASE_PUBLISHABLE_KEY;

let client: SupabaseClient | null = null;

export function supabaseConfigured() {
  return Boolean(SUPABASE_URL && SUPABASE_KEY);
}

export function getSupabaseClient() {
  if (!supabaseConfigured()) return null;
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

export type DuskAccount = {
  user: User;
  session: Session;
  username: string;
  email: string;
};

async function loadUsername(user: User) {
  const supabase = getSupabaseClient();
  if (!supabase) return "";
  const { data } = await supabase
    .from("dusk_accounts")
    .select("username")
    .eq("user_id", user.id)
    .maybeSingle();
  return data?.username || String(user.user_metadata?.username || "");
}

export async function currentDuskAccount(): Promise<DuskAccount | null> {
  const supabase = getSupabaseClient();
  if (!supabase) return null;
  const { data, error } = await supabase.auth.getSession();
  if (error) throw error;
  const session = data.session;
  if (!session?.user) return null;
  return {
    user: session.user,
    session,
    username: await loadUsername(session.user),
    email: session.user.email || "",
  };
}

export async function registerDuskAccount(input: {
  email: string;
  username: string;
  password: string;
}): Promise<{ account: DuskAccount | null; message: string }> {
  if (!supabaseConfigured()) {
    throw new Error("Dusk cloud accounts are not configured in this build.");
  }

  const email = input.email.trim();
  const username = input.username.trim();

  if (!/^[A-Za-z0-9_.-]{3,24}$/.test(username)) {
    throw new Error("Username must be 3-24 characters using letters, numbers, ., _, or -.");
  }
  if (input.password.length < 6) {
    throw new Error("Password must be at least 6 characters.");
  }

  const response = await fetch(`${SUPABASE_URL}/functions/v1/dusk-register`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      apikey: SUPABASE_KEY,
    },
    body: JSON.stringify({ email, username, password: input.password }),
  });

  const payload = await response.json().catch(() => ({}));
  if (!response.ok || !payload?.session?.access_token || !payload?.session?.refresh_token) {
    throw new Error(payload?.error || "Could not create Dusk account.");
  }

  const supabase = getSupabaseClient()!;
  const { data, error } = await supabase.auth.setSession({
    access_token: payload.session.access_token,
    refresh_token: payload.session.refresh_token,
  });

  if (error || !data.session || !data.user) {
    throw error || new Error("Dusk could not store the new account session.");
  }

  return {
    account: {
      user: data.user,
      session: data.session,
      username: String(payload.username || username).trim(),
      email: data.user.email || email,
    },
    message: "Account created and signed in.",
  };
}

export async function loginDuskAccount(username: string, password: string): Promise<DuskAccount> {
  if (!supabaseConfigured()) {
    throw new Error("Dusk cloud accounts are not configured in this build.");
  }
  const response = await fetch(`${SUPABASE_URL}/functions/v1/dusk-username-login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      apikey: SUPABASE_KEY,
    },
    body: JSON.stringify({ username: username.trim(), password }),
  });

  const payload = await response.json().catch(() => ({}));
  if (!response.ok || !payload?.session?.access_token || !payload?.session?.refresh_token) {
    throw new Error(payload?.error || "Invalid username or password.");
  }

  const supabase = getSupabaseClient()!;
  const { data, error } = await supabase.auth.setSession({
    access_token: payload.session.access_token,
    refresh_token: payload.session.refresh_token,
  });
  if (error || !data.session || !data.user) {
    throw error || new Error("Dusk could not store the account session.");
  }

  return {
    user: data.user,
    session: data.session,
    username: String(payload.username || username).trim(),
    email: data.user.email || "",
  };
}

export async function logoutDuskAccount() {
  const supabase = getSupabaseClient();
  if (!supabase) return;
  const { error } = await supabase.auth.signOut();
  if (error) throw error;
}
