export const meta = {
  name: 'swolemate-frontend-audit',
  description: 'Resumable, adversarially-verified data-integrity + client-security audit of the SwoleMate SvelteKit (Svelte 5 runes, offline-first PWA) frontend, per subsystem, severity-gated and checkpointed',
  phases: [
    { title: 'Scan', detail: 'detect which categories already have a final report on disk' },
    { title: 'Find', detail: 'one Opus-medium finder per remaining category (checkpointed to state/)', model: 'opus' },
    { title: 'Verify', detail: 'diverse-lens panel (code-truth / reachability / already-mitigated) per category, severity-gated, Sonnet-medium', model: 'sonnet' },
    { title: 'Write', detail: 'Haiku scribe writes the final category report' },
    { title: 'Summarize', detail: 'regenerate SUMMARY.md launch-blocker rollup' },
  ],
}

const ROOT = '/home/battermanz/coding/SwoleMate'
const CLIENT = `${ROOT}/client`
const DIR = `${ROOT}/docs/audits/frontend`
const STATE = `${DIR}/state`
const STACK = 'SvelteKit 2 + Svelte 5 (runes: $state/$derived/$effect), adapter-static (SPA + service worker), Tailwind 4, Chart.js, TypeScript. Offline-first PWA: an in-progress workout session is a client-side state engine persisted to IndexedDB/localStorage and synced to the Rust backend on reconnect. All backend calls go through a single api.ts seam (dev-proxied /api, /mcp, /oauth). The real authorization gate lives in the backend; client route guards are UX/data-leak surface, not the enforcement boundary.'

// Paths are relative to the client package unless noted; exact locations under
// client/src/lib — finders use codegraph_explore to resolve them precisely.
const CATEGORIES = [
  {
    slug: '01-session-engine-state',
    title: 'Live workout-session state engine',
    scope: 'state-machine correctness of the live workout logger: rune $state/$derived integrity, action reducers, invariant preservation across actions, rapid-action races (double-tap / concurrent mutations), and PR / weight-mode / set-volume math correctness. A bug here means a logged set is silently lost or mis-recorded.',
    sources: 'client/src/lib/today/controller/state.ts, client/src/lib/today/controller/index.ts, client/src/lib/today/controller/metrics.ts, client/src/lib/today/controller/actions/session.ts, client/src/lib/today/controller/actions/exercise/*.ts, client/src/lib/today/controller/actions/*.ts',
  },
  {
    slug: '02-offline-persistence',
    title: 'Offline persistence & recovery of in-progress sessions',
    scope: 'durability of in-progress sessions: write timing (a debounce/throttle window = data lost on crash/reload), quota handling and serialization/migration of the stored session shape (version skew corrupting on read), and recovery correctness when the app reloads mid-session.',
    sources: 'client/src/lib/offline/todaySessions.ts, client/src/lib/offline/storage.ts, client/src/lib/pwa/persistentStorage.ts, client/src/lib/today/controller/offline.ts',
  },
  {
    slug: '03-sync-reconnect',
    title: 'Sync-on-reconnect / offline mutation queue',
    scope: 'the offline-mutation replay queue: replay ordering, IDEMPOTENCY (can a set be double-logged on retry?), server-divergence / conflict resolution, partial-failure handling (some mutations applied, some not), retry storms, and correctness across online<->offline transitions.',
    sources: 'client/src/lib/today/controller/sync.ts, client/src/lib/today/controller/actions/sync.ts, client/src/lib/today/backend.ts, client/src/lib/today/controller/actions/backend.ts, client/src/lib/stores/network.ts',
  },
  {
    slug: '04-api-seam',
    title: 'API seam (api.ts backend contract)',
    scope: 'the api.ts contract: backend ErrorResponse parsing, handling of 401/403/409/5xx, request abort/timeout, JSON-parse failure on a malformed body, credential/cookie handling (credentials: include), and any silent error-swallow that turns a failed write into an apparent success.',
    sources: 'client/src/lib/api.ts, client/src/lib/stores/network.ts, client/src/lib/logger.ts',
  },
  {
    slug: '05-auth-scope-guard',
    title: 'Client auth / scope / route guards',
    scope: 'client session+scope state, client-only route guards (data-leak on flash-of-content vs guard bypass), 401 -> redirect behaviour, whether client scope matches the backend scope model, and demo-mode bypass paths. NOTE: treat a bypassable client guard as low/medium — the backend authz is the real gate; this flags data-leak/UX, not enforcement.',
    sources: 'client/src/lib/auth/index.ts, client/src/lib/auth/scope.ts, client/src/lib/preferences/demoMode.ts, client/src/routes/login/+page.svelte, client/src/routes/+layout.svelte',
  },
  {
    slug: '06-xss-rendering',
    title: 'XSS / unsafe rendering',
    scope: 'every {@html} use, unsanitized user/exercise/template-supplied text rendered as HTML, Chart.js tooltip HTML, javascript:/data: URL construction from user input, and SVG injection. Svelte auto-escapes text interpolation, so this hunts the EXCEPTIONS to that (raw HTML, bound href/src, dangerous sinks).',
    sources: 'grep for {@html} across client/src; client/src/lib/components/ui/*, client/src/lib/progress/*, and the help / templates / workouts / exercises route pages',
  },
  {
    slug: '07-token-secret-storage',
    title: 'Token / secret storage & leakage',
    scope: 'what sensitive data lands in localStorage / IndexedDB / the SW cache, sensitive values written by logger.ts (tokens, creds in log lines), MCP-token display/copy handling, demo credentials in source, secrets placed in URLs/query strings, and exposure via the service-worker response cache.',
    sources: 'client/src/lib/api.ts, client/src/lib/logger.ts, client/src/lib/offline/storage.ts, client/src/routes/settings/+page.svelte, client/src/lib/stores/whatsNew.ts',
  },
  {
    slug: '08-service-worker-pwa',
    title: 'Service worker / PWA cache lifecycle',
    scope: 'SW cache staleness across deploys (users stuck on old assets), offline navigation fallback correctness, API-response cache poisoning/serving stale mutations, an update flow that forces a reload MID-SESSION (data loss on the live logger), and install/activate lifecycle correctness.',
    sources: 'client/svelte.config.js (serviceWorker config), client/src/service-worker.* if present, client/src/lib/pwa/persistentStorage.ts, client/vite.config.ts',
  },
]

const FINDING_PROPS = {
  title: { type: 'string' },
  severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
  category: { type: 'string', enum: ['data-integrity', 'security'] },
  file: { type: 'string' },
  line: { type: 'string' },
  description: { type: 'string' },
  attackOrTrigger: { type: 'string' },
  why: { type: 'string' },
  fixSketch: { type: 'string' },
}

const FINDINGS_SCHEMA = {
  type: 'object',
  required: ['findings'],
  properties: {
    findings: { type: 'array', items: { type: 'object', required: Object.keys(FINDING_PROPS), properties: FINDING_PROPS } },
  },
}

const PANEL_SCHEMA = {
  type: 'object',
  required: ['verdicts'],
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object',
        required: ['title', 'refuted', 'reason'],
        properties: { title: { type: 'string' }, refuted: { type: 'boolean' }, reason: { type: 'string' } },
      },
    },
  },
}

const STATE_SCHEMA = {
  type: 'object',
  required: ['findings', 'panel', 'verified'],
  properties: {
    findings: { type: 'array', items: { type: 'object' } },
    panel: { type: 'array', items: { type: 'object' } },
    verified: { type: 'array', items: { type: 'object' } },
  },
}

const OK_SCHEMA = { type: 'object', required: ['ok'], properties: { ok: { type: 'boolean' } } }

function finderPrompt(cat) {
  return `You are auditing the SwoleMate FRONTEND for DATA-INTEGRITY and CLIENT-SECURITY defects before a public launch.

Stack: ${STACK}
Repo root: ${ROOT}; client package: ${CLIENT} (this repo is indexed by CodeGraph — use codegraph_explore to pull verbatim source + call paths, then Read specific line ranges).

Category: ${cat.title}
Scope: ${cat.scope}
Primary sources to read: ${cat.sources}

Find REAL defects that exist in THIS code — not generic Svelte/web lore. Every finding MUST cite an actual file and line you have read. For each, label it "data-integrity" (a user's logged data is silently lost, duplicated, corrupted, or mis-computed) or "security" (XSS, token/secret exposure, client-guard data-leak). Describe the concrete trigger path: which route/state, online vs offline, what user action or reconnect sequence makes it bite. Prefer fewer, concrete, code-grounded findings over a long speculative list. If a sub-area is genuinely solid, do not invent issues.

Severity reflects launch impact: critical = silent loss/corruption of a user's logged workout, or a real XSS/secret leak; high = data duplicated or lost under a plausible offline/reconnect sequence, or scope/guard data-leak; medium = correctness bug with real but recoverable user impact, or a weaker hardening gap; low = defense-in-depth / minor / cosmetic.

Reactivity reality-check: a finding is only real if the Svelte 5 rune/store behaviour actually produces it — account for $derived recomputation, $effect timing, and store-subscription lifecycle. Remember the backend is the real authorization gate: do NOT rate a bypassable client-only route guard as high/critical on enforcement grounds; rate it on data-leak/UX.

CHECKPOINT: After analysis, use the Write tool to save the findings as {"findings": [...]} to ${STATE}/${cat.slug}.findings.json (the Write tool creates parent dirs). Then return the same object via the schema.`
}

const LENSES = {
  1: {
    name: 'code-truth',
    focus: `LENS — CODE TRUTH (Svelte 5 / TS). Read the exact cited lines and decide whether the code, as written, actually does what the finding claims. Refute (refuted=true) if: the cited line/symbol is wrong or stale, the claim misreads rune reactivity ($state/$derived recomputation, $effect timing), store subscription lifecycle, async/await ordering, or the mechanism simply isn't in the code, or an in-function guard/early-return already prevents it. You are judging "is the mechanism real in this source?" — nothing else.`,
  },
  2: {
    name: 'reachability',
    focus: `LENS — REACHABILITY / REPRO. Assume the code is as described; decide whether a real user can actually hit it in the running app. Refute (refuted=true) if: no reachable route/state produces the precondition, the online/offline transition or action sequence can't actually occur, the "user/attacker-controlled" input is actually app-controlled or validated upstream, or the guard/redirect the finding ignores already blocks the flow. For security items: is the input genuinely attacker-controlled (e.g. exercise/template text a victim would render)? You are judging "can a real user trigger this end-to-end?"`,
  },
  3: {
    name: 'already-mitigated',
    focus: `LENS — ALREADY MITIGATED / SEVERITY. Decide whether the case is already handled or overstated. Refute (refuted=true) if: Svelte's DEFAULT auto-escaping already neutralizes it (no {@html}/no dangerous sink), a sanitizer / try-catch / existing guard / debounce-flush-on-unload / idempotency key already covers it, another code path makes it unreachable, the backend already prevents the bad outcome, or the impact is trivial/recoverable so the severity is inflated. You MAY glance at the cited files' imports to confirm an existing guard, but do NOT explore the wider repo.`,
  },
}

function panelPrompt(cat, subset, idx) {
  const lens = LENSES[idx] || LENSES[1]
  const list = subset
    .map((f, n) => `${n + 1}. [${f.severity}/${f.category}] "${f.title}"\n   at ${f.file}:${f.line}\n   claim: ${f.description}\n   trigger: ${f.attackOrTrigger}\n   reason given: ${f.why}`)
    .join('\n\n')
  const cited = [...new Set(subset.map((f) => f.file))].join(', ')
  return `You are reviewer #${idx} (lens: ${lens.name}) on a panel verifying frontend data-integrity/security findings for SwoleMate (category: ${cat.title}). Repo root: ${ROOT}, client: ${CLIENT}.

EFFICIENCY: read ONLY the specific cited files/line ranges below — do NOT explore the wider codebase. The cited files are: ${cited}. Read each one once; it backs several findings.

Stack reminder: ${STACK}

${lens.focus}

Judge each finding THROUGH YOUR LENS ONLY. Set refuted=true only when your lens exposes a genuine reason the finding is not a real, reachable defect. Set refuted=false if, from your lens, it holds up — or if your lens is simply not the right angle to judge it (another reviewer covers the other angles). Do not refute for reasons outside your lens.

Findings to review:
${list}

Return one verdict per finding (match by exact title): {title, refuted, reason}. State your lens's specific reason.`
}

phase('Scan')
const scan = await agent(
  `List the files in ${DIR} (use: ls -la ${DIR} 2>/dev/null). Return the set of category slugs whose "<slug>.md" FINAL report file already exists and is non-empty. Candidate slugs: ${CATEGORIES.map((c) => c.slug).join(', ')}. If the directory does not exist yet, return an empty done list.`,
  { label: 'scan-ledger', phase: 'Scan', model: 'haiku', effort: 'low', schema: { type: 'object', required: ['done'], properties: { done: { type: 'array', items: { type: 'string' } } } } },
)

const doneSet = new Set((scan && scan.done) || [])
const remaining = CATEGORIES.filter((c) => !doneSet.has(c.slug))
log(`${doneSet.size} categories complete; running ${remaining.length}: ${remaining.map((c) => c.slug).join(', ') || '(none)'}`)

for (const cat of remaining) {
  const st = await agent(
    `Read these three files if they exist (use: cat <file> 2>/dev/null) and parse each as JSON.
- ${STATE}/${cat.slug}.findings.json -> {"findings":[...]} (or a bare array)
- ${STATE}/${cat.slug}.panel.json -> an array of {"idx":N,"verdicts":[...]} per-panelist checkpoints
- ${STATE}/${cat.slug}.verdicts.json -> an array of already-verified finding objects
Return {"findings": <findings array or []>, "panel": <panel array or []>, "verified": <verified array or []>}. Missing/empty file -> []. Do not invent data.`,
    { label: `load:${cat.slug}`, phase: 'Find', model: 'haiku', effort: 'low', schema: STATE_SCHEMA },
  )

  let findings = (st && st.findings) || []
  let panel = (st && st.panel) || []
  let verified = (st && st.verified) || []

  if (findings.length === 0) {
    log(`▶ ${cat.slug}: finding (Opus-medium)…`)
    const r = await agent(finderPrompt(cat), { label: `find:${cat.slug}`, phase: 'Find', model: 'opus', effort: 'medium', schema: FINDINGS_SCHEMA })
    findings = (r && r.findings) || []
  } else {
    log(`▶ ${cat.slug}: resumed — ${findings.length} findings, ${panel.length} panelist(s) done, verified=${verified.length}`)
  }

  if (verified.length === 0 && findings.length > 0) {
    const sev = (f) => f.severity
    const highCrit = findings.filter((f) => sev(f) === 'critical' || sev(f) === 'high')
    const medium = findings.filter((f) => sev(f) === 'medium')

    const panelists = []
    if (highCrit.length > 0) {
      panelists.push({ idx: 1, subset: [...highCrit, ...medium] })
      panelists.push({ idx: 2, subset: highCrit })
      panelists.push({ idx: 3, subset: highCrit })
    } else if (medium.length > 0) {
      panelists.push({ idx: 1, subset: medium })
    }
    log(`  ${cat.slug}: ${highCrit.length} high/crit (3 diverse lenses), ${medium.length} medium (code-truth lens), ${findings.length - highCrit.length - medium.length} low (unverified); ${panelists.length} panelist(s)`)

    const doneIdx = new Set(panel.map((p) => p.idx))
    for (const p of panelists) {
      if (doneIdx.has(p.idx)) continue
      const r = await agent(panelPrompt(cat, p.subset, p.idx), { label: `panel:${cat.slug}#${p.idx}`, phase: 'Verify', model: 'sonnet', effort: 'medium', schema: PANEL_SCHEMA })
      panel.push({ idx: p.idx, verdicts: (r && r.verdicts) || [] })

      const panelBlob = JSON.stringify(panel, null, 2)
      await agent(
        `Use the Write tool to write the following EXACT text to ${STATE}/${cat.slug}.panel.json. Copy byte-for-byte: do not change, summarize, reformat, or comment. Then return {"ok": true}.

<<<JSON
${panelBlob}
JSON`,
        { label: `ckpt-panel:${cat.slug}#${p.idx}`, phase: 'Verify', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
      )
      log(`    ${cat.slug}: panelist ${p.idx}/${panelists.length} done & checkpointed`)
    }

    const verdictsFor = (title) => panel.map((pr) => (pr.verdicts || []).find((v) => v.title === title)).filter(Boolean)
    verified = findings.map((f) => {
      const s = sev(f)
      if (s === 'low') return { ...f, survives: true, status: 'unverified', refutes: 0, votes: 0, verdicts: [] }
      const vs = verdictsFor(f.title)
      const refutes = vs.filter((v) => v.refuted).length
      const needed = s === 'medium' ? 1 : 2
      return { ...f, survives: refutes < needed, status: 'verified', refutes, votes: vs.length, verdicts: vs }
    })

    const blob = JSON.stringify(verified, null, 2)
    await agent(
      `Use the Write tool to write the following EXACT text to ${STATE}/${cat.slug}.verdicts.json. Copy byte-for-byte: do not change, summarize, reformat, or comment. Then return {"ok": true}.

<<<JSON
${blob}
JSON`,
      { label: `ckpt:${cat.slug}`, phase: 'Verify', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
    )
  }

  const confirmed = verified.filter((v) => v.survives)
  const rejected = verified.filter((v) => !v.survives)
  const payload = JSON.stringify({ confirmed, rejected }, null, 2)
  await agent(
    `Write a markdown audit report to ${DIR}/${cat.slug}.md using the Write tool. Category title: "${cat.title}".

Structure:
- H1 with the category title.
- One-line summary: N confirmed (by severity), M refuted. Note that LOW findings are included as "unverified" (panel did not vote on them).
- H2 "Confirmed findings": for EACH confirmed finding an H3 "SEVERITY [data-integrity|security]: title" (append " (unverified)" if its status is unverified), then bullets: Trigger, Location (\`file:line\`), What happens, Why, Fix sketch. Order critical > high > medium > low.
- H2 "Refuted (not real / already handled)": each rejected finding title + one-line reason from its verdicts.

Data (JSON, each finding has severity/category/status/verdicts):
${payload}

After writing, return {"ok": true}.`,
    { label: `write:${cat.slug}`, phase: 'Write', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
  )
  log(`✔ ${cat.slug}: banked (${confirmed.length} confirmed, ${rejected.length} refuted)`)
}

phase('Summarize')
const SUMMARY_SCHEMA = { type: 'object', required: ['markdown'], properties: { markdown: { type: 'string' } } }
const summaryGenPrompt = `Read every NN-*.md category report in ${DIR} (files named NN-*.md, NOT SUMMARY.md, NOT anything under state/). Produce a launch-readiness rollup as MARKDOWN: a table of all confirmed findings across categories sorted by severity (columns: Severity, Type, Category, Title, Location), plus a "Top launch blockers" section listing the critical/high items with one line each. Mark unverified (low) rows as such. Keep it concise. Do NOT write any file — RETURN the full markdown text as {"markdown": "..."}.`

// Split "read/think" from "write": the generator only RETURNS markdown (schema-forced),
// so it cannot silently no-op. Retry once on empty.
let summaryMd = ''
for (let attempt = 1; attempt <= 2 && !summaryMd.trim(); attempt++) {
  const s = await agent(summaryGenPrompt, { label: `summary-gen#${attempt}`, phase: 'Summarize', model: 'haiku', effort: 'low', schema: SUMMARY_SCHEMA })
  summaryMd = (s && s.markdown) || ''
  if (!summaryMd.trim()) log(`summary generator returned empty (attempt ${attempt})`)
}

// Write, then VERIFY on disk with an independent ground-truth check (wc -c) — a write agent
// can return {ok:true} without ever calling Write (observed), so never trust its own report.
// Retry the write until the bytes are actually present.
const target = `${DIR}/SUMMARY.md`
let summaryWritten = false
if (summaryMd.trim()) {
  for (let attempt = 1; attempt <= 3 && !summaryWritten; attempt++) {
    await agent(
      `Use the Write tool to write the EXACT text between the markers to ${target} — byte-for-byte, no changes, no commentary, no code fence. Then Read ${target} back to confirm it is present. Return {"ok": true}.

<<<MARKDOWN
${summaryMd}
MARKDOWN`,
      { label: `summary-write#${attempt}`, phase: 'Summarize', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
    )
    const chk = await agent(
      `Run exactly this shell command: wc -c < "${target}" 2>/dev/null || echo 0 . Return {"ok": true} if the printed byte count is greater than 50, otherwise {"ok": false}. Do not write or modify anything.`,
      { label: `summary-verify#${attempt}`, phase: 'Summarize', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
    )
    summaryWritten = !!(chk && chk.ok)
    if (!summaryWritten) log(`SUMMARY.md not confirmed on disk (attempt ${attempt}) — retrying write`)
  }
  log(summaryWritten ? `SUMMARY.md written & verified (${summaryMd.length} chars)` : 'ERROR: SUMMARY.md could not be written after 3 attempts — regenerate manually from the NN-*.md reports')
} else {
  log('WARNING: summarizer returned empty markdown twice; SUMMARY.md NOT written — regenerate manually from the NN-*.md reports')
}

return { done: [...doneSet], ran: remaining.map((c) => c.slug), summaryWritten }
