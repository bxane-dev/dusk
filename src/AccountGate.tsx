import { type FormEvent, type MouseEvent, type ReactNode, useEffect, useState } from "react";
import { LockKeyhole, LogIn, Minus, Square, UserRound, UserPlus, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import duskLogo from "./assets/dusk-logo.png";
import {
  currentDuskAccount,
  loginDuskAccount,
  registerDuskAccount,
  supabaseConfigured,
  type DuskAccount,
} from "./lib/auth";
import { hydrateAccountState } from "./lib/accountSync";
import { api } from "./lib/api";

type Mode = "login" | "register";

function AccountWindowBar() {
  async function handleMouseDown(event: MouseEvent<HTMLElement>) {
    if (event.button !== 0) return;
    const target = event.target;
    if (target instanceof HTMLElement && target.closest("button")) return;

    const appWindow = getCurrentWindow();
    if (event.detail === 2) {
      await appWindow.toggleMaximize().catch(() => undefined);
      return;
    }
    await appWindow.startDragging().catch(() => undefined);
  }

  return (
    <header
      className="account-window-titlebar"
      data-tauri-drag-region
      onMouseDown={(event) => void handleMouseDown(event)}
    >
      <div className="account-window-brand" data-tauri-drag-region>
        <img src={duskLogo} alt="" draggable={false} />
        <span>Dusk</span>
      </div>
      <div className="account-window-controls">
        <button
          type="button"
          className="account-window-control"
          aria-label="Minimize Dusk"
          title="Minimize"
          onClick={() => void getCurrentWindow().minimize()}
        >
          <Minus size={15} />
        </button>
        <button
          type="button"
          className="account-window-control"
          aria-label="Maximize or restore Dusk"
          title="Maximize / restore"
          onClick={() => void getCurrentWindow().toggleMaximize()}
        >
          <Square size={12} />
        </button>
        <button
          type="button"
          className="account-window-control close"
          aria-label="Close Dusk"
          title="Close"
          onClick={() => void getCurrentWindow().close()}
        >
          <X size={15} />
        </button>
      </div>
    </header>
  );
}

export default function AccountGate(props: { children: ReactNode }) {
  const [account, setAccount] = useState<DuskAccount | null>(null);
  const [guest, setGuest] = useState(false);
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
        if (localStorage.getItem("dusk-account-mode") === "guest") {
          await api.setAccountScope(null);
          setGuest(true);
          return;
        }

        const existing = await currentDuskAccount();
        if (existing) {
          await api.setAccountScope(existing.user.id);
          await hydrateAccountState().catch(() => false);
          setAccount(existing);
        } else {
          await api.setAccountScope(null);
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
          localStorage.removeItem("dusk-account-mode");
          await api.setAccountScope(result.account.user.id);
          await hydrateAccountState();
          setAccount(result.account);
        }
      } else {
        const signedIn = await loginDuskAccount(username, password);
        localStorage.removeItem("dusk-account-mode");
        await api.setAccountScope(signedIn.user.id);
        await hydrateAccountState();
        setAccount(signedIn);
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function continueAsGuest() {
    setBusy(true);
    setMessage("");
    try {
      localStorage.setItem("dusk-account-mode", "guest");
      await api.setAccountScope(null);
      setGuest(true);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  if (checking) {
    return (
      <main className="account-shell">
        <AccountWindowBar />
        <div className="account-card compact">
          <div className="account-logo">
            <img className="account-logo-image" src={duskLogo} alt="" draggable={false} />
          </div>
          <strong>Opening Dusk…</strong>
        </div>
      </main>
    );
  }

  if (account || guest) return <>{props.children}</>;

  return (
    <main className="account-shell">
      <AccountWindowBar />
      <section className="account-card">
        <div className="account-brand">
          <div className="account-logo">
            <img className="account-logo-image" src={duskLogo} alt="Dusk" draggable={false} />
          </div>
          <div>
            <span>DUSK ACCOUNT</span>
            <h1>{mode === "login" ? "Welcome back" : "Create your account"}</h1>
            <p>Sync your Dusk library, profiles, progress, collections, and private cloud saves across devices.</p>
          </div>
        </div>

        {!supabaseConfigured() && (
          <div className="account-message error">
            Dusk cloud accounts are temporarily unavailable. Guest mode still works.
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

        <div className="account-divider"><span>or</span></div>

        <button
          className="account-guest"
          type="button"
          disabled={busy}
          onClick={() => void continueAsGuest()}
        >
          <UserRound size={17} />
          Continue as guest
        </button>
        <p className="account-guest-note">
          Guest mode keeps your library and saves local to this PC. You can sign in later from Settings.
        </p>
      </section>
    </main>
  );
}
