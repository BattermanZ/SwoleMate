export const meta = {
  name: 'swolemate-backend-audit',
  description: 'Resumable, adversarially-verified security + correctness audit of the SwoleMate Rust (actix-web + sqlx/SQLite) backend, per subsystem, severity-gated and checkpointed',
  phases: [
    { title: 'Scan', detail: 'detect which categories already have a final report on disk' },
    { title: 'Find', detail: 'one Opus-medium finder per remaining category (checkpointed to state/)', model: 'opus' },
    { title: 'Verify', detail: 'diverse-lens panel (code-truth / exploitability / already-mitigated) per category, severity-gated, Sonnet-medium', model: 'sonnet' },
    { title: 'Write', detail: 'Haiku scribe writes the final category report' },
    { title: 'Summarize', detail: 'regenerate SUMMARY.md launch-blocker rollup' },
  ],
}

const ROOT = '/home/battermanz/coding/SwoleMate'
const DIR = `${ROOT}/docs/audits/backend`
const STATE = `${DIR}/state`
const STACK = 'Rust, actix-web 4, sqlx 0.8 (SQLite, compile-time-checked macros), argon2 password hashing, actix-cors, tar+flate2 for backups. A session-cookie web client AND token-authed MCP clients (AI agents) both hit this backend.'

const CATEGORIES = [
  {
    slug: '01-authn-session',
    title: 'Password authentication & session lifecycle',
    scope: 'argon2 configuration/params and whether password verification is constant-time, session token generation & entropy (rand), session cookie flags (HttpOnly / Secure / SameSite), session expiry & rotation, logout/session invalidation, session fixation, and any user-enumeration or timing side-channels in login.',
    sources: 'server/src/auth/mod.rs, server/src/auth/password.rs, server/src/middleware/session_auth.rs, server/src/db/auth.rs, server/src/routes/auth.rs',
  },
  {
    slug: '02-oauth-flow',
    title: 'OAuth authorization flow',
    scope: 'CSRF protection via the state parameter, PKCE presence/verification, strict redirect_uri validation (open-redirect / redirect smuggling), authorization-code single-use & replay, token issuance & storage, scope enforcement, and whether an attacker can bind their code/token to a victim account.',
    sources: 'server/src/oauth/mod.rs, server/src/oauth/routes.rs, server/src/db/oauth.rs',
  },
  {
    slug: '03-mcp-token-auth',
    title: 'MCP token authentication & the MCP tool surface',
    scope: 'how MCP tokens are generated and whether they are hashed at rest (sha2), token scoping/expiry/revocation, mcp_auth middleware bypass, exactly which routes/tools an MCP token can reach, and whether an MCP token grants full user authority (privilege breadth for AI agents), plus token leakage in logs/errors.',
    sources: 'server/src/mcp/mod.rs, server/src/mcp/routes.rs, server/src/mcp/rate_limit.rs, server/src/middleware/mcp_auth.rs, server/src/db/mcp_tokens.rs, server/src/routes/mcp_tokens.rs',
  },
  {
    slug: '04-authz-ownership',
    title: 'Authorization / resource ownership (IDOR & privilege escalation)',
    scope: 'for every route that reads or mutates a workout, template, exercise, or progress record: is per-user ownership actually enforced (horizontal IDOR)? Are admin-only routes gated (vertical escalation)? Does authz run before the DB read, and does the MCP path go through the same checks? Look for handlers that trust an id from the request without an ownership predicate.',
    sources: 'server/src/services/authz.rs, server/src/routes/admin.rs, server/src/services/workouts.rs, server/src/services/templates.rs, server/src/services/progress.rs, server/src/services/exercises.rs, server/src/routes.rs',
  },
  {
    slug: '05-rate-limiting',
    title: 'Rate limiting (login & MCP)',
    scope: 'how limiters key requests (per-IP vs per-user, and whether the IP is taken from a spoofable X-Forwarded-For / peer addr), window & counter correctness, reset behaviour, bypass routes, unbounded memory growth (map keyed by attacker-controlled value = DoS), and concurrency races on the shared counter state.',
    sources: 'server/src/auth/rate_limit.rs, server/src/mcp/rate_limit.rs',
  },
  {
    slug: '06-backup-archive',
    title: 'Backup tar/gz creation, extraction & restore (path traversal)',
    scope: 'tar extraction path sanitization — zip-slip via ../ entries, absolute-path entries, and symlink entries that escape the target dir; gzip decompression-bomb / unbounded-size risk (flate2); WHO can trigger backup and especially restore (authz on the trigger route); arbitrary file overwrite on restore; and integrity/validation of the archive before it is applied.',
    sources: 'server/src/backup.rs, server/src/routes/admin.rs',
  },
  {
    slug: '07-sql-data-layer',
    title: 'SQL / sqlx data-access layer correctness & injection',
    scope: 'any dynamically string-built SQL vs parameterized queries / compile-time-checked macros (SQLi surface), transaction boundaries and atomicity of multi-statement writes, error leakage of raw DB messages, N+1 or unbounded result sets, and schema/type assumptions that could silently corrupt or truncate data.',
    sources: 'server/src/db/mod.rs, server/src/db/workouts.rs, server/src/db/progress.rs, server/src/db/templates.rs, server/src/db/exercises.rs, server/src/schema.rs',
  },
  {
    slug: '08-progress-consistency',
    title: 'Progress & consistency business-logic correctness',
    scope: 'the invariants progress_consistency is meant to enforce, aggregate/streak/volume math correctness, concurrent-update races (lost updates, double-count), ordering assumptions, off-by-one on set/rep/date boundaries, deletion cascades that leave orphaned or inconsistent rows, and idempotency of repeated writes (e.g. logging the same set twice).',
    sources: 'server/src/db/progress_consistency.rs, server/src/db/progress.rs, server/src/services/progress.rs, server/src/services/workouts.rs',
  },
  {
    slug: '09-error-input-handling',
    title: 'Error handling, input validation & information leakage',
    scope: 'whether error responses leak internals (DB errors, stack/debug, file paths) via errors.rs, panics/unwrap/expect on request-controlled input (panic = DoS), request-body validation on deserialized models, integer overflow / negative or absurd numeric inputs, uuid parsing failures, CORS configuration (actix-cors — is it permissive?), and missing request body-size limits.',
    sources: 'server/src/errors.rs, server/src/models.rs, server/src/routes.rs, server/src/main.rs, server/src/schema.rs',
  },
]

const FINDING_PROPS = {
  title: { type: 'string' },
  severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
  category: { type: 'string', enum: ['security', 'correctness'] },
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

// A panelist reviews many findings at once and returns one verdict per finding.
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
    panel: { type: 'array', items: { type: 'object' } }, // [{idx, verdicts:[...]}] — per-panelist checkpoints
    verified: { type: 'array', items: { type: 'object' } },
  },
}

const OK_SCHEMA = { type: 'object', required: ['ok'], properties: { ok: { type: 'boolean' } } }

function finderPrompt(cat) {
  return `You are auditing the SwoleMate BACKEND for SECURITY and CORRECTNESS defects before a public launch.

Stack: ${STACK}
Repo root: ${ROOT} (this repo is indexed by CodeGraph — use codegraph_explore to pull verbatim source + call paths, then Read specific line ranges).

Category: ${cat.title}
Scope: ${cat.scope}
Primary sources to read: ${cat.sources}

Find REAL defects that exist in THIS code — not generic Rust/web lore. Every finding MUST cite an actual file and line you have read. For each, state whether it is a "security" or "correctness" issue, and describe the concrete attack or trigger path (who calls what, through which route/middleware, to make it bite). Prefer fewer, concrete, code-grounded findings over a long speculative list. If a sub-area is genuinely solid, do not invent issues. Severity reflects launch impact: critical = remote auth bypass / data loss / arbitrary file write / privilege escalation; high = exploitable IDOR / injection / DoS; medium = weaker hardening gap or a correctness bug with real user impact; low = defense-in-depth / minor.

Remember middleware order matters: a finding is only real if the vulnerable code is actually reachable given the auth/session/mcp middleware guarding that route.

CHECKPOINT: After analysis, use the Write tool to save the findings as {"findings": [...]} to ${STATE}/${cat.slug}.findings.json (the Write tool creates parent dirs). Then return the same object via the schema.`
}

// Each panelist reviews through ONE independent lens, so the three votes catch
// different failure modes instead of redundantly agreeing (diverse-lens panel).
const LENSES = {
  1: {
    name: 'code-truth',
    focus: `LENS — CODE TRUTH. Read the exact cited lines and decide whether the Rust, as written, actually does what the finding claims. Refute (refuted=true) if: the cited line/symbol is wrong or stale, the claim misreads the control flow / ownership / async behaviour, the mechanism simply isn't in the code, or an in-function guard/branch/? -operator already prevents it. You are judging "is the mechanism real in this source?" — nothing else.`,
  },
  2: {
    name: 'exploitability',
    focus: `LENS — EXPLOITABILITY / REACHABILITY. Assume the code is as described; decide whether a real attacker or user can actually reach and trigger it. Refute (refuted=true) if: the vulnerable handler is gated by session_auth / mcp_auth / an admin check that the finding ignores, the trigger path can't occur for a real request, the "attacker-controlled" input is actually server-controlled or validated upstream, or the ownership/authz layer already blocks the described request before it lands. You are judging "can this actually be triggered end-to-end?" — not whether the code is locally imperfect.`,
  },
  3: {
    name: 'already-mitigated',
    focus: `LENS — ALREADY MITIGATED / SEVERITY. Decide whether the case is already handled or overstated. Refute (refuted=true) if: a guard, middleware, DB constraint (NOT NULL / FK / UNIQUE), sqlx compile-time-checked query, error mapper, or the upstream crate already covers it, another code path makes it unreachable, or the impact is trivial/recoverable so the severity is wrong. You MAY glance at the cited files' imports and the schema to confirm an existing guard, but do NOT explore the wider repo.`,
  },
}

// Batched panelist: apply ONE lens to a list of findings in a single read pass.
function panelPrompt(cat, subset, idx) {
  const lens = LENSES[idx] || LENSES[1]
  const list = subset
    .map((f, n) => `${n + 1}. [${f.severity}/${f.category}] "${f.title}"\n   at ${f.file}:${f.line}\n   claim: ${f.description}\n   attack/trigger: ${f.attackOrTrigger}\n   reason given: ${f.why}`)
    .join('\n\n')
  const cited = [...new Set(subset.map((f) => f.file))].join(', ')
  return `You are reviewer #${idx} (lens: ${lens.name}) on a panel verifying backend security/correctness findings for SwoleMate (category: ${cat.title}). Repo root: ${ROOT}.

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
  // --- Load checkpointed state (cheap Haiku reader) ---
  const st = await agent(
    `Read these three files if they exist (use: cat <file> 2>/dev/null) and parse each as JSON.
- ${STATE}/${cat.slug}.findings.json -> {"findings":[...]} (or a bare array)
- ${STATE}/${cat.slug}.panel.json -> an array of {"idx":N,"verdicts":[...]} per-panelist checkpoints
- ${STATE}/${cat.slug}.verdicts.json -> an array of already-verified finding objects
Return {"findings": <findings array or []>, "panel": <panel array or []>, "verified": <verified array or []>}. Missing/empty file -> []. Do not invent data.`,
    { label: `load:${cat.slug}`, phase: 'Find', model: 'haiku', effort: 'low', schema: STATE_SCHEMA },
  )

  let findings = (st && st.findings) || []
  let panel = (st && st.panel) || [] // [{idx, verdicts}]
  let verified = (st && st.verified) || []

  // --- Stage 1: find (Opus-medium), only if not checkpointed ---
  if (findings.length === 0) {
    log(`▶ ${cat.slug}: finding (Opus-medium)…`)
    const r = await agent(finderPrompt(cat), { label: `find:${cat.slug}`, phase: 'Find', model: 'opus', effort: 'medium', schema: FINDINGS_SCHEMA })
    findings = (r && r.findings) || []
  } else {
    log(`▶ ${cat.slug}: resumed — ${findings.length} findings, ${panel.length} panelist(s) done, verified=${verified.length}`)
  }

  // --- Stage 2: batched, severity-gated adversarial panel (only if not done) ---
  if (verified.length === 0 && findings.length > 0) {
    const sev = (f) => f.severity
    const highCrit = findings.filter((f) => sev(f) === 'critical' || sev(f) === 'high')
    const medium = findings.filter((f) => sev(f) === 'medium')

    // Panelist subsets: p1 (code-truth lens) covers high/crit + medium (medium gets that 1 lens);
    // p2 (exploitability) and p3 (already-mitigated) cover high/crit only => 3 diverse lenses on high/crit.
    const panelists = []
    if (highCrit.length > 0) {
      panelists.push({ idx: 1, subset: [...highCrit, ...medium] })
      panelists.push({ idx: 2, subset: highCrit })
      panelists.push({ idx: 3, subset: highCrit })
    } else if (medium.length > 0) {
      panelists.push({ idx: 1, subset: medium })
    }
    log(`  ${cat.slug}: ${highCrit.length} high/crit (3 diverse lenses), ${medium.length} medium (code-truth lens), ${findings.length - highCrit.length - medium.length} low (unverified); ${panelists.length} panelist(s)`)

    // SEQUENTIAL: run each panelist one at a time, checkpoint panel.json after each,
    // so an interrupt loses at most a single panelist.
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

    // Aggregate verdicts across panelists by finding title.
    const verdictsFor = (title) => panel.map((pr) => (pr.verdicts || []).find((v) => v.title === title)).filter(Boolean)
    verified = findings.map((f) => {
      const s = sev(f)
      if (s === 'low') return { ...f, survives: true, status: 'unverified', refutes: 0, votes: 0, verdicts: [] }
      const vs = verdictsFor(f.title)
      const refutes = vs.filter((v) => v.refuted).length
      const needed = s === 'medium' ? 1 : 2 // medium: 1 refute kills; high/crit: majority of 3
      return { ...f, survives: refutes < needed, status: 'verified', refutes, votes: vs.length, verdicts: vs }
    })

    // checkpoint final aggregated verdicts (verify-complete marker)
    const blob = JSON.stringify(verified, null, 2)
    await agent(
      `Use the Write tool to write the following EXACT text to ${STATE}/${cat.slug}.verdicts.json. Copy byte-for-byte: do not change, summarize, reformat, or comment. Then return {"ok": true}.

<<<JSON
${blob}
JSON`,
      { label: `ckpt:${cat.slug}`, phase: 'Verify', model: 'haiku', effort: 'low', schema: OK_SCHEMA },
    )
  }

  // --- Stage 3: final report (Haiku) ---
  const confirmed = verified.filter((v) => v.survives)
  const rejected = verified.filter((v) => !v.survives)
  const payload = JSON.stringify({ confirmed, rejected }, null, 2)
  await agent(
    `Write a markdown audit report to ${DIR}/${cat.slug}.md using the Write tool. Category title: "${cat.title}".

Structure:
- H1 with the category title.
- One-line summary: N confirmed (by severity), M refuted. Note that LOW findings are included as "unverified" (panel did not vote on them).
- H2 "Confirmed findings": for EACH confirmed finding an H3 "SEVERITY [security|correctness]: title" (append " (unverified)" if its status is unverified), then bullets: Attack/trigger, Location (\`file:line\`), What happens, Why, Fix sketch. Order critical > high > medium > low.
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
