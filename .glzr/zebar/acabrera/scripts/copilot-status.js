// Reads local GitHub Copilot CLI state (session-store.db + open-sessions-state.json)
// and prints a JSON summary of active sessions + token/AIU consumption.
// Run with: node --experimental-sqlite copilot-status.js
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { DatabaseSync } = require('node:sqlite');

const COPILOT_DIR = path.join(os.homedir(), '.copilot');
const DB_PATH = path.join(COPILOT_DIR, 'session-store.db');
const OPEN_SESSIONS_PATH = path.join(COPILOT_DIR, 'open-sessions-state.json');

// Only consider sessions whose "open" state was refreshed within this window,
// so closed/stale CLI instances drop off automatically.
const ACTIVE_WINDOW_MS = 2 * 60 * 60 * 1000; // 2 hours

function readOpenSessions() {
  if (!fs.existsSync(OPEN_SESSIONS_PATH)) return {};
  try {
    return JSON.parse(fs.readFileSync(OPEN_SESSIONS_PATH, 'utf8'));
  } catch {
    return {};
  }
}

function main() {
  const openSessions = readOpenSessions();
  const now = Date.now();

  const activeIds = Object.entries(openSessions)
    .filter(([, meta]) => {
      const refreshedAt = Date.parse(meta.refreshedAt ?? meta.openedAt ?? 0);
      return Number.isFinite(refreshedAt) && now - refreshedAt <= ACTIVE_WINDOW_MS;
    })
    .map(([id]) => id);

  if (!fs.existsSync(DB_PATH) || activeIds.length === 0) {
    console.log(JSON.stringify({ totalAiu: 0, sessions: [] }));
    return;
  }

  const db = new DatabaseSync(DB_PATH, { readOnly: true });

  const sessionStmt = db.prepare(
    'SELECT id, repository, cwd, summary, updated_at FROM sessions WHERE id = ?'
  );
  const usageStmt = db.prepare(
    `SELECT SUM(total_nano_aiu) AS totalNanoAiu, COUNT(*) AS turnCount
     FROM assistant_usage_events WHERE session_id = ?`
  );
  const lastModelStmt = db.prepare(
    `SELECT model, reasoning_effort FROM assistant_usage_events
     WHERE session_id = ? ORDER BY id DESC LIMIT 1`
  );

  const sessions = [];
  let totalAiu = 0;

  for (const id of activeIds) {
    const meta = openSessions[id] ?? {};
    const row = sessionStmt.get(id);
    const usage = usageStmt.get(id);
    const modelRow = lastModelStmt.get(id);

    const aiu = usage?.totalNanoAiu ? usage.totalNanoAiu / 1e9 : 0;
    totalAiu += aiu;

    const repo = row?.repository || (row?.cwd ? path.basename(row.cwd) : 'unknown');

    sessions.push({
      id,
      repo,
      summary: row?.summary ?? null,
      model: modelRow?.model ?? null,
      reasoningEffort: modelRow?.reasoning_effort ?? null,
      aiu: Math.round(aiu * 100) / 100,
      turnCount: usage?.turnCount ?? 0,
      working: !!meta.working,
      refreshedAt: meta.refreshedAt ?? meta.openedAt ?? null,
    });
  }

  db.close();

  sessions.sort((a, b) => (b.refreshedAt ?? '').localeCompare(a.refreshedAt ?? ''));

  console.log(
    JSON.stringify({
      totalAiu: Math.round(totalAiu * 100) / 100,
      sessions,
    })
  );
}

main();
