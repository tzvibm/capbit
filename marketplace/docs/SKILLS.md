# Wing — Skills as the Core Abstraction

**July 2026 · Why skills are the unit of everything, and the discipline that keeps them from sprawling**

Earlier documents treated skills as a context-budget device: eight playbooks stuffed costs ~6,400 tokens, indexed costs ~320 (`AGENT-MODEL.md` §2). True, and an undersell. **Skills are the unit of domain knowledge, of iteration, of coverage, and of learning** — which makes them the thing this product actually is.

---

# PART I — WHY FUNDAMENTAL, NOT OPTIMISATION

## 1. Progressive disclosure improves accuracy, not just cost

The token argument understates its own case. Measured effect of three-tier disclosure versus a single-prompt equivalent at similar task complexity: **~40% fewer tokens and task-completion accuracy up an estimated 15–20%** ([Firecrawl](https://www.firecrawl.dev/blog/agent-skills)).

That second number is the interesting one. Loading only the relevant playbook doesn't just save money — it removes competing instructions that would otherwise dilute the one that matters. A model told how to handle openers, stalls, ghosting, screening and date planning *simultaneously* handles each of them worse. **Skills are a precision mechanism that happens to also be cheap.**

## 2. Skills are where domain knowledge lives

The model knows how conversations work in general. It does not know what actually works in dating — *answer her question before you pivot; one specific day beats three options; after a three-day gap, don't open with a question.*

That knowledge has to live somewhere, and there are only three places:

| Where | Problem |
|---|---|
| System prompt | Always loaded, undifferentiated, dilutes everything else, unversioned |
| Fine-tuning | Slow, expensive, opaque, can't be reverted per-behaviour |
| **Skills** | Versioned text, loaded only when relevant, individually testable |

**The knowledge is the product, and the skill is its container.** Everything else in the architecture — perception, memory, routing — exists to get the right skill in front of the right situation.

## 3. Skills are the unit of iteration, which is the product loop

If advice quality *is* the product, then improving the product means editing a skill. That is a versioned text file with an attached eval, not a retrain and not a risky edit to a monolithic prompt where every change threatens every behaviour.

This is now standard practice: prompts as "versioned configs measured by attached graders," where "each change comes with evidence," A/B tested before promotion, and where **"domain experts can compress months of work into a week by iterating directly without waiting on engineers"** ([Braintrust](https://www.braintrust.dev/articles/ab-testing-llm-prompts), [Statsig](https://www.statsig.com/perspectives/prompt-versioning-managing-history)).

That last clause matters more here than in most products. **The person who knows dating does not have to be the person who knows code.** For a small team that is the difference between shipping domain quality and bottlenecking on engineering.

## 4. Skills are where outcome data becomes product

This is the most important reason and it is the one that makes skills strategic rather than architectural.

`answer_outcomes` produces a signal — this advice worked, that advice didn't. **Without skills, that data has nowhere to go.** You would have to retrain, or hand-tune a monolith, or just read the numbers and feel bad.

With skills, there is a write target:

```
outcome data  →  aggregate patterns  →  encode in the relevant skill
     ↑                                              │
     └──────────  better advice  ←──────────────────┘
```

*"Across 4,000 stalled threads, opening the recovery with a callback to something she said beat a fresh topic 1.6:1"* is a sentence that goes into `diagnose-stall/SKILL.md`. It is proprietary, it is derived from data nobody else has (because nobody else keeps the screenshot — `CONTEXT-STRATEGY.md` §1.5), and it compounds.

**And it survives model upgrades.** If the advantage is a clever prompt, a better model erases it. If the advantage is forty outcome-validated playbooks, a better model *amplifies* it — the same knowledge, executed better. That is the difference between being a wrapper and holding an asset.

## 5. Skills are the map of what the product can do

"Which situations does this handle well?" = "which skills exist, and how do they score?" That gives a legible competence map, a roadmap driven by the gap log (`AGENT-LOOP.md` §11), and an honest answer to what the product is not yet good at.

---

# PART II — ANATOMY

## 6. The description is the routing surface

The single most consequential line in a skill, because **it is the only thing the model sees before activation.** The named anti-pattern: *"a beautiful 4,000-word SKILL.md body with a lazy one-line description — the agent never reads the body if the description doesn't match."*

For Wing, routing is mostly deterministic (perception → stage → skill, per `AGENT-LOOP.md` §1), which makes descriptions *more* important rather than less: they are what a human reads when deciding whether a new skill overlaps an existing one.

## 7. A worked example

```markdown
---
name: diagnose-stall
description: A conversation that had rhythm has lost it. Use when the
  last exchange was 24h–14d ago, the thread previously had back-and-forth,
  and no reply is pending from the user. Not for never-started threads
  (see opener) or past 14 days (see ghost-recovery).
stage: stalled
output_modes: [read, verdict, reply]
version: 4
---

# Diagnose a stall

## Read the cause before proposing anything
Stalls have four causes and the fix differs. Identify which:

1. **Open loop** — she asked something and it went unanswered.
   *Most common, most fixable.* Check `open_loops` first; if present,
   this is the whole diagnosis. Answer it, don't pivot.
2. **Momentum decay** — replies got shorter and slower on both sides.
   Neither person did anything wrong. Needs a reason to re-engage,
   not an apology.
3. **Over-investment** — his messages grew longer as hers shrank.
   Check `length_ratio` trend. Do not send another long message.
4. **Natural pause** — she said she was busy/travelling. Not a stall.
   Say so and advise waiting.

## Rules
- **Never open a recovery with a question.** After a gap it reads as
  demanding. Give before asking.
- **Never reference the gap itself.** "Hey stranger", "did I lose you" —
  both make the silence the topic.
- If cause 3, the correct output is usually a **hold** (see the-call).
  Sending anything reinforces the asymmetry.
- One recovery attempt only. If it fails, the stage is `ghosted`.

## Evidence
- Callback to something she said beats a fresh topic (outcome data, v3)
- Recoveries sent 9am–noon outperform evening ones (outcome data, v4)
- Latency is U-shaped: don't advise replying instantly either

## references/
- `causes.md` — worked examples of each of the four
- `openings.md` — recovery openings that tested well
```

Three things to note. The **description states when *not* to fire**, which is what prevents overlap. **Rules are prohibitions as much as instructions** — in this domain what not to do carries more weight. And the **Evidence section is versioned against outcome data**, which is §4 made concrete: that section is why version 4 exists.

## 8. Structure and limits

Per the spec and the practice around it:

```
skills/diagnose-stall/
  SKILL.md          ≤ ~5,000 tokens — loaded entirely on activation
  references/       loaded only if the skill asks for them
  assets/           templates, if any
```

Keep the body to procedure; push examples and long reference material into `references/`. Stick to the core cross-agent spec — `name`, `description`, markdown body — so skills stay portable now that Agent Skills is an open standard.

---

# PART III — DISCIPLINE

## 9. What earns a skill

Skills sprawl if everything qualifies. The test — a candidate must meet **all four**:

1. **A distinct situation** that perception can identify without ambiguity.
2. **Different advice**, not just different wording. If the guidance is the same as an existing skill's, it's a branch inside that skill.
3. **Its own failure mode.** If it can't fail in a way another skill can't, it isn't separate.
4. **An eval set** of at least a few real situations. No skill ships without one, because otherwise you cannot tell whether editing it helped — which forfeits §3, the reason skills are valuable at all.

Typical registries run 5–10 skills; 30+ works without context problems. **Wing should launch with one** (`AGENT-PRODUCT.md` §14) and earn the rest from the gap log.

## 10. What is not a skill

| Not a skill | Because |
|---|---|
| **Anything detection** — stage, state, intent, escalation | Perception. Runs always; can't be progressively disclosed without a bootstrapping circle (`AGENT-LOOP.md` §1) |
| **Tone or voice** | A parameter, not a playbook. It varies within every skill |
| **Safety** | Must run before and independently of skill selection |
| **Memory management** | Infrastructure |
| **A single heuristic** | Belongs *in* a skill. One rule is not a playbook |

## 11. Routing and precedence

Selection must pick exactly one primary skill. Rules, in order:

1. **Safety pre-empts everything.** No skill runs.
2. **Stage determines the default** — one skill per stage is the primary.
3. **User invocation overrides.** If they asked for a roast, roast.
4. **`the-call` can pre-empt any output mode** — if holding is right, that supersedes whatever the stage skill would have produced. This is the one cross-cutting override and it earns it, because "send nothing" has to be able to beat every skill that wants to send something.
5. **Ties go to the narrower description.** If two fire, that is a registry bug — log it as a gap.

## 12. The flywheel, stated plainly

```
subject memory  →  outcome data  →  patterns  →  skill edits  →  better advice
      ↑                                                              │
      └──────────────────  more usage  ←─────────────────────────────┘
```

Every stage of that loop is impossible for the current category. They discard the screenshot, so there is no subject memory; without it there is no outcome linkage; without that there are no patterns; and with a single stuffed prompt there is nowhere to write a pattern even if they had one.

**That is the argument for skills being fundamental.** Not that they save tokens — that they are the only component in the architecture where accumulated learning can be written down, versioned, tested, and shipped.

---

# PART IV — TACTICS, AND SELECTION AT SCALE

## 13. Skills don't scale flat — and the threshold is low

The instinct that skill selection is the same problem as tool selection is correct, and the research puts a hard number on it:

| Finding | Source |
|---|---|
| **After 10–15 tools, selection accuracy drops** | [webscraft](https://webscraft.org/blog/tool-rag-scho-robiti-koli-u-agenta-zabagato-instrumentiv?lang=en) |
| At 49–741 tools, performance drops **7–85%** | [arXiv 2606.17519](https://arxiv.org/pdf/2606.17519) |
| At 527 tools, **retrieval errors cause ~50% of all agent failures** | [arXiv 2606.17519](https://arxiv.org/pdf/2606.17519) |
| Two-level hierarchy — route to an agent, which selects from its own subset — scales to thousands | [Tool-to-Agent Retrieval](https://arxiv.org/pdf/2511.01854) |

**The current registry is already at the edge.** Nine stage skills plus `roast`, `screening` and `the-call` is eleven. Adding a library of technique skills to a flat registry would cross the threshold immediately and make *selection* the dominant failure mode — retrieval errors are half of all failures at scale, which means the thing that breaks is not the advice but the choosing.

## 14. Two tiers: skills and tactics

So split the abstraction rather than growing one registry.

| | **Skill** | **Tactic** |
|---|---|---|
| Is | A playbook for a *situation* | One *technique*, with conditions |
| Scope | Stage-scoped | Cross-cutting |
| Size | ~800–5,000 tokens | **~100–200 tokens** |
| Count | **Hard cap: 12** | Unbounded — hundreds is fine |
| Selected by | **Deterministic** — perception → stage → skill | **Retrieved** by relevance, within the active skill |
| Loaded | On trigger | 2–4 injected into the skill's context |

This is precisely the two-level hierarchy that scales: the **stage skill is the "agent"** and **tactics are its "tools."** Level 1 stays deterministic and small, so it never suffers retrieval error at all. Level 2 can grow indefinitely because tactics are never selected from a flat pool — only from those tagged for the active stage.

```
perception → stage → skill (deterministic, ≤12)
                       └→ retrieve 2–4 tactics tagged for this stage + signals
```

### Tactics form a graph; skills do not

The tactic layer is where dependency-aware retrieval earns its keep, and there is a direct result for it. **Graph-of-Skills** ([arXiv 2604.05333](https://arxiv.org/abs/2604.05333)) identifies the failure mode precisely: semantic retrieval "surfaces topically relevant skills but **misses their prerequisite chain of upstream and downstream skills**, creating a **prerequisite gap** that leaves the retrieved bundle execution-incomplete." Its fix — build the dependency graph offline, then retrieve a bounded dependency-aware bundle — reports **+43.6% average reward and −37.8% input tokens versus full skill-loading**, across three model families.

Note it beats *loading everything*, not merely flat retrieval. Graph > stuffing > flat semantic retrieval.

**But apply it at the right layer.** GoS solves for massive libraries; Wing's skill registry is capped at twelve and routed deterministically, where a graph is overkill and hand-written precedence is clearer. **Tactics are the unbounded layer, and they genuinely have prerequisites:**

```
established-rapport ──→ specific-day ──→ confirm-logistics
        │
        └──→ callback-to-detail   (requires: stored history exists)
```

*"Callback to something she said"* is incoherent without stored history. *"Propose a specific day"* presupposes rapport. Retrieving one without its prerequisite produces advice that is locally sensible and situationally wrong — which is the prerequisite gap, in this domain.

So: **`prerequisites` becomes a tactic field, and retrieval walks the graph before ranking.** At a few hundred tactics that's a small offline graph and an ordinary traversal, not a research problem.

**Tactic anatomy** — small enough that a few fit in the budget:

```markdown
---
id: specific-day
stages: [rapport, escalation-ready]
conditions: reciprocity=mutual, open_loops=0
evidence: strong
---
Propose one specific day, not "sometime" or a choice of three.
Options read as scheduling; one day reads as a decision.
Fails if: she has already said she's busy this week.
```

Note the `conditions` field. That is what makes retrieval precise rather than fuzzy — tactics are filtered by state before relevance ranking, so the candidate pool at any moment is small.

## 15. Authoring: AI-assisted, human-gated, evidence-tagged

Authoring skills and tactics with AI help is right, and it is exactly the workflow the prompt-as-product tooling describes: **draft with AI → human edits → eval set attached → versioned → A/B tested before promotion.** The AI drafts; it does not decide what ships.

**Three sources, in descending order of trust:**

**1 · The research base.** Better than the practitioner canon because it is conditional and mechanistic. Some findings that are immediately encodable as tactics:

| Finding | Source |
|---|---|
| **Reciprocal turn-taking disclosure beats extended one-sided disclosure** — alternating outperforms monologue, even measured at the end of the interaction | [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S002210311300070X) |
| **Moderate self-disclosure is optimal** in online dating — not maximum | [Current Psychology](https://link.springer.com/article/10.1007/s12144-026-09787-y) |
| **Positive humour beats negative humour** — and the difference shows up for long-term intentions specifically | [In-Mind](https://in-mind.org/article/funny-thing-happened-way-romance-how-humor-influences-romantic-relationship-initiation) |
| **Self-deprecating humour works for high-status people and not for low-status ones** | [Langley & Shiota](https://journals.sagepub.com/doi/10.1177/01461672231202288) |
| **Attitude similarity is among the most potent predictors of attraction** | [In-Mind](https://in-mind.org/article/funny-thing-happened-way-romance-how-humor-influences-romantic-relationship-initiation) |

That fourth one is the kind of thing only a conditional tactic can carry: self-deprecation is good advice for a confident user and actively harmful for an insecure one. A monolithic prompt cannot express that. A tactic with a `conditions` field can.

**2 · Your own outcome data** — the highest-trust source once it exists, and the only proprietary one (§4).

**3 · Practitioner literature**, filtered. Which brings us to the filter.

## 16. The filter: would it still work if she knew you'd read it?

Pickup and seduction material is genuinely mixed, and the useful filter is mechanical rather than a matter of taste or vocabulary:

> **Does the technique work by giving the user something real to say or do — or by exploiting a predictable reaction in someone who doesn't know it's being applied?**

| Survives disclosure | Requires concealment |
|---|---|
| "Propose one specific day, not three options" | Manufactured scarcity — fake busyness, delayed replies as tactics |
| "Alternate disclosure; don't monologue" | Negging — undermining her confidence so she seeks approval |
| "Tell a story instead of asking a third question" | Manufactured jealousy, false social proof |
| "Positive humour, not negative" | Anything designed to overcome a stated no |

**This is an effectiveness filter as much as an ethical one, which is why it's the right one to use.** Techniques that require the target's ignorance are *fragile*: they fail if she has seen them before, if the user executes clumsily, or if she simply asks what he's doing. Wing's users are amateurs executing under emotional stress — fragile techniques are bad engineering for that population regardless of anything else.

And the evidence already collected points the same way. Reviews of the incumbent category describe output that is *"clever, performative, slightly too smooth — exactly the trying-to-impress energy that gets ignored."* The commitment-stage finding says a genuine, vulnerable, self-written message beat both AI and a professional coach. **The seduction canon's characteristic register is the one the data says underperforms.** Negative humour losing to positive humour is the same result arriving from the academic side.

So the boundary is not squeamishness about tactical advice — tactical advice is the product, and bland therapy-speak is the failure mode the reviews are complaining about. **Be tactical. Just prefer the tactics that survive the other person knowing about them**, because those are the ones that keep working.

Two hard stops remain, already specified: **anything aimed at overcoming a stated refusal routes to safety** (`AGENT-HARNESS.md` §14, disallowed intents as reachable enum values), and **roast the user's own material, never a person** (`AGENT-LOOP.md` §9).

## 17. Summary

**Skills are not a context-budget trick.** Progressive disclosure independently improves task accuracy 15–20%; skills are the only sane home for domain knowledge; they are the unit of iteration, so improving the product means editing a text file with an attached eval; they let a domain expert improve quality without an engineer; and they are the **write target for outcome data**, which is what turns a data flywheel into a product.

**They survive model upgrades.** A prompt advantage is erased by a better model. Forty outcome-validated playbooks are amplified by one.

**The discipline that keeps them honest:** four tests to earn a skill, no skill without an eval, detection is never a skill, and `the-call` is the only cross-cutting override — because "send nothing" must be able to beat every skill that wants to send something.

**And they don't scale flat.** Selection accuracy drops past 10–15 items and retrieval errors become ~50% of failures at scale, so the registry splits in two: **≤12 stage skills chosen deterministically**, and an **unbounded library of ~150-token tactics retrieved within the active skill**. Level 1 never suffers retrieval error because it never retrieves; level 2 grows indefinitely because it only ever selects from tactics tagged for the current stage and state.

**Author with AI, gate with humans, tag with evidence** — and filter source material on whether a technique **survives the other person knowing about it**. That test is mechanical rather than moral, and it selects for robustness: techniques requiring concealment break when the user executes them badly, which amateurs under stress reliably do.
