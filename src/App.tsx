import "./App.css";

const providers = ["Codex", "Claude / Claude Code", "OpenCode Go", "Antigravity"];

function App() {
  return (
    <main className="panel">
      <header className="panel-header">
        <div>
          <h1>InfUsage</h1>
          <p>AI usage from the Windows tray</p>
        </div>
        <span className="status">Idle</span>
      </header>

      <section className="provider-list" aria-label="Providers">
        {providers.map((provider) => (
          <div className="provider-row" key={provider}>
            <span>{provider}</span>
            <span className="muted">Not connected</span>
          </div>
        ))}
      </section>

      <footer>No providers connected yet.</footer>
    </main>
  );
}

export default App;
