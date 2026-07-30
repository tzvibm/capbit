# Wing — The Coding-Agent Model, Applied to Dating

**July 2026 · Modelling the match thread on Claude Code: memory files, skills, tools**

The proposal: each match thread gets something like a `CLAUDE.md` holding goals and state, plus its own memory, plus skills and tools invocable at any point in the conversation. Model the coding-agent pattern, for dating.

**This is the right architecture, and the mapping is tighter than it first looks.** Three of the four pieces transfer almost unchanged, and one of them — skills with progressive disclosure — solves a token-budget problem `CONTEXT-STRATEGY.md` §8 left open.

**But there is one structural asymmetry between coding and dating that changes what the harness has to do**, and it is the most important thing in this document. Claude Code works because it can check its own work. Dating has no compiler (§7).

---

# PART I — WHAT TRANSFERS

## 1. The memory hierarchy maps exactly

Claude Code loads memory from four scopes — managed policy, user (`~/.claude/CLAUDE.md`), project (`./CLAUDE.md`), and local — **concatenated rather than overriding**, broadest first, closest to the working directory last. Files above the working directory load in full at launch; files in subdirectories **load on demand** when Claude reads into them ([Claude Code docs](https://code.claude.com/docs/en/memory)).

That is precisely the typing already specified in `AI-LANDSCAPE.md` §22, arrived at independently from a privacy argument. Pleasing when two lines of reasoning land on the same structure:

| Claude Code | Wing | Loaded |
|---|---|---|
| `~/.claude/CLAUDE.md` (user scope) | **`you/MEMORY.md`** — how this person writes, what they always get wrong, their goals | **Always**, every thread |
| `./CLAUDE.md` (project scope) | **`matches/<id>/MATCH.md`** — this match's goals, state, what's been tried, what happened | Only in this match's thread |
| Subdirectory, on demand | Per-job working notes | On job invocation |
| Managed policy | Platform rules, safety boundaries, the no-impersonation rule | Always, unwritable by user |

Two properties worth copying deliberately:

- **Concatenation, not override.** The user-scope file always applies; the match file adds. A coach reading the handoff gets both.
- **"Claude Code won't carry memories from one project into another unless they're in user scope."** That is the *one match per context* rule from `CONTEXT-STRATEGY.md` §8, enforced by the same mechanism — scope isolation rather than prompt instruction. The cross-attribution failure mode (Jamie's dog reported as Priya's) becomes structurally impossible.

## 2. Skills with progressive disclosure — the biggest unlock

This is the piece that solves a problem I had not solved.

`MATCH-THREADS-DESIGN.md` §6 specifies six job types, each needing its own playbook, heuristics and output format. Stuffing all six playbooks into every request is exactly the context bloat `CONTEXT-STRATEGY.md` warns against. Progressive disclosure resolves it:

> At startup only each skill's **name and description** load — roughly **30–100 tokens per skill**. The full `SKILL.md` body stays on disk until triggered. Reported effect: **70,000 tokens of always-loaded documentation reduced to a ~500-token index — a 140× difference** ([Claude Platform Docs](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview), [DEV](https://dev.to/jimquote/claude-skills-vs-mcp-complete-guide-to-token-efficient-ai-agent-architecture-4mkf)). Bodies are recommended under 500 lines; 5–10 skills is typical and 30+ works without context issues.

Applied here:

```
skills/
  convo-rescue/SKILL.md      "A conversation has stalled or gone cold"
  opener/SKILL.md            "First message to a new match"
  screening/SKILL.md         "Is this person worth more time?"
  date-plan/SKILL.md         "Planning or deciding about a date"
  bio-review/SKILL.md        "Improving profile text"
  photo-lineup/SKILL.md      "Choosing and ordering photos"
  post-date/SKILL.md         "Debrief after meeting"
  the-call/SKILL.md          "When the right advice is to send nothing"
```

**The token arithmetic, which is the point:**

| | Index only | All bodies stuffed |
|---|---|---|
| 8 skills | **~320 tokens** | ~6,400 tokens |
| Total request (per `CONTEXT-STRATEGY.md` §8) | **~5,800** | **~11,400** |

Stuffing every playbook roughly doubles the request and pushes it into the range where Chroma measured real degradation. **Progressive disclosure is what makes a multi-job-type product fit inside a safe context budget at all** — which means the skills pattern isn't a nicety borrowed for elegance, it is load-bearing.

It also gives the typed-UI requirement a clean home: the skill defines the playbook *and* declares which UI the job renders in, so adding a job type is adding a directory rather than editing the agent.

## 3. Tools map directly

The eight primitives in `AGENT-HARNESS.md` §11 already follow the coding-agent pattern: a small, fixed, stable catalogue. Keeping it fixed per session is a caching requirement (§12 there), and Claude Code's split — *the model decides what to attempt, the tool system decides what's allowed* — is the guardrail architecture already adopted.

## 4. The coach handoff is a subagent with scoped memory

A mapping I did not expect, and it is exact.

Claude Code subagents can declare **which memory scopes to load** (`memory: user | project | local`), so they run with focused context rather than inheriting everything. The coach handoff is the same shape: a **fresh context, given a bounded scope, returning a result.**

`handoffs.context_snapshot` (`MATCH-THREADS-DESIGN.md` §5) should therefore carry exactly `you/MEMORY.md` + this match's `MATCH.md` + the triggering exchange — not the whole thread history, not other matches. Frozen at purchase, for the reason already given: the coach is paid to answer a specific situation and it must not move underneath them.

## 5. Compaction and permissions

**Compaction → distillation.** Claude Code auto-compacts near the window limit with a memory-flush pass first. Wing's version runs constantly rather than at a threshold, because the budget is ~5k not 200k. `CONTEXT-STRATEGY.md` §9 already specifies the order: distil `you/`, decay match state, summarise older turns.

**Permission modes → the stakes gate.** Claude Code gates capabilities by risk, requiring confirmation for destructive operations. Wing's equivalent is the drafting decay in `MATCH-THREADS-DESIGN.md` §3.3 — free at the opener stage, handoff-first at commitment. Same idea, calibrated to a different notion of destructive (§8).

---

# PART II — THE ASYMMETRY

## 6. Why the analogy holds structurally

Everything above transfers because both systems solve the same shape of problem: a long-running, stateful, multi-session task where context exceeds the window, work decomposes into recognisable job types, and durable human-readable state beats an opaque store.

## 7. There is no compiler

Here is where it stops, and it is not a detail.

**Claude Code works because it can verify its own output.** Write code, run the test, read the error, fix, repeat. That verification loop is what converts a plausible-text generator into something reliable — the harness literature lists *verification loops* as one of the five layers, and it is the layer doing the most work. Remove it and you have an agent that confidently produces plausible, wrong output with no way to notice.

Dating has no verification loop available:

| | Coding | Dating |
|---|---|---|
| Feedback | Immediate, deterministic | **Hours to days, or never** |
| Signal quality | Exact — it compiles or it doesn't | **Noisy and confounded** — she was busy, not put off |
| Absent feedback | Rare | **The most common outcome.** Ghosting is a null signal |
| Retries on the same input | Unlimited | **Zero.** You cannot re-send a different opener to the same person |
| Reversibility | `git revert` | **A sent message cannot be unsent** |

Five consequences follow, and they are the whole design:

1. **No self-correction within a session.** The agent cannot iterate toward correctness. It gets one attempt with no feedback, so it must be *right first time or appropriately uncertain* — which is a much harder posture than the coding-agent loop requires.
2. **Elicitation substitutes for verification.** Claude Code doesn't need much goal inference because the user states the goal and the tests check the result. Wing has neither, so the EVOI machinery in `AGENT-HARNESS.md` §5 isn't over-engineering — **it is the only pre-hoc substitute for a post-hoc check.** Asking is what you do when you can't test.
3. **`answer_outcomes` is a test suite with days of latency and terrible signal-to-noise.** It works statistically across many users and never for one interaction. Useful for ranking and for bet 1; useless for in-session correction.
4. **Conservative defaults are correct, not timid.** In coding, a wrong attempt costs a retry. Here it costs a match. That asymmetry justifies the stakes gate independently of any brand argument.
5. **The coach is the verification layer.** This is the deepest justification for the two-thread design yet found. In Claude Code, the compiler checks the agent. In Wing, **the human checks the agent** — and the handoff is not only the monetisation event, it is the missing verification loop being bought on demand, at the moments where being wrong is expensive.

That last point reframes the whole product. The handoff isn't an upsell bolted onto an AI thread. It is the architecture's answer to the one thing the coding-agent pattern cannot supply.

## 8. Irreversibility changes the permission calibration

Claude Code's permission model asks before destructive operations and proceeds freely otherwise, because most coding actions are cheap to undo.

**In Wing, every send is irreversible.** There is no revert. So the calibration inverts: the *default* path needs more care than a coding agent's default, and the gradient should follow stakes rather than a fixed allowlist. `MATCH-THREADS-DESIGN.md` §3.3 already encodes this — and note it survives the correction that removed the drafting prohibition, because it was never resting on the ToS argument that failed.

## 9. Different session shape

Coding sessions are long, dense, single-task, and hit the context limit. Dating threads are short bursts, spread over weeks, many running in parallel, never near the limit.

So: **less compaction pressure, far more cross-session state.** The memory profile favours small durable state over long working context — which is why `CONTEXT-STRATEGY.md` budgets 5k rather than managing 200k, and why the interesting engineering is in what persists, not in what fits.

---

# PART III — WHAT NOT TO IMPORT

| Claude Code has | Wing should not | Why |
|---|---|---|
| Long autonomous loops | — | Nothing verifies them. Autonomy without verification is confident error at scale |
| Subagent fan-out | — | Expensive, and no task here decomposes that way |
| Arbitrary execution | — | Not applicable |
| Auto-compact at ~98% of window | — | Budget is 5k, not 200k. Never get close |
| **A file-editing interface** | — | **The most important one — see below** |

**Users are not developers.** Claude Code's memory model works because its users hand-edit `CLAUDE.md`, read tool output, and know what a skill is. A dating-app user will do none of that.

So the *architecture* transfers; the *interface* cannot. `MATCH.md` is **generated and maintained by the agent, presented as a readable card, and edited through UI affordances** — a "that's not right" button, a fact chip you can delete — never a text editor. The file remains genuinely human-readable and exportable, which is the trust feature from `AI-LANDSCAPE.md` §22, but authorship stays with the system.

---

# PART IV — THE SPEC

## 10. Layout and budget

```
you/MEMORY.md                always loaded          ~1,000 tok
matches/<id>/MATCH.md        this match only          ~600 tok
skills/<name>/SKILL.md       index always, body on trigger
platform/RULES.md            always, user-unwritable  ~400 tok
```

| Component | Tokens | Cached |
|---|---|---|
| System prompt + tool definitions | 1,600 | ✓ |
| `platform/RULES.md` | 400 | ✓ |
| `you/MEMORY.md` | 1,000 | ✓ |
| Skills index (8 × ~40) | 320 | ✓ |
| `matches/<id>/MATCH.md` | 600 | |
| Active skill body | 800 | |
| Screenshot facts | 300 | |
| Recent turns | 800 | |
| **Total** | **~5,820** | **~57%** |

Comfortably below where degradation becomes measurable, with the majority cacheable at a 90% discount.

## 11. What `MATCH.md` holds

Deliberately thin — this is the file the privacy constraint in `AI-LANDSCAPE.md` §21 governs, and **accumulation depth is the legal variable.**

```markdown
# Priya · Hinge · added 12 Jul
## Goal          intent: unsure→dating (drifting) · investment: high
                 timeline: she's waiting · reciprocity: mutual, she initiates ~40%
## State         3 days since her last message; open question from her unanswered
## Tried         Jul 18 · specific-day ask → she said yes, then rescheduled
                 Jul 22 · advised waiting 24h → she messaged first ✓
## Not this      Don't reference her ex. She raised it once and moved on.
```

Note what's absent: no personality assessment, no appearance, no inferred psychology. **State and history, never portraiture.** A tool guardrail rejects writes matching third-party trait patterns (`AGENT-HARNESS.md` §13), so the constraint is enforced in code rather than in a style guide.

## 12. Coach-authored skills — interesting, and not yet

A tempting extension: let coaches author `SKILL.md` playbooks — *dating after divorce*, *queer openers*, *screening for women* — earning a share whenever their skill is invoked. It scales a coach's expertise past their own answering hours and is a supply contribution requiring no engineering.

**The tension is real and worth naming before anyone gets excited.** A coach who writes down their playbook is training their replacement. That is Cameo's disintermediation problem inverted: rather than the coach leaving with the customer, the platform keeps the expertise without the coach. Given `MARKET-ANALYSIS.md` §30 identifies the 10% coach-sourced take rate as the single learned lesson from the nearest comparable failure, introducing a mechanism that quietly devalues coaches is exactly the wrong move at the wrong time.

**Recommendation: not now.** It creates a supply-side incentive problem before supply exists. If revisited, the shape that survives the tension is skills covering *routine* patterns with attribution and a standing revenue share, while coaches keep the exceptions — the same stakes-tiering as §3.5 of the design doc.

## 13. Summary

**The mapping is real and four things transfer:** the concatenating memory hierarchy (already arrived at independently, from a privacy argument), skills with progressive disclosure, the fixed tool catalogue, and subagent-with-scoped-memory as the coach handoff.

**Skills are the biggest win.** Six-plus job types with their own playbooks and UI don't fit in a safe context budget stuffed; at ~40 tokens each indexed and loaded on trigger, they do — a ~2× difference on total request size. That makes progressive disclosure load-bearing rather than elegant.

**The asymmetry is the thing to design around.** Coding agents work because the compiler checks them. Dating offers no verification: feedback is delayed, noisy, frequently absent, unrepeatable, and irreversible. So elicitation substitutes for testing, conservative defaults are correct rather than timid, and — the reframe worth keeping — **the coach handoff is not an upsell attached to an AI thread. It is the verification loop, bought on demand, at the moments where being wrong is expensive.**
