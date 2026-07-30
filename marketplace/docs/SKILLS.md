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

## 13. Summary

**Skills are not a context-budget trick.** Progressive disclosure independently improves task accuracy 15–20%; skills are the only sane home for domain knowledge; they are the unit of iteration, so improving the product means editing a text file with an attached eval; they let a domain expert improve quality without an engineer; and they are the **write target for outcome data**, which is what turns a data flywheel into a product.

**They survive model upgrades.** A prompt advantage is erased by a better model. Forty outcome-validated playbooks are amplified by one.

**The discipline that keeps them honest:** four tests to earn a skill, no skill without an eval, detection is never a skill, and `the-call` is the only cross-cutting override — because "send nothing" must be able to beat every skill that wants to send something.
