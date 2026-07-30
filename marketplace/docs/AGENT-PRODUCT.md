# Wing — The Agentic Dating Coach

**v1 · July 2026 · Primary product specification**

A standalone agentic dating coach: skills, subagents, durable memory, scoped context, and question generation that verifies its own understanding. No human marketplace.

**Status of the other documents.** The marketplace research in `MARKET-ANALYSIS.md`, `GO-TO-MARKET.md`, `COACH-RECRUITING.md` and `MATCH-THREADS-DESIGN.md` is **parked, not deleted** — the market sizing, competitor pricing, regulatory findings and unit-economics work all still apply, and the human layer may return later as a premium tier. Where those documents assume a marketplace, this one supersedes them. `AGENT-HARNESS.md`, `CONTEXT-STRATEGY.md` and `AGENT-MODEL.md` remain fully in force; this document is the layer above them.

---

# PART 0 — THE DECISION, AND WHAT I'D WATCH

## 1. Where the earlier analysis was wrong

`AI-LANDSCAPE.md` §15 scored a pure AI product at 12/30 and an agentic one at 14/30, largely on a saturation score of 1–2 out of 5. **That was wrong, and the correction is the argument for this pivot.**

The category is saturated with *one-prompt products*. It is empty of agentic ones. My own teardown (`CONTEXT-STRATEGY.md` §1–3) found that the market leader — 7.5M users, $15M revenue — models the user's goal as a **five-option tone dropdown**, and that across the entire category nobody infers goals, computes reciprocity, holds per-match state, captures outcomes, or verifies anything. Saturation of a category with weak products is not saturation of the category. Revised: **agentic scores ~19, not 14.**

Removing the human layer also removes the hardest unproven thing in the plan — recruiting supply, cold-starting a two-sided market, and controlling quality at sub-$20/hr labour — none of which had been demonstrated. Adjusted for execution risk rather than paper score, this is the better shape for a small team.

## 2. Three risks that do not go away

Stated once, with where each is answered:

| Risk | Status | Answer |
|---|---|---|
| **Rizz makes $15M with a tone dropdown** — sophistication may not be what people buy | Real | Sophistication must earn its keep on **retention**, which is the one number the category is bad at (§14). Not on being better engineering |
| **ChatGPT Projects is the substitute** — persistent typed workspaces, files, editable memory, 100Ms of users at $20/mo | Real | This is a product advantage, not a moat: purpose-built capture, a goal model, enforced scoping, dating-tuned skills, outcome tracking. Wins on focus if it wins |
| **Incumbents bundle free** — Tinder/Hinge AI wingmen, Grindr shipping per-match memory to 14M by 2027 | Real and worsening | Cross-platform, user-owned memory, and a depth no bundled feature will match. Genuinely the weakest of the three answers |

**And one that got harder by removing the human**, which is the subject of Part III: the coach was the verification layer, and there is now nothing checking the agent.

---

# PART I — THE PRODUCT

## 3. Shape

```
┌──────────┬──────────┬──────────┬──────────┐
│ Matches  │ Profile  │ Insights │   You    │
└──────────┴──────────┴──────────┴──────────┘
```

| Surface | Contains |
|---|---|
| **Matches** | One agentic thread per person. Goals, state, history, advice, outcomes |
| **Profile** | Bio, photos, prompts — the artifact scope. Version history and what changed |
| **Insights** | The `you/` memory made visible: your patterns, what works for you, what you keep doing |
| **You** | Account, memory controls, export, delete |

**Insights is the retention surface and it did not exist in the marketplace design.** It is where the compounding asset becomes visible to the user — *"you've opened with a question in 9 of your last 11 conversations; the two that didn't both got replies."* Nothing in the category does this, and it is the clearest answer to the churn problem, because it is the one screen that is worth more the longer you stay.

## 4. Pricing

Subscription only. The marketplace take-rate arithmetic is gone, which simplifies everything.

| Anchor | Figure |
|---|---|
| Rizz | ~$7/week or ~$20/month |
| AI chatbot apps, subscription-led | $30–100+ blended annual ARPU |
| RevenueCat median Y1 RLTV per payer, North America | $32 |
| Grindr EDGE (proves the premium ceiling) | $349–500/month |

**Recommendation: $12–15/month, not $8.** The CAC arithmetic in `MATCH-THREADS-DESIGN.md` §11.2 was marginal at $8 (~$61/payer against $20–80 CAC). At $14/month and an 8.2-month average life it's ~$107/payer, which moves LTV:CAC from ~0.8–3:1 to **~1.3–5:1** and crosses the 4:1 benchmark at the good end. Price is the cheapest lever available and the category supports it.

Free tier stays **hard-capped** — two match threads, twenty exchanges — for the reason established earlier: free converts on freemium economics (2–5%), capped trials on trial economics (5–20%).

---

# PART II — THE FOUR SUBSYSTEMS

## 5. Memory

Per `AGENT-MODEL.md` §1 and `AI-LANDSCAPE.md` §22 — typed by **data subject**, not by task.

```
you/MEMORY.md            always loaded, grows, distilled       ~1,000 tok
profile/NOTES.md         artifact scope, version history
matches/<id>/MATCH.md    this match only, capped, decaying       ~600 tok
platform/RULES.md        always, user-unwritable                 ~400 tok
```

Scopes **concatenate, never override**. Match scopes never co-occur in one context — enforced in the query layer, because Chroma's finding that *semantic similarity drives decay more than length* makes cross-match contamination the signature failure of this domain (`CONTEXT-STRATEGY.md` §6).

`MATCH.md` holds **state and history, never portraiture** — no personality assessment, no appearance, no inferred psychology. This is now a product decision rather than a legal one, since there is no marketplace to protect, but it stays for two reasons: it is what makes user-owned exportable memory a credible trust claim, and GDPR Art. 6(1)(f) balancing still weighs profile detail and comprehensiveness on a non-consenting third party.

## 6. Skills

Progressive disclosure — name and description indexed at ~40 tokens, body loaded on trigger. This is load-bearing: eight playbooks stuffed is ~6,400 tokens and roughly doubles the request into the degradation range; indexed it is ~320.

```
skills/
  opener/            "First message to a new match"
  convo-rescue/      "A conversation has stalled or gone cold"
  screening/         "Is this person worth more time?"
  date-plan/         "Planning or deciding about a date"
  the-call/          "When the right advice is to send nothing"
  bio-review/        "Improving profile text"
  photo-lineup/      "Choosing and ordering photos"
  post-date/         "Debrief after meeting"
```

Each skill declares its playbook **and its renderer**, so a photo lineup renders as a grid and a bio review as a diff. Adding a job type is adding a directory.

## 7. Subagents — and precisely what they are not for

The research here is unhelpful and must be reported straight. **Naive self-critique does not work:** prompting a model to check its own work without external grounding *degrades* performance; absent external feedback LLMs largely cannot self-correct reasoning, and naive self-correction can make answers worse. Models **measurably favour their own output** in self-review ([SELF-INCORRECT](https://arxiv.org/pdf/2404.04298), [Zylos](https://zylos.ai/research/2026-04-10-llm-as-judge-production-agent-verification-2026/)).

Two qualifiers rescue the pattern, and they define the design:

1. **Separation works where self-review doesn't.** Adversarial review splits maker from checker across **separate context, instructions, and model** — the writing agent is a weak checker of its own output, a separate critic is not.
2. **Checkable constraints behave differently from open-ended reasoning**, and checkable constraints are where unaided verification retains value.

So: **subagents get information the main agent lacks, or check things that are mechanically checkable. Never "is this good advice?"**

| Subagent | Gets | Asked | Why it works |
|---|---|---|---|
| **Her-eyes** | Thread + draft only. **Not** the user's goals, not the agent's reasoning | "You received this. What do you think?" | Different *information*, not a second opinion. The main agent is contaminated by knowing the intent |
| **Consistency** | `you/MEMORY.md` + `MATCH.md` + proposed answer | "Does this contradict anything known? Did we advise the opposite before? Does it violate a stated constraint?" | A **checkable constraint** task — the case where verification works |
| **Pattern miner** | All of one user's match history | "What does this person keep doing?" | Async, off the request path. Writes `you/MEMORY.md`. This is what makes memory compound |
| **Skill workers** | One bounded artifact | Photo lineup, bio diff | Ordinary task decomposition |

Different model family for Her-eyes and Consistency than for the main agent, per the separation finding and the LLM-as-judge guidance never to use the same family as generator and judge.

## 8. Questions — three kinds, three jobs

The most under-built thing in the category, and the sharpest differentiation. **Three distinct question types**, and conflating them is the standard mistake.

| Type | Targets | Oracle | Budget |
|---|---|---|---|
| **Elicitation** | The **goal** — intent, investment, timeline | The user knows it | ≤2 before first output, EVOI-selected (`AGENT-HARNESS.md` §5) |
| **Verification** | The **diagnosis** — "here's my read, is that right?" | **The user can check it** | 1, when the read drives the advice |
| **Constraint** | Blocking unknowns — "anything I should know?" | The user knows it | Rare; only when a wrong assumption is costly |

**The verification question is the important one and it is new.** Elicitation asks what you want. Verification confirms what the agent *thinks is happening* before it acts on it:

> *"Reading this, it looks like you're more invested than she is right now — she replies but never starts. Does that match how it feels?"*

Everything about that is user-checkable. They have ground truth the agent doesn't — that she's been travelling, that they met in person last week, that the flat replies are just how she texts. And getting it wrong is the failure that most damages trust, because a confident misread of your situation is worse than no advice.

---

# PART III — THE VERIFICATION PROBLEM

## 9. What was removed

`AGENT-MODEL.md` §7 argued that coding agents work because the compiler checks them, dating has no compiler, and the coach was therefore the verification layer. Removing the coach removes that. This is the central engineering problem of the pivot and it deserves a real answer rather than optimism.

**Three partial substitutes, none sufficient alone:**

## 10. Substitute 1 — the user is an oracle for diagnosis, not prescription

The cleanest structural insight available, and it falls out of splitting the two:

| Claim type | Can the user verify it? |
|---|---|
| **Observation** — "she took 3 days, you sent 2 messages in a row" | **Yes, trivially** — it's in the screenshot |
| **Diagnosis** — "you're more invested than she is" | **Yes** — they have context the agent lacks |
| **Prescription** — "ask her out on Thursday" | **No.** Nobody can, until it's sent |

So verification questions target diagnosis, where a check is genuinely available, and the prescription is where calibrated hedging goes (§12). **The agent gets a real verification loop for half its output** — which is not nothing, and no competitor has any.

## 11. Substitute 2 — separated adversarial subagents, on checkable things only

Per §7. Her-eyes supplies genuinely different information; Consistency checks mechanically verifiable constraints. Neither is asked to judge advice quality, because that is exactly where the research says self-verification fails.

**Honest limitation:** this catches contradiction, forgotten constraints, and drafts that read badly to a naive reader. It does not catch *plausible but wrong strategy*, which is the failure mode that matters most and which nothing available catches.

## 12. Substitute 3 — calibrated hedging, per claim

Also unhelpful research, also reported straight: LLMs **often do not know when they are wrong**; verbalised confidence is overconfident, clustering **80–100%**, and diverges from both token probabilities and actual accuracy. Models "prioritise helpfulness and decisiveness at the expense of expressing uncertainty" ([ConfidenceBench](https://arxiv.org/html/2607.20526), [arXiv 2607.03882](https://arxiv.org/pdf/2607.03882)).

But one finding is directly actionable: **medium-expressed uncertainty — specific hedging rather than confident assertion or reflexive "I don't know" — produces the best human–AI collaboration outcomes.**

So hedge **per claim, not per answer**, and derive the confidence class structurally rather than asking the model how sure it is:

| Class | Source | Rendered as |
|---|---|---|
| **Observed** | Screenshot facts, extracted | Stated plainly |
| **Inferred** | Reciprocity, intent, from the belief state | *"It looks like…"* — and offered as a verification question when it drives the advice |
| **Predicted** | What will happen if you send this | **Always hedged, never asserted.** *"My guess is…"* |

Deriving the class from provenance sidesteps the calibration failure entirely: the system knows which pipeline produced a claim even though the model doesn't know how confident it should be.

## 13. What this adds up to, honestly

**Verification is genuinely weaker without the human**, and no amount of architecture fully closes it. What the design gets:

- A real check on **observations and diagnosis** (the user)
- A real check on **contradiction and constraint violation** (Consistency)
- A real check on **how it reads to a naive recipient** (Her-eyes)
- Structural honesty about **predictions** (provenance-derived hedging)
- Delayed statistical signal on **outcomes** (`answer_outcomes`, days of latency, poor SNR, useful only in aggregate)

What it does not get: any check on whether the *strategy* is right. That gap is real, it is why `answer_outcomes` matters more here than it did in the marketplace design, and it is the honest reason a human premium tier may eventually be worth reintroducing — not as the business, but as the verification layer for the cases where being wrong is expensive.

---

# PART IV — BUILD ORDER

## 14. What to build, in sequence

Each stage is independently shippable and the early ones contain little or no novel machinery.

| Stage | Build | Proves |
|---|---|---|
| **1** | Screenshot → structured facts. Single-turn advice, no memory, no goals | Capture works; extraction is reliable |
| **2** | **Goal schema + belief state.** Ordinary code, unit-tested. `ask_choice` on a hand-written decision tree | Structured elicitation improves advice over a tone dropdown — the core bet, testable without any inference machinery |
| **3** | `MATCH.md` + `you/MEMORY.md`, scoped and budgeted. Verification questions | Memory changes advice quality; and whether the second session is better than the first |
| **4** | Skills, progressive disclosure, per-skill renderers | Multi-job-type without context bloat |
| **5** | EVOI selection replacing the decision tree; Her-eyes and Consistency subagents | Question efficiency; caught-error rate |
| **6** | Pattern miner → **Insights surface**; `answer_outcomes` | Retention — the number that decides the business |

**Stage 2 is the whole thesis and it needs no LLM machinery.** If a six-dimension goal model with two well-chosen questions doesn't beat a tone dropdown on advice quality, nothing downstream saves it. That is the equivalent of the old bet 1 and it should be run the same way: pairwise, both orderings, cross-family judge (`AGENT-HARNESS.md` §17).

## 15. Metrics that decide it

| Metric | Why it's the one |
|---|---|
| **Month-2 retention** | The category's fatal weakness — AI apps churn ~30% faster (21.1% vs 30.7% annual) on top of dating's sub-5% twelve-month survival. Memory is the only antidote, and this is where it shows up or doesn't |
| **Second-session advice quality vs first** | Direct test of whether memory earns its cost |
| **Verification-question hit rate** | How often the user corrects the agent's read. Too low means the questions are trivial; too high means the diagnosis is bad |
| **Insights engagement** | Whether the compounding asset is visible enough to retain |
| Questions to decision | Friction control, capped at 2 |

## 16. Summary

**The pivot is right for a reason worth stating precisely:** the category is saturated with one-prompt products and empty of agentic ones, and the hardest unproven part of the previous plan — recruiting and quality-controlling human supply — disappears.

**Four subsystems.** Memory typed by data subject and scoped per match. Skills with progressive disclosure, which is what makes multiple job types fit the context budget. Subagents that supply *different information* or check *mechanical constraints* — never ones asked to grade their own work, because that is documented not to function. And three distinct kinds of question, of which the **verification question** — *"here's my read, is that right?"* — is the genuinely novel one.

**The honest gap is verification.** The user can check observations and diagnosis; subagents can check contradiction and reception; provenance-derived hedging keeps predictions honest. Nothing checks whether the strategy is right. That is the real cost of dropping the human, it is not fatal, and it is worth remembering if a premium human tier is ever reconsidered — as the verification layer, not as the business.
