# Social Interaction Tracker — Design

> Note: This document lives in the `capbit` repo for branch convenience only.
> The system described here is unrelated to capbit's permission model.

A behavior execution system with structured planning and enforced action.

This is **not** a chatbot, a coaching system, or a content library.

---

## 1. Philosophy

### 1.1 Two-phase cognition

All interaction with the system is split into two phases, with a hard boundary
between them:

| Phase | Mode | Thinking | Input | Purpose |
|---|---|---|---|---|
| 1 | **Planning** (Deliberation) | Encouraged, unlimited | Full | Expand understanding |
| 2 | **Execution** (Action) | Forbidden | None | Complete the sequence |

Core principle:

> **Think deeply before. Don't think at all during.**

The boundary between the two phases is a **core invariant** and must be
enforced in UX, data flow, and agent behavior.

### 1.2 Sequence completion reward model

The brain is rewarded for **completing the planned sequence**, not for the
outcome.

- Rejection is a valid success if the sequence was completed.
- The system MUST NOT surface outcome-based success metrics.
- Outcome tracking exists only for personal feedback / iteration — never for
  validation or scoring.

### 1.3 Script availability principle

> Where a script exists, hesitation collapses.

The user MUST always be able to obtain a usable sequence, via either:

- the **vault** (curated marketplace sequences served from the backend), or
- the **AI agent** (dynamically generated, optionally composed from vault
  content).

This is the justification for having both a curated vault and an agent.

### 1.4 Dynamic script adaptation

Scripts are **not static**. The agent adapts them to:

- context
- environment
- group dynamics
- culture
- user style

> Scripts provide structure. The agent provides context adaptation.

### 1.5 User sovereignty (within the app)

No single "correct" behavior path exists. Within the app, the user may
always:

- pick a vault (marketplace) sequence
- request an AI-generated sequence
- edit either before committing
- swap or regenerate at any point in Planning Mode

> Sovereignty here is *behavioral* (the user controls what they execute),
> not *infrastructural* (the user does not own the raw vault files —
> see §2.1 on the closed-source moat).

---

## 2. Architecture

### 2.1 Dual engines

**A. Marketplace — the Vault (closed-source moat)**

The marketplace is a **proprietary, closed-source vault** of curated
sequences. It is the system's primary moat. Sequences are:

- authored and curated centrally (or by vetted contributors)
- served to clients via the company's backend API
- **not** distributed via git, public repos, or any open protocol
- composable through internal references (see §2.4)

The internal storage format is markdown-with-frontmatter (see §2.4) for
agent ergonomics, but the **distribution, hosting, and personalization data
are closed**. Users interact with the vault through the app, not through
files. The format is an implementation detail, not a contract.

This is a deliberate v1 decision: lock-in through curated quality,
personalization data, and accumulated user history is the business model.
Personal vault content is also tied to the user's account on the backend.

**B. AI Agent — harness**

The agent is not merely a generator. Its harness must:

1. Interpret user context (from chat + explicit input).
2. Decide: search the vault, compose existing sequences, or generate fresh.
3. Adapt output to context (environment, culture, user style).
4. Refine iteratively based on conversation.
5. Produce a **Final Compiled Sequence** for execution (see §4).

The agent reads and writes the vault through a small, fixed set of internal
tools (see §2.5). It does **not** speak any open protocol (no MCP, no public
plugin surface).

### 2.2 Stacked mini-sequences

Each interaction is a series of complete loops. The base loop is:

1. Greeting
2. Small talk
3. Invitation **or** Exit

If the interaction continues, a new loop begins. Sequences stack.

### 2.3 The 3-2-1 trigger

`3 → 2 → 1 → GO` is not a UI flourish. It is the **system primitive** that
marks the boundary between Planning Mode and Execution Mode. Crossing it:

- locks the compiled sequence
- disables chat and editing
- enters Execution Mode

### 2.4 Vault format (internal implementation)

> This section describes how sequences are represented *inside* the closed
> vault. It is not a public spec. The format may change without notice.

Each sequence is stored as a markdown document with YAML frontmatter. The
markdown form is chosen because it is cheap to read for the agent, easy to
diff internally, and trivially composable via wikilink-style references.

```
vault/
├── sequences/                  ← curated marketplace content
│   ├── greeting/
│   │   ├── casual-approach.md
│   │   └── direct-intro.md
│   ├── small-talk/
│   │   └── weather-bridge.md
│   └── invitation/
│       └── coffee-ask.md
├── memory/                     ← per-user long-term memory (account-bound)
│   ├── style.md
│   ├── outcomes/
│   │   └── 2026-04-12.md
│   └── reflections.md
└── compiled/                   ← archived Final Compiled Sequences
    └── 2026-04-12-coffee-shop.md
```

A sequence file:

```markdown
---
id: casual-approach
type: greeting
tags: [casual, low-pressure, stranger]
pairs_with: [[small-talk/weather-bridge]], [[invitation/coffee-ask]]
visibility: marketplace        # marketplace | personal
version: 3
---

# Casual Approach

A low-stakes greeting for relaxed contexts.

## Abstract steps
1. Walk over casually
2. "Hey — quick hi"
3. Light comment
4. Invite or exit

## Planning notes (for the agent)
Use when context is relaxed, no time pressure.
Avoid when the target looks busy or mid-task.
```

Wikilinks (`[[other-sequence]]`) express composition between sequences. The
agent resolves them when assembling stacked mini-sequences.

### 2.5 Vault tools (internal)

The agent operates on the vault through a small, fixed toolset. These are
**internal Python functions** wrapped as harness tools — not a public
protocol.

Vault tools (unstructured, content):

- `search_vault(query)` — find relevant sequences in marketplace + personal
- `read_sequence(path)` — load a sequence file
- `list_folder(path)` — browse vault folders
- `read_memory(topic)` — pull prior personal notes / outcome logs
- `write_note(path, content)` — append to personal memory
- `archive_compiled(sequence)` — save the Final Compiled Sequence to
  `compiled/` after execution (feeds personalization)

Structured tool (schema-enforced, exits planning):

- `emit_final_sequence(steps: list[str])` — see §4. Validated against
  length and detail constraints. The **only** way to leave Planning Mode.

### 2.6 Memory access — search-based, not whole-vault

The agent must not load the full vault per session. Instead, it uses
`search_vault` and `read_memory` to pull only what's relevant. This keeps
context windows small, costs predictable, and the architecture viable as
the vault grows.

---

## 3. Planning Mode

### 3.1 Conversational planning layer

Planning Mode includes a chat interface with the agent. Users may freely ask
detailed questions about:

- approach mechanics (e.g. "how do I walk up?")
- body language
- positioning
- tone
- edge cases
- social dynamics

The agent must answer **in depth**, refine the evolving plan based on the
conversation, and adapt sequences dynamically as context changes.

### 3.2 Allowed outputs in Planning Mode

During planning, the agent MAY:

- show multiple options
- explain tradeoffs
- give nuanced, detailed guidance
- regenerate / iterate

### 3.3 The evolving plan

The planning screen maintains a visible, evolving plan that updates as the
chat progresses. It is distinct from — and always accompanied by — a dedicated
**Final Sequence** section.

---

## 4. Compilation Step (Mandatory)

Before execution, the system MUST produce a **Final Compiled Sequence**.

### 4.1 Cognitive compression

> Expand understanding during planning.
> Compress behavior before execution.

The agent's job at compile time:

1. Gather context via chat.
2. Answer user questions in detail.
3. Build a full internal plan.
4. **Compress** it into a minimal executable sequence.

### 4.2 Final Compiled Sequence — rules

- **3–5 steps only**
- each step is high-level and human-executable
- no technical detail
- no body language specifics
- no explanations
- no alternatives
- no branching

Example:

```
1. Walk over casually
2. "Hey — quick hi"
3. Light comment
4. Invite or exit
```

### 4.3 Abstraction boundary (critical)

Even if the user asked during planning about posture, eye contact, angles, or
tone nuance, those details MUST remain in Planning Mode only. **Detail must
not leak into execution steps.**

This is a hard constraint on the agent and on the UI.

---

## 5. Execution Mode

Triggered by `3 → 2 → 1 → GO`.

### 5.1 Hard constraints

- No thinking.
- No AI interaction.
- No editing.
- No new input.
- No alternatives.
- No explanations.
- No branching.

### 5.2 What the screen shows

Only the Final Compiled Sequence — 3–5 abstract steps, nothing else. Chat is
disabled. Input is removed. Explanations are stripped.

### 5.3 Completion

On completion, the user may optionally log outcome data for personal feedback.
This is not a score.

---

## 6. Chat → Sequence Pipeline

```
1. User enters Planning Mode
2. User chats with agent (optional but encouraged)
3. Agent:
     - answers questions in depth
     - refines evolving plan
     - may invoke marketplace tools or generate fresh sequences
     - adapts to context
4. Agent produces the Final Compiled Sequence
5. User taps 3-2-1 GO
6. System enters Execution Mode (locked)
7. User executes
8. (Optional) Outcome logged for personal feedback
```

---

## 7. Agent Harness

### 7.0 Stack

| Layer | Choice | Notes |
|---|---|---|
| **Model (primary)** | **Gemma 4 31B Dense** (hosted) | Apache 2.0; native function calling as a first-class training objective; ranks #3 open on LMArena. Function-call reliability is high enough that the schema-enforced compile step is sufficient — no post-hoc regex classifier needed in v1. |
| **Model (on-device, future)** | **Gemma 4 E4B** | Runs on phone. Reserved for a future "private mode" deployment branch (see §7.3). |
| **Harness** | **Pydantic AI** | Schema-first. The Final Compiled Sequence is a Pydantic model with hard validators; the harness retries automatically on validation failure. |
| **Provider router** | **LiteLLM** | Lets us swap between Google AI Studio / Groq / Together / Ollama / on-device with a config string. |
| **Local dev** | **Ollama** running `gemma4` | Free, OpenAI-compatible. |

### 7.1 Compile-step schema

The compile step is the one place where structure protects the abstraction
boundary. It MUST be a schema-validated tool call, not free-text generation.

Pseudocode:

```python
class FinalCompiledSequence(BaseModel):
    steps: list[StepText]              # 3 <= len(steps) <= 5

class StepText(ConstrainedStr):
    max_length = 80
    forbidden_substrings = [
        "eye contact", "posture", "tone of voice",
        "body language", "angle", "lean in",
        # ...detail keywords that must stay in planning
    ]
```

If the agent emits a sequence that fails validation, Pydantic AI retries
with the validation error in context. The agent cannot exit Planning Mode
without producing a valid `FinalCompiledSequence`.

### 7.2 Why this stack

- **Cost.** Gemma 4 31B at hosted rates is roughly 5× cheaper than Claude
  Haiku 4.5 per planning session (~$0.0006 vs ~$0.003 for typical 3k-in /
  500-out sessions). Apache 2.0 means no commercial licensing concerns.
- **Tool-call reliability.** Function calling is a first-class training
  objective in Gemma 4, not a prompt-engineering workaround. This is what
  makes the schema-enforced compile step viable on an open-weights model.
- **Provider-agnostic.** LiteLLM means we are not locked to any single
  hosting provider. If Groq becomes cheaper than Google AI Studio next
  quarter, we change one line.
- **No protocol exposure.** No MCP, no plugin surface, nothing that would
  let third parties script against the vault. The closed-source moat is
  preserved.

### 7.3 Deployment branches

**Default (v1): server-side.**

- Vault lives on the backend (closed source moat).
- Agent runs server-side (Pydantic AI + LiteLLM + Gemma 4 hosted).
- Client is a thin app that renders chat, the evolving plan, the Final
  Compiled Sequence, and the Execution screen.

**Future branch: on-device "private mode."**

- Gemma 4 E4B runs locally on the user's phone.
- Personal vault content syncs to the device (encrypted at rest).
- Marketplace sequences are still served from the backend but cached
  per-session — never bulk-exported.
- Sold as a privacy upgrade, not the default. Defaulting on-device would
  ship the full vault to clients, which erodes the moat.

### 7.4 Verification TODOs

These are not assumed-true; they need direct confirmation before code:

- [ ] Confirm Groq's exact pricing for Gemma 4 31B Dense (catalog lists it,
      $/Mtok not yet confirmed). Compare to Google AI Studio + Together.
- [ ] Confirm Pydantic AI / LiteLLM integration path for Gemma 4's custom
      tool-call special tokens. The official function calling docs are at
      `ai.google.dev/gemma/docs/capabilities/text/function-calling-gemma4`.
- [ ] Decide on personal-data export policy (impacts moat strength vs
      regulatory compliance in EU).

---

## 8. UX Contract

### 8.1 Planning screen

Must support:

- context input
- chat interface with the agent
- visible evolving plan
- clear **Final Sequence** section (always distinct from chat)
- ability to regenerate, edit, or swap sequences

No time pressure. Reflective, cognitive.

### 8.2 Execution screen

Must:

- show only the Final Compiled Sequence (3–5 steps)
- disable chat and all input
- remove all explanations and alternatives
- preserve action integrity above all else

### 8.3 The boundary

`3-2-1 GO` is the only path from Planning to Execution. There is no other
transition, and there is no path back mid-execution.

---

## 9. Invariants (must not regress)

The following are core and MUST remain intact through all future changes:

- Two-phase model (planning vs execution)
- Thinking allowed before, forbidden during
- 3-2-1 execution trigger as system primitive
- Sequence completion reward model (not outcome-based)
- Stacked mini-sequences (greeting / small talk / invitation or exit)
- **Closed-source vault as the marketplace moat** (no public protocol, no
  git distribution, no MCP, no third-party plugin surface)
- Vault-as-memory (sequences, personal notes, outcomes, and compiled
  archives all live in the same vault abstraction)
- Search-based memory access (never load the full vault per session)
- Agent harness (interpret → decide → compose → adapt → compile)
- Compilation step producing a Final Compiled Sequence via a
  **schema-validated tool call** (not free-text generation)
- Abstraction boundary (no detail leaks into execution)
- User sovereignty *within the app* across marketplace / AI / manual edits
- Outcome tracking for feedback only, never validation
- Composability (via internal wikilink references)
- Per-interaction sequence execution

> Note: "User sovereignty" in v1 means *within the app*, not raw filesystem
> ownership. The closed vault is a deliberate trade-off — see §2.1.

---

## 10. What this system is not

- Not a chatbot
- Not a coaching app
- Not a content library
- Not an outcome-optimization engine

It is a **behavior execution system** with structured planning and enforced
action.
