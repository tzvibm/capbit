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

- the **marketplace** (pre-authored tools), or
- the **AI agent** (dynamically generated).

This is the justification for having both a marketplace and an agent.

### 1.4 Dynamic script adaptation

Scripts are **not static**. The agent adapts them to:

- context
- environment
- group dynamics
- culture
- user style

> Scripts provide structure. The agent provides context adaptation.

### 1.5 User sovereignty

No single "correct" behavior path exists. The user may always:

- pick a marketplace sequence
- request an AI-generated sequence
- edit either before committing

---

## 2. Architecture

### 2.1 Dual engines

**A. Marketplace — composable tools**

Marketplace "scripts" are internally modeled as **MCP-style composable tools**,
not passive content. Each tool:

- represents a mini-sequence
- has structured steps
- is callable by the agent
- is composable with other tools
- is forkable and transparent

**B. AI Agent — harness**

The agent is not merely a generator. Its harness must:

1. Interpret user context (from chat + explicit input).
2. Decide: use existing tools, compose tools, or generate from scratch.
3. Adapt output to context (environment, culture, user style).
4. Refine iteratively based on conversation.
5. Produce a **Final Compiled Sequence** for execution.

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

## 7. UX Contract

### 7.1 Planning screen

Must support:

- context input
- chat interface with the agent
- visible evolving plan
- clear **Final Sequence** section (always distinct from chat)
- ability to regenerate, edit, or swap sequences

No time pressure. Reflective, cognitive.

### 7.2 Execution screen

Must:

- show only the Final Compiled Sequence (3–5 steps)
- disable chat and all input
- remove all explanations and alternatives
- preserve action integrity above all else

### 7.3 The boundary

`3-2-1 GO` is the only path from Planning to Execution. There is no other
transition, and there is no path back mid-execution.

---

## 8. Invariants (must not regress)

The following are core and MUST remain intact through all future changes:

- Two-phase model (planning vs execution)
- Thinking allowed before, forbidden during
- 3-2-1 execution trigger as system primitive
- Sequence completion reward model (not outcome-based)
- Stacked mini-sequences (greeting / small talk / invitation or exit)
- Tool-based marketplace architecture (scripts = composable tools)
- Agent harness (interpret → decide → compose → adapt → compile)
- Compilation step producing a Final Compiled Sequence
- Abstraction boundary (no detail leaks into execution)
- User sovereignty across marketplace / AI / manual edits
- Outcome tracking for feedback only, never validation
- Composability
- Per-interaction sequence execution

---

## 9. What this system is not

- Not a chatbot
- Not a coaching app
- Not a content library
- Not an outcome-optimization engine

It is a **behavior execution system** with structured planning and enforced
action.
