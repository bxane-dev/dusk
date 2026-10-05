import { type FormEvent, type ReactNode, useEffect, useState } from "react";
import { LockKeyhole, LogIn, Moon, UserPlus } from "lucide-react";
import {
  currentDuskAccount,
  loginDuskAccount,
  registerDuskAccount,
  supabaseConfigured,
  type DuskAccount,
} from "./lib/auth";
import { hydrateAccountState } from "./lib/accountSync";

type Mode = "login" | "register";

export default function AccountGate(props: { children: ReactNode }) {
  const [account, setAccount] = useState<DuskAccount | null>(null);
  const [checking, setChecking] = useState(true);
  const [mode, setMode] = useState<Mode>("login");
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    void (async () => {
      try {
        const existing = await currentDuskAccount();
        if (existing) {
          await hydrateAccountState().catch(() => false);
          setAccount(existing);
        }
      } catch (error) {
        setMessage(error instanceof Error ? error.message : String(error));
      } finally {
        setChecking(false);
      }
    })();
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setMessage("");

    try {
      if (mode === "register") {
        const result = await registerDuskAccount({ email, username, password });
        setMessage(result.message);
        if (result.account) {
          await hydrateAccountState();
          setAccount(result.account);
        }
      } else {
        const signedIn = await loginDuskAccount(username, password);
        await hydrateAccountState();
        setAccount(signedIn);
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  if (checking) {
    return (
      <main className="account-shell">
        <div className="account-card compact">
          <div className="account-logo"><Moon size={28} /></div>
          <strong>Opening Dusk…</strong>
        </div>
      </main>
    );
  }

  if (account) return <>{props.children}</>;

  return (
    <main className="account-shell">
      <section className="account-card">
        <div className="account-brand">
          <div className="account-logo"><Moon size={30} /></div>
          <div>
            <span>DUSK ACCOUNT</span>
            <h1>{mode === "login" ? "Welcome back" : "Create your account"}</h1>
            <p>Sync your Dusk library, profiles, progress, collections, and private cloud saves across devices.</p>
          </div>
        </div>

        {!supabaseConfigured() && (
          <div className="account-message error">
            This build is missing the Dusk Supabase URL or publishable key.
          </div>
        )}

        <form className="account-form" onSubmit={submit}>
          {mode === "register" && (
            <label>
              <span>Email</span>
              <input
                type="email"
                autoComplete="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="you@example.com"
                required
              />
            </label>
          )}
          <label>
            <span>Username</span>
            <input
              autoComplete="username"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              placeholder="bxane"
              minLength={3}
              maxLength={24}
              required
            />
          </label>
          <label>
            <span>Password</span>
            <input
              type="password"
              autoComplete={mode === "login" ? "current-password" : "new-password"}
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="••••••••"
              minLength={6}
              required
            />
          </label>

          {message && <div className="account-message">{message}</div>}

          <button className="account-submit" disabled={busy || !supabaseConfigured()} type="submit">
            {mode === "login" ? <LogIn size={17} /> : <UserPlus size={17} />}
            {busy ? "Working…" : mode === "login" ? "Sign in" : "Create account"}
          </button>
        </form>

        <button
          className="account-switch"
          type="button"
          onClick={() => {
            setMode(mode === "login" ? "register" : "login");
            setMessage("");
          }}
        >
          <LockKeyhole size={15} />
          {mode === "login" ? "Need an account? Register" : "Already have an account? Sign in"}
        </button>
      </section>
    </main>
  );
}
