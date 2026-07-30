# Wing — What Sales AI Already Solved

**July 2026 · Transfers from agentic sales tooling to the dating coach**

The observation is right and it is the most useful analogy available. Agentic sales tooling — Gong, Microsoft Copilot for Sales, Sybill, Coffee, Anthropic's own Cowork sales plugin — solves a structurally identical problem: **multi-turn persuasion toward a goal, one counterparty per thread, state accumulating over weeks, where the right next move depends entirely on where you are and what you want.**

It is a far better reference than the rizz apps, because sales has had forty years of iteration on the exact sub-problem Wing found hardest: **how do you know what the user is actually trying to achieve, without interrogating them.**

Five things transfer directly. One finding refines a rule I changed last week. And there is a line the analogy must not cross (§9).

---

# PART I — THE FIVE TRANSFERS

## 1. Progressive schema depth — the answer to the two-question budget

`AGENT-HARNESS.md` §7 set a hard budget of two questions before first output, and treated the goal vector as one fixed six-dimension schema. Sales does it better: **the schema itself changes depth by stage.**

> "Asking a prospect for their budget, timeline, and decision process on a first call is **interrogation, not discovery**." Frameworks layer across stages: **BANT for initial screening, CHAMP for discovery, MEDDIC for deal qualification** ([Salesmotion](https://salesmotion.io/blog/lead-qualification-framework-guide), [Outreach](https://www.outreach.io/resources/blog/sales-opportunity-qualification))

BANT is four fields — Budget, Authority, Need, Timeline. MEDDIC is six, and heavier. The rule is that **light schema early, heavy schema only once the thing is real** keeps early interaction fast without leaving late-stage state thin.

Applied to Wing, replacing the single fixed vector:

| Stage | Schema | Fields | Rationale |
|---|---|---|---|
| **New match** | Light | Interested? · What are you after? · Are they responsive? | Three fields, mostly inferable. A new match doesn't justify a goal interview |
| **Active** | Medium | + timeline · reciprocity read · investment appetite | Earned by the conversation continuing |
| **Real** (met, or heading there) | Full | + constraints · what they're worried about · self-presentation | Only for matches that reached stakes |

**This is a genuine improvement over what I specified.** A fixed six-dimension schema means either interrogating on match one or leaving most fields empty forever. Staged depth fixes both, and it makes the two-question budget comfortable rather than tight — at the light stage there are only three fields to fill and two are inferable from a screenshot.

## 2. Qualification versus discovery — two jobs, not one

A distinction Wing does not have, and it maps cleanly onto two segments the earlier research already identified:

> **"Qualification tells you whether to pursue a deal, while discovery tells you how to win it."**

| Sales | Wing | Who it serves |
|---|---|---|
| **Qualification** — is this worth pursuing? | *"Is he worth your Thursday?"* — screening, standards, time protection | The screening job type. Historically Segment C |
| **Discovery** — how do we win it? | *"How do I not blow this?"* — the conversation help everyone assumes the product is | The volume job |

These are **different products with different outputs**, and conflating them is why generic dating advice is useless. The screening job's correct answer is frequently *disengage*, which no reply-generator will ever produce — and which is The Call register in a different costume.

## 3. Gap flagging — the best idea in the whole analogy

This one is genuinely new to the design and it may be the strongest single transfer.

> "Sales Brain analyses call data and **surfaces qualification gaps in real time. If a champion hasn't been identified by stage three, it flags it.** If the economic buyer hasn't appeared in any conversation, that gap shows up in the deal score."

**What's missing from the state is more informative than what's in it.** The system doesn't ask a question — it notices an absence and names it.

Dating equivalents, all computable from state Wing already holds:

- *"Four weeks in and she has never asked you a question."*
- *"You've never mentioned anything you actually care about."*
- *"Three conversations, and you've suggested meeting in none of them."*
- *"Every message you've sent has been in the evening after 10pm."*
- *"She's proposed a time twice; both times you deflected."*

This is **insight without interrogation**, which is exactly what the two-question budget needs. It produces the product's most valuable moments — the ones that feel like being noticed — without spending question budget at all. And it is a natural feed for the **Insights surface** (`AGENT-PRODUCT.md` §3), which was the weakest-specified part of the product.

## 4. Behavioural signals — the reciprocity read, already quantified

Gong analyses **300+ signals per conversation**: talk ratios, sentiment shifts, objection patterns, response times, and finds an optimal **talk-time ratio of ~43% prospect / 57% rep**. Deal health degrades on "a sudden drop in meeting frequency, lack of multi-threading, or vague answers about budget and timeline."

That is precisely the reciprocity read from `AGENT-HARNESS.md` §4 — message length ratio, reply latency, who initiates, who asks questions — except sales **quantified it and found the optimum empirically.** Nobody has done that for dating.

Two consequences. It validates that behavioural metadata is the highest-leverage signal, which was previously an assertion. And it defines a research programme: *what is the optimal message-length ratio in a conversation that leads to a date?* That is answerable from `answer_outcomes` at scale, and it would be a genuinely defensible proprietary finding — the kind of thing that compounds.

## 5. Auto-population from conversation

> "AI listens to discovery conversations, identifies when BANT or MEDDIC criteria are mentioned, and **populates the CRM automatically**" — Sybill, Coffee, and others fill framework fields without human data entry.

This validates the inference-versus-ask split already specified: spend questions only on what cannot be observed. Sales does exactly this and the pattern is mature. Wing's version reads screenshots rather than call transcripts, but the architecture is the same and the failure modes will be too.

**Also worth noting:** Anthropic's Agent Skills became an **open standard in December 2025**, and Claude Cowork ships a sales plugin built on it — so the skills-with-progressive-disclosure pattern recommended in `AGENT-MODEL.md` §2 is now a cross-product standard rather than a Claude Code idiosyncrasy.

---

# PART II — THE FINDING THAT REFINES THE DRAFTING RULE

## 6. Autonomous AI writing underperforms AI-assisted human writing by 40–60%

From the sales side, with a number attached:

> AI-generated outreach is "technically personalised but emotionally vacant… references a prospect's job change in a way that feels like a lookup table rather than genuine human care, **and recipients can tell.**" And: **"Response rates on fully autonomous AI outreach consistently run 40–60% lower than outreach where a human wrote the message using AI-gathered research"** ([Cotera](https://cotera.co/articles/ai-sales-agent-guide), [Instantly](https://instantly.ai/blog/ai-sales-agent-mistakes-avoid/))

**This does not reverse last week's correction — it sharpens it.** Both findings are true and they are not in conflict:

| Configuration | Evidence |
|---|---|
| AI writes alone vs **unaided average person** | AI wins (dating openers, ~60% vs 48%) |
| AI writes alone vs **human writing with AI research** | **AI loses by 40–60%** (sales outreach) |

The optimum is neither. It is **human writes, machine researches and structures** — which beats AI-alone *and* unaided-human-alone.

**Product consequence, and it is concrete:** keep drafting (you were right that withholding it was wrong), but make the highest-performing path the default one. The draft should arrive with the *research visible* — here's what's actually going on, here are the two things in her last message worth picking up, here's a draft — and an **"make it yours"** step that is presented as the strong move rather than as extra friction. The current stage-decay rule already points this way at the commitment stage; this evidence says the same posture pays off everywhere, just less dramatically.

The corollary is a warning about the thing that would kill the product: **"technically personalised but emotionally vacant, and recipients can tell."** That is chatfishing described from the sender's side, with a measured cost.

---

# PART III — WHERE IT BREAKS

## 7. Five asymmetries, honestly

| | Sales | Dating |
|---|---|---|
| **Context available** | CRM, firmographics, email opens, calendar, call transcripts, website behaviour | **A screenshot.** The context-aggregation layer sales AI depends on does not exist |
| **Outcome signal** | Closed-won / closed-lost, dated and attributed | **Ghosting** — a null signal, and the most common outcome |
| **Volume per user** | A rep runs 50+ deals, hundreds of calls a year | A dater has ~3 active matches, maybe 30 conversations a year |
| **Learning substrate** | Gong trains on "billions of interactions" across an org | Per-user data is tiny. Any learning must be **cross-user**, which raises its own privacy questions |
| **Stakeholders** | Gartner: ~15 per enterprise purchase, hence MEDDIC's stakeholder mapping | **One.** The schema is simpler, which is the one place dating is easier |

The volume asymmetry is the most consequential. Sales AI works partly because a rep generates enough data to personalise against. A dater does not, which means Wing's `you/MEMORY.md` will be thin for months and its value depends on **being right from few observations** rather than on accumulating many. That is an argument for hand-built heuristics early and learned ones much later.

## 8. Sales has a verification loop too, and it is the same one Wing lacks

Worth noting because it recurs: sales AI is verified by pipeline outcomes and by a manager reviewing calls. Gong's "coachable moments" go **to a human manager**. The five-layer sales architecture explicitly includes *Governance and Human-in-the-Loop Controls* as a layer.

So the mature analog in this space **kept a human in the loop** — which is the gap `AGENT-PRODUCT.md` §13 named as the real cost of dropping the coach. It does not change the pivot decision, but it means the standalone product is attempting something the best-funded analogous category did not attempt.

## 9. The line the analogy must not cross

Sales vocabulary applied to a person you are dating would be a brand catastrophe, and the reason is a genuine asymmetry rather than squeamishness.

**In sales, both parties know a sale is being attempted.** The prospect understands the rep's goal. In dating, the other person does not know they are being processed by a system optimising toward an objective. That difference is why "deal health score: Priya 34/100" is repellent where "deal health: Acme Corp 34/100" is unremarkable.

**Borrow the internals. Never the surface.**

| Internal (fine) | Surface (never) |
|---|---|
| Compute an interaction-health signal | Show a score for a person |
| Track stage progression | "Move her to stage 3" |
| Detect declining engagement | "Deal at risk" |
| Flag a state gap | "Champion not identified" |

The operative rule, which is already in the spec and which the sales analogy stress-tests: **`MATCH.md` holds state and history, never portraiture** (`AGENT-PRODUCT.md` §5). A score attached to a person *is* portraiture. **Score the interaction, not the human** — "this conversation has lost momentum" carries the same information and is about the thing the user actually controls.

---

# PART IV — WHAT CHANGES

## 10. Concrete amendments

| # | Change | Where |
|---|---|---|
| **1** | **Goal schema becomes staged** — light (3 fields) / medium / full, deepening as the match becomes real, rather than one fixed six-dimension vector | `AGENT-HARNESS.md` §1 |
| **2** | **Add gap flagging** as a first-class output. Absences in the state are named without spending a question. Primary feed for Insights | `AGENT-PRODUCT.md` §3, §8 |
| **3** | **Split qualification from discovery** as separate job types — *should I pursue this* is a different product from *how do I not blow this* | Skills registry |
| **4** | **Drafts arrive with the research visible plus a "make it yours" step**, presented as the strong move — the 40–60% finding says assisted-human beats autonomous everywhere, not just at commitment | `AGENT-PRODUCT.md` §6 |
| **5** | **Interaction-health signal, never a person score.** Behavioural metadata computed internally; surfaced as momentum language about the conversation | `AGENT-PRODUCT.md` §5 |
| **6** | Research programme: **find the dating equivalent of the 43/57 talk ratio** from outcome data. Genuinely proprietary if it works | `answer_outcomes` |

## 11. Summary

**The analogy holds and it is the most useful reference found so far.** Sales solved the goal-elicitation problem Wing found hardest, and the solution is *staged schema depth* — light early, deep once it's real — which fixes the tension between a two-question budget and a six-dimension vector.

**The best single idea is gap flagging.** Sales AI's most valuable move is noticing what is *absent* from the state — no champion by stage three — and naming it. Its dating equivalents are immediate and strong, they cost no question budget, and they are the natural content for the Insights surface.

**One finding sharpens the drafting rule rather than reversing it.** Autonomous AI outreach underperforms AI-researched human writing by 40–60%, while AI still beats an unaided average person. Both are true: the optimum is machine researches, human writes. Keep the drafts, make "make it yours" the default path.

**And the line worth holding:** in sales both parties know a sale is happening. In dating they do not. Borrow the internals, never the vocabulary — score the interaction, never the person.
