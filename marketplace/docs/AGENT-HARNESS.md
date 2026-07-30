# Wing — The Agent Harness

**July 2026 · Engineering spec for the match-thread agent**

Companion to `MATCH-THREADS-DESIGN.md` (what the product is) and `AI-LANDSCAPE.md` §22 (memory typing). This document is **how the agent is actually built**: goal inference, question selection, tool contracts, guardrails, cost, and evals.

The framing observation is correct and is the right place to start: **dating advice is almost entirely goal-conditional.** "She replied after three days" has opposite correct responses depending on whether the user wants a hookup this weekend or a relationship by spring. An agent that doesn't know the goal isn't giving bad advice — it's giving advice to an average of people, which is worse than useless because it sounds confident.

So the agent's first job is not advice. It is **figuring out what this user wants from this specific person, at the lowest possible question cost.**

---

# PART I — THE GOAL PROBLEM

## 1. Goal is a vector, not a value

One "what are you looking for?" question is the standard approach and it is close to worthless, because it collapses six independent dimensions into one label. The dimensions that actually change advice:

| Dimension | Values | Why it changes the answer |
|---|---|---|
| **Intent** | casual · open · dating · serious · unsure · friends | Determines pace, what to disclose, whether to escalate to a meet |
| **Reciprocity read** | mutual · you're keener · they're keener · unclear | **Highest-leverage variable.** "Lean in" vs "pull back" is entirely determined by this |
| **Timeline** | now · days · no deadline | Whether waiting is a strategy or a mistake |
| **Investment appetite** | low · medium · high | How much effort this match is worth spending |
| **Constraint** | distance · travel · exclusivity · time · none | Rules out otherwise-correct advice |
| **Self-presentation** | funny · warm · direct · low-key | Only matters for drafting — so it belongs to the **coach**, not the agent |

That is ~216 meaningful combinations before constraints. Most are irrelevant to any given question, which is the fact the whole design exploits.

**A seventh variable lives elsewhere.** *What this user is habitually bad at* — over-questioning, double-texting, disappearing when anxious — is a property of the person, not the match. It belongs in the `you/` memory scope (`AI-LANDSCAPE.md` §22), persists across every match, and is the thing that compounds into a real asset.

## 2. Goal is per match, and it is unstable

Three properties that make this harder than an onboarding questionnaire:

1. **It varies per match.** The same user wants different things from different people. A global onboarding answer is a weak prior at best. This is the strongest argument for per-match threads existing at all.
2. **It drifts.** "Open to anything" becomes "actually I like this one" in a week. The belief state must decay and be re-elicited, not captured once.
3. **Stated goals are unreliable.** Social desirability runs both directions — people overstate seriousness to seem substantial and understate it to seem relaxed. And frequently the honest answer is *"I don't know yet,"* which must be a **first-class value, not a failure state.** For a new match it is often the correct answer.

The design consequence: never confront a stated goal, and never treat `unsure` as something to resolve. Treat it as information — `unsure` plus high investment is a recognisable and common state with its own correct advice.

## 3. The platform is a free, strong prior

Before asking anything, one already-collected field carries real information. `matches.platform` is set at match creation, one tap.

| Platform | Serious intent | Source |
|---|---|---|
| **Hinge** | **~87–90%** | [DoULike](https://www.doulike.com/blog/statistics/hinge-statistics/), [CupidAI](https://getcupid.ai/blog/editorial/hinge-statistics) |
| Match.com | ~90% | [GRASS](https://grass.camp/en-US/blog/best-dating-apps-serious-relationships) |
| eharmony | ~89% | [GRASS](https://grass.camp/en-US/blog/best-dating-apps-serious-relationships) |
| **Tinder** | **~50%** serious, 32% open, 18% casual | [catfishfinder](https://catfishfinder.org/dating-app-statistics/) |
| Grindr | ~66% | [GRASS](https://grass.camp/en-US/blog/best-dating-apps-serious-relationships) |
| Happn | ~58% | [GRASS](https://grass.camp/en-US/blog/best-dating-apps-serious-relationships) |

**On Hinge the intent prior is so concentrated that asking about intent is usually a wasted question.** On Tinder it is nearly uniform and the question is worth asking. That single distinction — ask on Tinder, infer on Hinge — is free accuracy from a field you already have, and it reinforces the Hinge-first positioning from `MARKET-ANALYSIS.md` §4.

## 4. What to infer versus what to ask

Elicitation budget is scarce, so spend it only on what cannot be observed.

| Dimension | Inferable from a screenshot? | How |
|---|---|---|
| **Reciprocity read** | **Yes, strongly** | Reply latency ratio, message length ratio, who initiates, who asks questions, emoji reciprocity |
| **Timeline** | **Yes, strongly** | An unanswered question, a proposed date, a visible gap |
| **Constraint** | Partly | Explicit mentions ("I'm away till Thursday") |
| **Intent** | **No** — prior only | Platform prior + must ask when it matters |
| **Investment appetite** | **No** | Must ask, or infer weakly from how much the user is agonising |

So: **infer reciprocity and timeline; spend questions on intent and investment.** And note the pleasing consequence — the highest-leverage variable (reciprocity) is the most inferable, so the agent's best signal is free.

---

# PART II — QUESTION SELECTION

## 5. The algorithm: expected value of information, not expected information gain

The literature gives the mechanism. **BED-LLM** uses expected information gain (EIG) to select questions, grounded in Bayesian optimal experimental design, with "substantial gains… when actively inferring user preferences" ([arXiv](https://www.researchgate.net/publication/386188872_Learning_to_Ask_Informative_Questions_Enhancing_LLMs_with_Preference_Optimization_and_Expected_Information_Gain)). Google's *Asking Clarifying Questions for Preference Elicitation with LLMs* trains sequential questioners ([arXiv 2510.12015](https://arxiv.org/abs/2510.12015)). **TO-GATE** improves on STaR-GATE, which was "limited by ineffective question **sequences**" ([arXiv 2506.02827](https://arxiv.org/html/2506.02827)) — so sequencing, not individual question quality, is the documented failure mode.

But pure EIG is the wrong objective here, and this is the most important design decision in the document.

> **EIG asks "which question most reduces my uncertainty about the goal?" The right question is "which uncertainty, if resolved, would change my advice?"**

That is expected *value* of information — EVOI, "a Bayesian technique that computes expected gain in user utility" and is established as effective for selecting elicitation queries. The practical difference is large:

- **EIG stopping rule:** keep asking until the goal posterior is confident. Produces an interrogation.
- **EVOI stopping rule:** stop as soon as the advice is the same across the remaining posterior mass. Produces a conversation.

Worked example. She asked a question three days ago; he hasn't answered. Under any intent — casual, serious, unsure — the advice is *answer her question*. **EIG would still ask about intent, because intent is uncertain. EVOI asks nothing and answers immediately**, because intent doesn't change the output. Conversely, if the situation is "she's warm but slow and he's wondering whether to ask her out," casual vs serious genuinely diverges, and there the question earns its friction.

## 6. The stopping rule, stated concretely

```
belief  := prior(platform) × decay(previous_belief) × evidence(screenshot facts)
loop:
    branches := candidate_advice(belief)          # advice per high-mass goal state
    if agree(branches):        answer now         # EVOI ≈ 0
    q := argmax_q  EVOI(q | belief, branches)
    if EVOI(q) < τ_friction:   answer now, hedged
    if questions_asked >= 2:   answer now, hedged # hard budget, see §7
    ask(q); belief := update(belief, answer)
```

Two clauses are doing the work. `if agree(branches)` is what makes it feel like talking to someone competent rather than filling a form. And `τ_friction` is a real, calibratable number — the point at which one more question costs more in abandonment than it buys in advice quality.

## 7. The question budget is two

The elicitation research is clear that questions are expensive: "every additional question costs you completion rate," keep to under ten total, and **at most one open-ended question** before fatigue drops completion ([Koji](https://www.koji.so/blog/customer-onboarding-survey-questions), [Lindy](https://www.lindy.ai/blog/customer-onboarding-survey)).

Those benchmarks are for surveys people agreed to take. A user who arrived at a match thread with an urgent, embarrassing problem has far less patience.

**Hard rule: at most two questions before the first substantive output.** After that the agent has to earn further questions by having been useful. Every question after the second must clear a higher `τ_friction`.

## 8. Multiple choice is right, and it must be a tool rather than prose

Structured choice "reduce[s] ambiguity and spot[s] patterns quickly." It is also the correct mobile modality — one tap, no typing, on a phone, about something embarrassing. And the harness literature explicitly names **"structured user elicitation"** as one of the compact set of high-leverage primitives a production agent should have.

```ts
ask_choice({
  question: string,              // ≤ 12 words, one clause
  options: Array<{
    label: string,               // ≤ 5 words, contextual — never generic
    evidence: Partial<GoalBelief> // what choosing this implies
  }>,                            // 2–4 options
  allow_skip: true               // always
})
```

Four rules:

- **Options are generated, not enumerated.** After a screenshot showing a three-day gap, the options are about *that gap*, not a generic intent menu. Generic options are what makes chatbots feel like phone trees.
- **2–4 options.** Five is a form.
- **Always skippable.** Forcing a choice the user can't honestly make injects noise into the belief state — worse than no answer, because the model then trusts it.
- **Each option carries its evidence payload.** The answer updates the belief directly; it does not go back through prose for re-interpretation. This is what makes the belief state debuggable.

## 9. The goal space is small — this does not need a research team

Worth stating plainly, because "Bayesian belief state over goal vectors with EVOI question selection" sounds like a six-month project. It isn't.

The space is **discrete and tiny**: 6 intents × 4 reciprocity × 3 timelines × 3 investments = 216 states, and any real situation has meaningful mass on maybe a dozen. A hand-specified conditional model over a few hundred discrete states, with the LLM supplying evidence updates and generating candidate questions, is entirely tractable, inspectable, and unit-testable. No embeddings, no training, no gradient anything — which also keeps it inside the `ARCHITECTURE.md` §2.1 prohibition on adding an embeddings dependency.

The LLM does three narrow jobs: extract facts from a screenshot, propose candidate questions with evidence payloads, and write prose. **The belief state and the stopping rule are ordinary code**, which is exactly where you want the logic that determines product behaviour.

---

# PART III — THE HARNESS

## 10. Five layers

Production harnesses converge on the same five: **tool orchestration, verification loops, context and memory, guardrails, observability**.

| Layer | Wing's implementation |
|---|---|
| **Tool orchestration** | 7 fixed tools (§11). Fixed catalogue per session is a caching requirement, not a preference |
| **Verification** | Output guardrail on every turn: does this contain send-ready text? (§13) |
| **Context & memory** | Typed scopes from `AI-LANDSCAPE.md` §22 — `you/` grows, `match/` capped and decaying |
| **Guardrails** | Three levels — input, tool, output (§13) |
| **Observability** | Every turn traces belief-before, tool calls, belief-after, stopping reason |

That last column entry matters more than it looks. **Logging the stopping reason** — *branches agreed* / *EVOI below threshold* / *budget exhausted* — is what makes the agent debuggable. Without it, "why didn't it ask?" is unanswerable.

## 11. Tool catalogue

Keep it small and stable. The harness guidance is a compact set of high-leverage primitives; the caching guidance is that a **fixed tool catalog per session** is part of what keeps the prefix stable.

| Tool | Contract | Notes |
|---|---|---|
| `read_screenshot(id)` | → `{turns[], latencies[], length_ratio, initiator, open_question}` | **Structured facts only.** Never "describe her." No appearance, no inferred traits |
| `ask_choice(...)` | → chosen option's evidence payload | §8 |
| `update_belief(dim, dist)` | → new posterior | Pure function, unit-tested |
| `note_pattern(id, evidence)` | writes to `you/` | The compounding asset. "Third time you've double-texted after a gap" |
| `write_memory(scope, k, v)` | scope-checked write | Tool guardrail rejects third-party trait content (§13) |
| `offer_handoff(job_type, reason)` | → renders the purchase card | The revenue moment. `reason` is shown to the user |
| `escalate_safety(category)` | → freezes thread, admin ticket | Hard stop. Not advice |
| `draft_message(stage, options_n)` | → 1–3 labelled drafts | **Stakes-gated (§13). Always labelled AI-authored** |

Eight primitives. Still no `send_message` and nothing that touches a dating app — the agent hands text to the *user*, who decides.

## 11.5 Context budget

Full reasoning and the competitor teardown are in `CONTEXT-STRATEGY.md`. The operative rules:

| Component | Budget | Cached |
|---|---|---|
| System prompt + tools | ~2,000 | yes |
| `you/` distilled memory | ~1,000 | yes |
| This match's state | 800 | no |
| Screenshot facts | 300 | no |
| Recent turns | 800 | no |
| **Total** | **~4,900** | ~60% |

**One match per context, ever** — enforced in the query layer, not by prompt instruction. Chroma's context-rot study across 18 frontier models found that **semantic similarity drives decay more than length does**, and a dating corpus is unusually self-similar: every fragment is a plan to meet, a pet, a job, a scheduling message. Two matches in one window produces confident cross-attribution — Jamie's dog reported as Priya's — which is silent and fatal to the product's core claim that someone was paying attention.

**Distil, never truncate.** The middle of the window is already where lost-in-the-middle costs 20–30 points; cutting it is the worst available option. Assert the budget in CI, or it becomes a suggestion.

## 12. State transitions as messages, not prompt rewrites

A specific and expensive mistake to avoid. The caching guidance: *"stable prompt prefix, append-only history, fixed tool catalog per session, and state transitions modeled as messages or mode flags rather than prompt rewrites."*

The tempting implementation is to re-render the system prompt each turn with the current belief state interpolated in. **That busts the cache on every turn and multiplies cost by roughly 5–10×.** Instead: the belief state arrives as an appended message or a short mode flag, and the prefix — system prompt, tool definitions, `you/` memory — stays byte-identical.

## 13. Guardrails, three levels

OpenAI's SDK structures these as input, output and tool guardrails. Anthropic's architectural separation is the principle to copy: **the model decides what to attempt; the tool system decides what's allowed** — Claude Code gates ~40 capabilities independently.

**Input guardrails** — run before the agent sees the turn:
- Safety classification: self-harm, abuse, coercive control, any indication of a minor → `escalate_safety`, agent never reasons about it
- Attachment scan: reject non-screenshot content

**Tool guardrails** — run on every invocation:
- `write_memory` with `scope='match'` rejects any value matching third-party trait patterns (appearance, personality, inferred psychology). This is `AI-LANDSCAPE.md` §21 enforced in code, not in a policy document
- `write_memory` enforces the per-match token cap
- `offer_handoff` rate-limited — an agent that offers a purchase every turn is a salesman

**Output guardrails — three, and none of them is a ban on drafting:**

An earlier version of this spec required *no send-ready text at all*. That was wrong (`MATCH-THREADS-DESIGN.md` §3): it rested on a ToS reading that doesn't hold and on a stated preference that `MARKET-ANALYSIS.md` §7 had already discounted, and it withheld the one thing AI measurably does better than the average person. What replaces it:

| Guardrail | Rule | Gate |
|---|---|---|
| **Provenance** | Every send-ready block carries its author. AI drafts render in the AI register and say so | **100%, CI** |
| **No coach impersonation** | The agent never claims to be a person, never uses a coach's name, never implies human authorship | **100%, CI** |
| **Stakes gate** | `draft_message` is refused at `stage='commitment'` unless the user has explicitly asked after being offered the handoff | 100%, CI |

The first two are the absolute ones, and they are cheap and deterministic — a rendering property plus a string check, not a model call. They preserve the only thing the old prohibition was protecting: that when Wing says *a person read this and will tell you why*, the claim is true and the user can verify which is which.

The stakes gate is soft by design. It **defaults** to offering a coach at the commitment stage — because that is where a genuine self-written message beat both AI and a professional — but a user who asks again gets a draft. Refusing a paying adult twice is paternalism, and it is not what the data supports; the data supports a *default*, not a lock.

## 14. Safety and goal elicitation are the same surface

A consequence worth designing for rather than discovering.

The mechanism that asks "what do you want here?" is also the mechanism that finds out when the answer is something you won't serve — persistence after refusal, deception about intent, pressure, or anything involving a minor.

**Therefore the intent enum must contain the disallowed values explicitly**, rather than leaving them as an unhandled else-branch. A classifier that can only choose between `casual` and `serious` will map "get her to change her mind about saying no" onto `casual` and coach it. One that has `coercive` as a reachable value routes it to `escalate_safety` instead.

This is a small schema decision with a large safety consequence, and it is the kind of thing that is nearly impossible to retrofit.

---

# PART IV — COST AND EVALS

## 15. Caching is the difference between viable and not

`MATCH-THREADS-DESIGN.md` §7 and §11.2 established that inference cost per user determines whether the AI tier works. The caching numbers are decisive: **Anthropic discounts cached input reads by 90%** (1.25× on write, 0.10× on read); OpenAI gives 50% above a 1,024-token stable prefix. Real-world: **75.6% total savings at an 84% hit rate**, and a documented agent going from **$720/month to $72/month by adding three `cache_control` markers**.

Modelled on Wing's shape — 6 turns per session, 8 sessions per month, ~300 output tokens per turn, mid-tier pricing:

| Prefix size (`you/` memory) | No caching | With caching | Saving |
|---|---|---|---|
| 4k tokens (new user) | ~$0.35/mo | ~$0.21/mo | 40% |
| 15k tokens (established) | ~$0.88/mo | ~$0.36/mo | **59%** |

**Caching's value scales with memory size**, which is precisely the direction this product moves. At $8/month subscription and $0.36 inference, gross margin is ~95%. Even a 5× power user lands near $1.80 — comfortable.

Which sharpens the §11.2 conclusion: **per-user cost was never the problem. The problem was volume of non-converting free users.** A capped trial plus caching is sufficient; nothing more exotic is needed.

## 16. Evals

The literature is unambiguous on method: **pairwise comparison beats absolute scoring for subjective quality**, with the gap widening the more subjective the dimension; LLM judges agree with human reviewers ~85% of the time, higher than two humans agree with each other; use **both orderings**, calibrate against human labels, and **never use the same model family as generator and judge**.

**CI gates — rubric-based, deterministic, block the build:**

| Gate | Threshold |
|---|---|
| No send-ready text in output | **100%** |
| Safety classification recall on a labelled set | ≥99% |
| `write_memory` schema and scope compliance | 100% |
| Questions asked before first output ≤ 2 | 100% |

**Quality evals — pairwise, tracked as trends:**

| Metric | Measures |
|---|---|
| **Advice quality vs baseline** | Pairwise, both orderings, cross-family judge |
| **Goal-inference accuracy** | Hold out an explicit ask; compare to the posterior the agent had |
| **Question efficiency** | Questions asked to reach a stable decision |
| **Handoff precision** | **See below — the important one** |

**Handoff precision is the health metric for the entire two-thread design.** When the agent said "this needs a person," did the coach's advice actually differ from what the agent would have said?

- Coach almost always agrees → the agent should have answered. Escalating was a wasted purchase and erodes trust.
- Coach usually differs → the escalation was correct, and the marketplace is earning its take.

That is directly measurable, it requires no extra instrumentation beyond logging the agent's suppressed candidate answer, and it is the operational version of the narrow band in `MATCH-THREADS-DESIGN.md` §2.

## 17. This is also the correct methodology for bet 1

`MARKET-ANALYSIS.md` §35's existential bet — does human judgment beat frontier AI on real threads — should be run using the method above rather than an informal comparison:

1. 30 real threads, coach answer and model answer for each
2. **Pairwise, both orderings**, to control position bias
3. Judge from a **different model family** than the generator
4. Calibrated against human labels on a subset
5. Primary outcome remains **recipient reply rate**; the judge is the cheap proxy that lets you scale past 30

The methodological upgrade matters: an absolute 1–5 rubric on "advice quality" would likely show no difference, because absolute scoring is weakest exactly where the dimension is subjective. Pairwise is what surfaces a real gap if one exists.

---

# PART V — WHAT'S ACTUALLY INVOLVED

## 18. Build estimate, one job type

| Component | Effort | Notes |
|---|---|---|
| Screenshot fact extraction | ~1 week | Vision model, structured output, strictly no trait inference |
| Belief state + EVOI selection | **2–3 weeks** | The novel part, and it's mostly ordinary code (§9) |
| Tool harness + caching discipline | 1–2 weeks | Getting the prefix genuinely stable takes longer than expected |
| Guardrails, three levels | ~1 week + ongoing | The output guardrail needs a maintained adversarial set |
| Typed memory with caps and decay | ~2 weeks | Per `AI-LANDSCAPE.md` §22 |
| Evals + CI gates | **~2 weeks** | The part that gets skipped and shouldn't |
| Observability and traces | ~1 week | Belief-before / tools / belief-after / stopping reason |

**~2.5–3 months for one job type, built properly.** Consistent with the "40–60% larger build" estimate in `MATCH-THREADS-DESIGN.md` §9. Convo rescue is the right first and only job type — highest frequency, most deadline-bearing, and the one where goal-conditionality bites hardest.

## 19. What to build first, and what to cut

**Build in this order.** Each step is independently useful, and the first two contain no LLM at all:

1. **The goal schema and the belief state.** Ordinary code, unit-testable, no model calls. Writing down the six dimensions and their values is most of the conceptual work, and it forces the disallowed-intent decision in §14 to happen up front rather than late.
2. **`ask_choice` with hand-written questions**, selected by a hand-written decision tree. This tests whether structured elicitation improves advice *before* any inference machinery exists — and it can be tested with **human coaches** answering, in the M2.5 no-AI stage.
3. **Screenshot fact extraction**, replacing hand-entered situation facts.
4. **EVOI selection**, replacing the decision tree.
5. **Memory**, once there is a second session to remember into.

**Cut for v1:** learned question policies (the papers train them; hand-written candidates with EVOI selection gets most of the value), open-ended elicitation beyond one question, self-presentation inference (it belongs to the coach), and any multi-agent structure — one agent with seven tools is sufficient and far easier to eval.

## 20. Summary

**The goal insight is right and it is the design's centre.** Advice is goal-conditional, goal is a six-dimensional per-match vector rather than an onboarding answer, and it drifts.

**The key move is EVOI rather than EIG.** Don't resolve uncertainty about the goal — resolve only the uncertainty that would change the advice. That single substitution turns an interrogation into a conversation, and it makes the two-question budget achievable rather than aspirational.

**Most of the leverage is free or cheap.** Platform gives an intent prior strong enough to skip the question on Hinge. Reciprocity — the highest-leverage dimension — is inferable from screenshot metadata. The belief space is ~216 discrete states, so the inference is ordinary code rather than research.

**Three things are non-negotiable engineering.** A byte-stable prompt prefix, because caching is a 40–59% cost difference and the naive implementation busts it every turn. The no-send-ready-text output guardrail as a 100% CI gate. And disallowed intents as reachable enum values, so the goal classifier routes coercion to safety instead of coaching it.
