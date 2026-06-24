import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Activity,
  Bot,
  Braces,
  Check,
  ChevronDown,
  CircleDollarSign,
  Cloud,
  Code2,
  Gauge,
  KeyRound,
  PlugZap,
  RefreshCw,
  Trash2,
  X,
} from "lucide-react";
import "./App.css";

type MetricLine = {
  label: string;
  value: string;
};

type ProviderSnapshot = {
  provider_id: string;
  lines: MetricLine[];
};

type SavedSnapshot = {
  provider_id: string;
  captured_at: number;
  snapshot: ProviderSnapshot;
};

type DeepSeekKeySlot = {
  id: number;
  has_key: boolean;
};

type OpenCodeView = "spend" | "quota";

const placeholders = ["Antigravity"];
const opencodeSpendLabels = new Set(["Last 7 days", "Last 30 days", "Tokens (30d)", "All-time"]);

function App() {
  const [apiKey, setApiKey] = useState("");
  const [keySlots, setKeySlots] = useState<DeepSeekKeySlot[]>([]);
  const [isAddingKey, setIsAddingKey] = useState(false);
  const [claudeSnapshot, setClaudeSnapshot] = useState<ProviderSnapshot | null>(null);
  const [codexSnapshot, setCodexSnapshot] = useState<ProviderSnapshot | null>(null);
  const [deepseekSnapshot, setDeepseekSnapshot] = useState<ProviderSnapshot | null>(null);
  const [opencodeSnapshot, setOpencodeSnapshot] = useState<ProviderSnapshot | null>(null);
  const [opencodeQuotaConnected, setOpencodeQuotaConnected] = useState(false);
  const [showOpencodeQuotaSetup, setShowOpencodeQuotaSetup] = useState(false);
  const [opencodeCookie, setOpencodeCookie] = useState("");
  const [opencodeWorkspace, setOpencodeWorkspace] = useState("");
  const [opencodeView, setOpencodeView] = useState<OpenCodeView>("spend");
  const [lastUpdatedAt, setLastUpdatedAt] = useState<Record<string, number>>({});
  const [status, setStatus] = useState("Idle");
  const [error, setError] = useState("");

  const savedKeyCount = useMemo(
    () => keySlots.filter((slot) => slot.has_key).length,
    [keySlots],
  );
  const hasKey = savedKeyCount > 0;
  const canAddKey = savedKeyCount === 0;
  const opencodeLines = useMemo(
    () =>
      opencodeSnapshot?.lines.filter((line) =>
        opencodeView === "spend"
          ? opencodeSpendLabels.has(line.label)
          : !opencodeSpendLabels.has(line.label),
      ) ?? [],
    [opencodeSnapshot, opencodeView],
  );

  useEffect(() => {
    invoke<DeepSeekKeySlot[]>("list_deepseek_api_keys")
      .then((slots) => {
        setKeySlots(slots);
        setIsAddingKey(slots.every((slot) => !slot.has_key));
      })
      .catch(() => setKeySlots([]));

    invoke<SavedSnapshot[]>("list_saved_snapshots")
      .then((savedSnapshots) => {
        const updatedAt: Record<string, number> = {};

        for (const saved of savedSnapshots) {
          updatedAt[saved.provider_id] = saved.captured_at;

          if (saved.provider_id === "claude") {
            setClaudeSnapshot(saved.snapshot);
          } else if (saved.provider_id === "codex") {
            setCodexSnapshot(saved.snapshot);
          } else if (saved.provider_id === "deepseek") {
            setDeepseekSnapshot(saved.snapshot);
          } else if (saved.provider_id === "opencode") {
            setOpencodeSnapshot(saved.snapshot);
          }
        }

        setLastUpdatedAt(updatedAt);
      })
      .catch(() => {});

    invoke<boolean>("opencode_quota_session_status")
      .then(setOpencodeQuotaConnected)
      .catch(() => setOpencodeQuotaConnected(false));
  }, []);

  function markUpdated(snapshot: ProviderSnapshot, capturedAt = Math.floor(Date.now() / 1000)) {
    setLastUpdatedAt((current) => ({
      ...current,
      [snapshot.provider_id]: capturedAt,
    }));
  }

  function updatedLabel(providerId: string) {
    const capturedAt = lastUpdatedAt[providerId];

    if (!capturedAt) {
      return "Not refreshed";
    }

    const date = new Date(capturedAt * 1000);
    const pad = (value: number) => String(value).padStart(2, "0");

    return `${pad(date.getDate())}-${pad(date.getMonth() + 1)} ${pad(date.getHours())}:${pad(
      date.getMinutes(),
    )}`;
  }

  async function saveKey() {
    setError("");
    setStatus("Saving");
    try {
      const slots = await invoke<DeepSeekKeySlot[]>("save_deepseek_api_key", { apiKey });
      setApiKey("");
      setKeySlots(slots);
      setIsAddingKey(false);
      setStatus("Saved");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function deleteKey(slot: number) {
    setError("");
    setStatus("Deleting");
    try {
      const slots = await invoke<DeepSeekKeySlot[]>("delete_deepseek_api_key", { slot });
      setKeySlots(slots);
      setDeepseekSnapshot(null);
      setIsAddingKey(slots.every((nextSlot) => !nextSlot.has_key));
      setStatus("Deleted");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function refreshDeepSeek() {
    setError("");
    setStatus("Refreshing");
    try {
      const nextSnapshot = await invoke<ProviderSnapshot>("refresh_deepseek");
      setDeepseekSnapshot(nextSnapshot);
      markUpdated(nextSnapshot);
      setStatus("Updated");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function refreshCodex() {
    setError("");
    setStatus("Refreshing");
    try {
      const nextSnapshot = await invoke<ProviderSnapshot>("refresh_codex");
      setCodexSnapshot(nextSnapshot);
      markUpdated(nextSnapshot);
      setStatus("Updated");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function refreshClaude() {
    setError("");
    setStatus("Refreshing");
    try {
      const nextSnapshot = await invoke<ProviderSnapshot>("refresh_claude");
      setClaudeSnapshot(nextSnapshot);
      markUpdated(nextSnapshot);
      setStatus("Updated");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function refreshOpenCode() {
    setError("");
    setStatus("Refreshing");
    try {
      const nextSnapshot = await invoke<ProviderSnapshot>("refresh_opencode");
      setOpencodeSnapshot(nextSnapshot);
      markUpdated(nextSnapshot);
      setStatus("Updated");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function saveOpenCodeQuota() {
    setError("");
    setStatus("Saving quota");
    try {
      const nextSnapshot = await invoke<ProviderSnapshot>("save_opencode_quota_session", {
        cookie: opencodeCookie,
        workspace: opencodeWorkspace,
      });
      setOpencodeCookie("");
      setOpencodeWorkspace("");
      setShowOpencodeQuotaSetup(false);
      setOpencodeQuotaConnected(true);
      setOpencodeView("quota");
      setOpencodeSnapshot(nextSnapshot);
      markUpdated(nextSnapshot);
      setStatus("Updated");
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  async function disconnectOpenCodeQuota() {
    setError("");
    setStatus("Disconnecting");
    try {
      const connected = await invoke<boolean>("disconnect_opencode_quota");
      setOpencodeQuotaConnected(connected);
      setOpencodeView("spend");
      setShowOpencodeQuotaSetup(false);
      setStatus("Disconnected");
      await refreshOpenCode();
    } catch (caught) {
      setStatus("Error");
      setError(String(caught));
    }
  }

  return (
    <main className="panel">
      <header className="panel-header" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <div className="brand-mark" data-tauri-drag-region>
            <Activity size={18} />
          </div>
          <div data-tauri-drag-region>
            <h1>InfUsage</h1>
            <p>Inference usage monitor</p>
          </div>
        </div>
        <div className="header-actions">
          <span className={status === "Error" ? "status error" : "status"}>{status}</span>
          <button
            aria-label="Close"
            className="icon-button"
            onClick={() => getCurrentWindow().hide()}
            type="button"
          >
            <X size={15} />
          </button>
        </div>
      </header>

      <section className="provider-list" aria-label="Providers">
        <ProviderBlock
          actions={<IconButton icon={<RefreshCw size={14} />} label="Refresh" onClick={refreshCodex} />}
          icon={<Code2 size={16} />}
          metrics={codexSnapshot?.lines ?? []}
          state={codexSnapshot ? "Updated" : "Local login"}
          title="Codex"
          updatedAt={updatedLabel("codex")}
          variant={codexSnapshot ? "ok" : "muted"}
        />

        <ProviderBlock
          actions={<IconButton icon={<RefreshCw size={14} />} label="Refresh" onClick={refreshClaude} />}
          icon={<Bot size={16} />}
          metrics={claudeSnapshot?.lines ?? []}
          state={claudeSnapshot ? "Updated" : "Local login"}
          title="Claude Code"
          updatedAt={updatedLabel("claude")}
          variant={claudeSnapshot ? "ok" : "muted"}
        />

        <ProviderBlock
          actions={
            <>
              {!isAddingKey && canAddKey && (
                <IconButton icon={<KeyRound size={14} />} label="Add key" onClick={() => setIsAddingKey(true)} />
              )}
              <IconButton
                disabled={!hasKey}
                icon={<RefreshCw size={14} />}
                label="Refresh"
                onClick={refreshDeepSeek}
              />
            </>
          }
          extra={
            <>
              {hasKey && (
                <div className="key-list">
                  {keySlots
                    .filter((slot) => slot.has_key)
                    .map((slot) => (
                      <div className="key-row" key={slot.id}>
                        <span>API key saved</span>
                        <button className="ghost-button danger" onClick={() => deleteKey(slot.id)} type="button">
                          <Trash2 size={13} />
                          Delete
                        </button>
                      </div>
                    ))}
                </div>
              )}

              {isAddingKey && canAddKey && (
                <div className="form-grid">
                  <input
                    aria-label="DeepSeek API key"
                    onChange={(event) => setApiKey(event.target.value)}
                    placeholder="DeepSeek API key"
                    type="password"
                    value={apiKey}
                  />
                  <button disabled={!apiKey.trim()} onClick={saveKey} type="button">
                    Save
                  </button>
                  {hasKey && (
                    <button className="ghost-button" onClick={() => setIsAddingKey(false)} type="button">
                      Cancel
                    </button>
                  )}
                </div>
              )}
            </>
          }
          icon={<CircleDollarSign size={16} />}
          metrics={deepseekSnapshot?.lines ?? []}
          state={hasKey ? "Connected" : "Not connected"}
          title="DeepSeek"
          updatedAt={deepseekSnapshot ? updatedLabel("deepseek") : "Not refreshed"}
          variant={hasKey ? "ok" : "muted"}
        />

        <ProviderBlock
          actions={
            <>
              <IconButton icon={<RefreshCw size={14} />} label="Refresh" onClick={refreshOpenCode} />
              {opencodeQuotaConnected && (
                <div className="segmented" role="group" aria-label="OpenCode view">
                  <button
                    aria-pressed={opencodeView === "spend"}
                    onClick={() => setOpencodeView("spend")}
                    type="button"
                  >
                    Spend
                  </button>
                  <button
                    aria-pressed={opencodeView === "quota"}
                    onClick={() => setOpencodeView("quota")}
                    type="button"
                  >
                    Quota
                  </button>
                </div>
              )}
              {opencodeQuotaConnected ? (
                <IconButton
                  icon={<PlugZap size={14} />}
                  label="Disconnect"
                  onClick={disconnectOpenCodeQuota}
                />
              ) : (
                <IconButton
                  icon={<ChevronDown size={14} />}
                  label="Dev quota"
                  onClick={() => setShowOpencodeQuotaSetup((current) => !current)}
                />
              )}
            </>
          }
          extra={
            showOpencodeQuotaSetup &&
            !opencodeQuotaConnected && (
              <div className="form-grid quota-form">
                <input
                  aria-label="OpenCode workspace URL"
                  onChange={(event) => setOpencodeWorkspace(event.target.value)}
                  placeholder="Workspace URL or wrk_ id"
                  type="text"
                  value={opencodeWorkspace}
                />
                <input
                  aria-label="OpenCode cookie header"
                  onChange={(event) => setOpencodeCookie(event.target.value)}
                  placeholder="Cookie header"
                  type="password"
                  value={opencodeCookie}
                />
                <button
                  disabled={!opencodeCookie.trim() || !opencodeWorkspace.trim()}
                  onClick={saveOpenCodeQuota}
                  type="button"
                >
                  Save
                </button>
              </div>
            )
          }
          icon={<Braces size={16} />}
          metrics={opencodeLines}
          state={opencodeQuotaConnected ? (opencodeView === "quota" ? "Quota" : "Spend") : "Local spend"}
          title="OpenCode Go"
          updatedAt={opencodeSnapshot ? updatedLabel("opencode") : "Not refreshed"}
          variant={opencodeSnapshot ? "ok" : "muted"}
        />

        {placeholders.map((provider) => (
          <ProviderBlock
            icon={<Cloud size={16} />}
            key={provider}
            metrics={[]}
            state="Not connected"
            title={provider}
            updatedAt="Not refreshed"
            variant="muted"
          />
        ))}
      </section>

      <footer className={error ? "footer error" : "footer"}>
        {error || "Latest snapshots are stored locally"}
      </footer>
    </main>
  );
}

type ProviderBlockProps = {
  actions?: React.ReactNode;
  extra?: React.ReactNode;
  icon: React.ReactNode;
  metrics: MetricLine[];
  state: string;
  title: string;
  updatedAt: string;
  variant: "ok" | "muted";
};

function ProviderBlock({
  actions,
  extra,
  icon,
  metrics,
  state,
  title,
  updatedAt,
  variant,
}: ProviderBlockProps) {
  return (
    <div className="provider-block">
      <div className="provider-heading">
        <div className="provider-title">
          <span className="provider-icon">{icon}</span>
          <span>{title}</span>
        </div>
        <span className={`chip ${variant}`}>{variant === "ok" && <Check size={12} />}{state}</span>
      </div>

      {actions && <div className="action-row">{actions}</div>}
      {extra}

      {metrics.length > 0 && (
        <div className="metric-list">
          {metrics.map((line) => (
            <div className="metric-row" key={line.label}>
              <span>{line.label}</span>
              <strong>{line.value}</strong>
            </div>
          ))}
        </div>
      )}

      <div className="provider-foot">
        <Gauge size={12} />
        <span>{updatedAt}</span>
      </div>
    </div>
  );
}

type IconButtonProps = {
  disabled?: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
};

function IconButton({ disabled = false, icon, label, onClick }: IconButtonProps) {
  return (
    <button className="ghost-button" disabled={disabled} onClick={onClick} type="button">
      {icon}
      {label}
    </button>
  );
}

export default App;
