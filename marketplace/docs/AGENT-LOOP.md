# Wing — Agent Layers and the Control Loop

**July 2026 · Turning the architecture sketch into a spine**

Takes the brainstormed shape — per-user memory, per-match memory, state-dependent skills, a harness that clarifies with multiple choice and produces replies/advice/roasts — and makes it buildable. The shape is right. **One structural correction changes everything downstream** (§1), and one research finding changes how detection has to work (§8).

---

# PART I — THE CORRECTION

## 1. Detection is not a skill

The sketch lists these together:

```
Skills:
  - openers of various kinds for various stages   ← produces output
  - ghosting recovery                              ← produces output
  - stage detection                                ← produces STATE
  - state detection                                ← produces STATE
  - intent of user / matched person detection      ← produces STATE
  - escalation-ready detection                     ← produces STATE
```

Four of those six are not skills, and treating them as skills creates a **bootstrapping circle**: skills load on trigger, but you'd have to load the stage-detection skill to learn the stage in order to know which skill to load.

**Perception must run always, before selection, and cannot be progressively disclosed.**

So the harness has three layers rather than one:

| Layer | Runs | Cost | Output |
|---|---|---|---|
| **1 · Perception** | **Always**, every turn | Cheap, fixed | Updated state — stage, signals, intent estimates, safety flags |
| **2 · Selection** | Always | ~Free — ordinary code | Which skill, which output mode |
| **3 · Skills** | On trigger only | Loaded body ~800 tok | The actual output |

This maps onto things already established. Perception is the sales-AI equivalent of *auto-populating the CRM from the conversation* (`SALES-PARALLEL.md` §5) — Gong reads its 300 signals on every call, not when asked. Selection is ordinary routing code, which is where product behaviour should live so it's testable. Skills are the progressive-disclosure layer that keeps the context budget viable (`AGENT-MODEL.md` §2).

**Consequence for cost:** perception runs on every turn, so it must be cheap and structured — a single call returning a fixed schema, not a reasoning pass. Most of it is arithmetic on screenshot metadata rather than model work at all (§8).

---

# PART II — MEMORY

## 2. Store and view are different things

The sketch lists "full conversation" under per-match memory. Correct as **storage**, fatal as **context**.

| | Store | View (per request) |
|---|---|---|
| Holds | Everything — full conversation, every screenshot's extracted facts, every piece of advice, every outcome | A budgeted projection, ~5k tokens |
| Grows | Unbounded, cheaply | **Never** |
| Cost | Disk | Tokens, and accuracy (`CONTEXT-STRATEGY.md` §5) |

Keep everything; **load a distillation**. Conflating the two is the single easiest way to reintroduce the context-rot failure this design exists to avoid. The view assembler is a real component with a hard budget asserted in CI.

## 3. The scopes, refined

```
you/MEMORY.md              always loaded            ~1,000 tok
matches/<id>/MATCH.md      this match only            ~600 tok
matches/<id>/store/        full history — stored, never loaded whole
platform/RULES.md          always, user-unwritable    ~400 tok
```

**Per-user (`you/`)** — the cross-match asset, and the thing that compounds:
- How this person writes — their real voice, sampled from messages they wrote themselves
- What they habitually do wrong — over-questioning, double-texting, disappearing when anxious
- What has actually worked for them, from outcome data
- Standing goals and dealbreakers

**Per-match (`MATCH.md`)** — five things, matching the sketch with one caveat:
- **Conversation state** — stage, open loops, who owes whom a reply
- **Signals** — the computed perception metrics and their trend
- **Their profile facts** — see the caveat below
- **User's goals for this match** — the staged schema (`AGENT-HARNESS.md` §1)
- **Advice given and what happened** — the outcome link

### The caveat on "insights from matched profile"

This is the portraiture line and it needs a usable test rather than a principle.

> **Would you be comfortable if she read this file?**

| Store | Never store |
|---|---|
| What she volunteered — training for a marathon, works nights, has a sister in Leeds | Inferred traits — "seems anxious", "likely avoidant attachment" |
| What she asked about | Assessments of her — attractiveness, desirability, ratings |
| Observable behaviour — replies in the evenings, initiates ~40% | Psychological profiling of any kind |

Facts she chose to share are hers to have shared. A psychological assessment of her, built without her knowledge, is the dossier the whole design has been avoiding (`AI-LANDSCAPE.md` §21). The test is concrete enough to enforce in a tool guardrail.

---

# PART III — STAGE, THE SPINE

## 4. The stage model

"State-dependent skills for various stages" is the right instinct, and it needs an actual enumeration, because stage drives skill routing, draft policy, and advice posture.

| Stage | Entered when | Default posture | Drafts? |
|---|---|---|---|
| **Pre-contact** | Matched, nothing sent | Give them something to open with | **Freely** — AI is strongest here |
| **Opening** | 1–3 messages, no rhythm | Establish reciprocity, don't over-invest | Freely |
| **Rapport** | Sustained back-and-forth | Depth over frequency | Yes, with the fork offered |
| **Escalation-ready** | Signals present for asking out | **Ask her out. Stop optimising messages** | Handoff to self-authoring |
| **Scheduled** | Date agreed | Don't over-plan; don't go cold | Sparingly |
| **Post-date** | Met | Read what happened, decide | Advice over drafts |
| **Stalled** | Was moving, momentum lost | Diagnose before acting | Yes — often The Call |
| **Ghosted** | No reply past a per-match threshold | Recovery or release | Yes, once, then stop |
| **Closed** | Over, either way | Extract the lesson into `you/` | None |

Two notes. **Stalled and ghosted are genuinely different** — stalled has a live thread and a diagnosable cause; ghosted needs a decision about whether to re-engage at all, and the honest answer is often no. The sketch was right to name ghosting recovery separately.

And **Escalation-ready is the most valuable detection in the system.** The commonest expensive failure in dating apps is a good conversation that never becomes a date. An agent that reliably says *"stop texting, ask her out, here's when"* is worth more than any number of better replies — and it is the one moment where the draft policy correctly steps back, because a self-written ask outperforms both AI and a professional (`MATCH-THREADS-DESIGN.md` §3.2).

---

# PART IV — PERCEPTION

## 5. What runs every turn

Fixed schema, cheap, before any skill loads.

```ts
type Perception = {
  safety:      SafetyFlag[]        // always first; can halt everything
  stage:       Stage               // §4
  signals:     Signals             // §6 — mostly arithmetic
  their_intent: Estimate<Interest> // inferred, always hedged
  open_loops:  OpenLoop[]          // unanswered questions, unmet proposals
  escalation:  Estimate<Ready>     // §4
  gaps:        Gap[]               // what's absent — SALES-PARALLEL.md §3
}
```

**Safety runs first and can halt the loop.** Self-harm, abuse, coercion, any indication of a minor → `escalate_safety`, no advice generated.

**Gaps are perception, not a skill.** *"Four weeks and she has never asked you a question"* is computed from stored history, costs no question budget, and is the strongest differentiator available (`AGENT-PRODUCT.md` §8.1). It belongs here because it must be noticed without being asked for.

## 6. The signals, and the finding that changes them

Mostly arithmetic on screenshot metadata — cheap, deterministic, testable. But the research corrects a naive reading:

| Signal | How | Caveat from research |
|---|---|---|
| **Initiation symmetry** | Ratio of who starts exchanges | **The most robust signal.** Studies find the strongest link to satisfaction is *similarity in initiation frequency* — symmetry, not volume |
| **Reply latency** | Time deltas | **⚑ Curvilinear, not monotonic.** A 2026 study finds a **U-shaped** effect: highest interest when texted the next morning, *lowest* after two days — and very fast replies can read as insecurity rather than interest ([Teichmann et al.](https://journals.sagepub.com/doi/10.1177/02654075251377184)) |
| **Length ratio** | Chars hers vs his | **Only meaningful against that person's own baseline** — some people are just verbose |
| **Question ratio** | Who asks | Drives the commonest gap flag |
| **Open loops** | Unanswered questions, unmet proposals | Highest-value single fact in most stalled threads |

**The curvilinear finding is a real trap avoided.** The obvious implementation — *faster reply = more interested* — is wrong at both ends, and would produce confidently bad advice in exactly the anxious cases where users need it most. Model latency as a U-shape with an optimum, not a slope.

**And baselining requires subject memory.** Length and latency only mean something relative to this match's own history — which is impossible for every competitor, since they discard the screenshot after answering (`CONTEXT-STRATEGY.md` §1.5). The signals worth computing are the ones only this architecture can compute.

## 7. Their intent is the highest-risk inference

The sketch bundles "intent of user / matched person detection" into one item. They are not the same problem:

- **User's intent** — askable. The staged schema, the fork pattern.
- **Their intent** — **only inferable**, from the signals above, about a third party.

That second one is the highest-value inference in the system (it decides lean-in versus pull-back) *and* the highest-risk (it is about someone who isn't there, from thin evidence, and stating it confidently is the failure that most damages trust). So it is permanently `Estimate<>`, never asserted, and it is the **primary target of the verification question** — *"does that match how it feels?"* — because the user holds ground truth the agent cannot see.

---

# PART V — SKILLS AND OUTPUT

## 8. The registry, routed by stage

Skills are the *doing* layer, selected by perception, loaded on trigger.

| Skill | Fires at stage | Output mode |
|---|---|---|
| `opener` | Pre-contact | Reply |
| `early-rhythm` | Opening | Reply + Read |
| `deepen` | Rapport | Read + Fork |
| `ask-her-out` | **Escalation-ready** | Verdict + timing |
| `date-plan` | Scheduled | Plan |
| `post-date` | Post-date | Read |
| `diagnose-stall` | Stalled | Read → often The Call |
| `ghost-recovery` | Ghosted | Verdict + one Reply |
| `screening` | Any — qualification, not discovery | Verdict |
| `roast` | Profile scope, user-invoked | Roast |
| `the-call` | Any — when holding is right | Verdict |

## 9. Output modes

The sketch's "replies, advice, roasting, etc." are genuinely distinct registers, and the skill declares which it produces — which also determines the renderer.

| Mode | Is | Renders as |
|---|---|---|
| **Reply** | Send-ready draft | Labelled block, copy action, "make it yours" |
| **Read** | What's actually happening | Prose, confidence-classed per claim |
| **Verdict** | A call — do this, don't, disengage | Single decisive statement |
| **Roast** | Adversarial critique of the **user's own** material | Blunt, itemised |
| **Fork** | Two branches + a choice | Both answers, two buttons (`AGENT-HARNESS.md` §8.1) |
| **Flag** | An unprompted gap | Quiet, dismissible |

**Roast deserves its own mode and a hard boundary.** It is a proven product — a company was built on it — and tonally it is the opposite of coaching. The boundary: **roast the user's own material, never a person.** Roasting his bio is the product. Roasting her photos is the thing that would end the brand, and it is the same portraiture line as §3.

---

# PART VI — THE LOOP

## 10. Putting it together

```
on turn:
    perception = perceive(view(store), new_screenshot)   # always, cheap, fixed schema
    if perception.safety:        → escalate, halt

    update(store, perception)                            # state persists

    skill = select(perception)                           # ordinary code
    load(skill)                                          # progressive disclosure

    branches = candidate_outputs(skill, goal_belief)
    if branches diverge:         → FORK  (the choice is the question)
    else:                        → skill.output_mode

    if perception.gaps:          → attach FLAG (unprompted, dismissible)
    if diagnosis drove output:   → attach VERIFICATION question

    record(advice, outcome_pending)
```

Four properties worth preserving. **Perception always runs**, so state stays fresh whether or not a skill fires. **Selection is code**, so routing is unit-testable rather than prompt-dependent. **Output never blocks on a question** — the fork carries both answers. And **every output is recorded against a pending outcome**, which is what makes `you/` compound.

## 11. On "generate skills wherever needed"

Runtime skill generation is the one part of the sketch to defer. It is unbounded, uncacheable, and un-evaluable — three properties that are individually manageable and jointly fatal for a product whose only verification is weak already (`AGENT-PRODUCT.md` §13).

**Keep the benefit without the risk: log the gap.** When perception lands in a state no skill covers, record it — the situation, the state, what was produced by fallback. That log is the authoring queue for the next skill, reviewed and evaluated before shipping. Coverage still grows; it grows through a gate.

Skills stay **authored, versioned, and evaluated**, which is also what makes the pairwise eval harness meaningful (`AGENT-HARNESS.md` §16).

## 12. Summary

**The sketch is right and needs one structural change:** detection is perception, not a skill. Three layers — perception always runs and cannot be progressively disclosed, selection is ordinary testable code, skills load on trigger and produce output.

**Store everything; load a distillation.** "Full conversation" belongs in the store and never in the view.

**Stage is the spine.** Nine stages drive skill routing, draft policy and posture — and *escalation-ready* is the most valuable detection in the system, because a good conversation that never becomes a date is the commonest expensive failure in the category.

**Two research corrections to perception.** Reply latency is **U-shaped, not a slope** — very fast reads as insecure, two days reads as disinterested — so the obvious implementation gives confidently wrong advice to anxious users. And length and latency only mean anything **against that match's own baseline**, which requires subject memory, which is precisely what no competitor has.

**Roast the user's material, never a person** — the same line as the memory rule, and the one that keeps this a coaching product rather than a dossier.
