# Wing — Build vs Buy

**July 2026 · What to adopt off the shelf, what to still build, and what to delete from the spec**

The specs hand-rolled several solved problems — staleness handling, memory pruning, relevance injection, trend detection. Mature frameworks do these better. This document picks them, and more usefully, **deletes work already specified.**

---

# PART I — MEMORY

## 1. The pick: Graphiti (Zep's engine)

**Dating memory is almost entirely facts that change over time.** *She's keen → she's gone cold. He wants casual → he actually likes her. Momentum building → momentum lost.* That is the whole state model, and it happens to be the exact axis these frameworks are benchmarked on.

On **LongMemEval**, the temporal-retrieval benchmark:

| Framework | Score (GPT-4o) |
|---|---|
| **Zep / Graphiti** | **63.8%** |
| Mem0 | 49.0% |

**A 15-point gap on precisely the thing this product is made of** ([particula](https://particula.tech/blog/agent-memory-frameworks-tested-mem0-zep-letta-cognee-2026), [vectorize](https://vectorize.io/articles/best-ai-agent-memory-systems)). Zep also reports +18.5% on LongMemEval and 90% lower latency versus a MemGPT baseline.

## 2. Bi-temporal edges are what I specced by hand, done properly

I wrote "fetched facts carry timestamps and match state decays." Graphiti does the real version:

> Every edge carries **four timestamps**: `t_created` / `t_expired` (when the system learned or unlearned it) and `t_valid` / `t_invalid` (when it was true in the world). On conflict it **invalidates rather than deletes**, "preserving historical accuracy without large-scale recomputation" — so **"the agent never has to choose between a stale and a current fact."** ([Zep paper](https://blog.getzep.com/content/files/2025/01/ZEP__USING_KNOWLEDGE_GRAPHS_TO_POWER_LLM_AGENT_MEMORY_2025011700.pdf), [Neo4j](https://neo4j.com/blog/developer/graphiti-knowledge-graph-memory/))

Two consequences for Wing, and the second is the valuable one.

**The stale-state failure disappears as a category.** `MATCH.md` asserting "she's keen" from three weeks ago was a bug I flagged and then specced a weak fix for (decay + timestamps). Edge invalidation solves it structurally.

**Trend detection becomes a query rather than a feature.** `AGENT-LOOP.md` lists trend detection and gap flagging as things to build. On a bi-temporal graph, *"when did her reciprocity change?"* is a query over validity intervals. **That deletes a chunk of specified work** — and gap flagging, the strongest differentiator identified, gets easier rather than harder.

## 3. The alternatives, and why not

| Framework | Strength | Why not here |
|---|---|---|
| **Mem0** | Most mature self-host, Apache 2.0, three-tier scopes (user/session/agent) — matches the scope typing well | **Graph features gated at $249/mo Pro**; vector-only below. And the 15-point temporal gap is disqualifying for this domain |
| **Letta / MemGPT** | OS-level memory management for long-horizon agents | Wrong shape — sessions here are short bursts over weeks, not long runs |
| **Cognee** | Local-first, privacy-critical, 14 retrieval modes | Genuinely attractive given the third-party-data constraint. Keep as fallback if self-hosting Neo4j proves painful |

**Licensing reality:** Zep retired its self-hosted Community Edition in 2025. **Graphiti itself is Apache-2.0 and self-hostable, but requires Neo4j.** Zep cloud's Flex tier is **$25/month** including the full temporal graph. Start on cloud; the engine is open if that changes.

## 4. Adopt the engine, keep the policy

The framework gives storage, invalidation, retrieval and pruning. It does **not** know that:

- match-scoped memory must be capped and must never co-occur with another match in one context
- portraiture is forbidden — facts she volunteered yes, inferred traits never
- the user must be able to read, edit, export and delete everything

**Those stay ours.** The correct division: **Graphiti is the store; our policy layer decides what may be written and what may be assembled into a view.** A tool guardrail still rejects third-party trait writes; the framework will happily store whatever you give it.

---

# PART II — SCREENSHOT → JSON

## 5. The honest finding: nothing purpose-built exists

There is no off-the-shelf "chat screenshot → structured transcript" model. The closest candidates solve an **adjacent** problem — UI agents that find and click things, not transcript extraction:

| Model | What it does | Fit |
|---|---|---|
| **OmniParser** (Microsoft) | YOLO icon detection + fine-tuned Florence-2 → **DOM-like structure with bounding boxes** | Partial — see §6 |
| **Ferret-UI** (Apple) | Mobile UI referring/grounding, any-resolution training for fine-grained mobile screens | Adjacent |
| **ScreenAI** (Google) | UI and infographics understanding, screen annotation + QA | Adjacent |

So extraction is a **VLM with a constrained JSON schema**, not a specialist model. That is the boring answer and it's correct.

## 6. But there is a real design finding: attribution is geometry, not semantics

**Who said what is a layout question.** Left bubble = them, right bubble = you. A VLM reading the image *infers* this and will get it wrong on ambiguous cases — and getting it wrong is catastrophic, because it **inverts the entire read**. Every signal downstream (initiation ratio, length ratio, who asked the question) flips.

So the extractor should be a **hybrid**:

```
bounding-box geometry  →  sender attribution, message ordering   (deterministic)
VLM + JSON schema      →  text content, timestamps               (semantic)
```

This is where OmniParser-style element detection earns a place — not as the extractor, but as the **layout pass** that makes attribution deterministic instead of inferred. Even simple bubble-alignment geometry gets most of the way there.

## 7. Model choice: hosted frontier VLM, for robustness

Gemini supports JSON Schema-constrained structured output, and one benchmark matters more than raw accuracy here:

> **Gemini loses under 3.5 percentage points when image resolution drops from 200 DPI to 50 DPI. Open models in the 8–17B range lose 38–40 points.** ([datastudios](https://www.datastudios.org/post/can-google-gemini-read-images-and-screenshots-vision-capabilities-and-text-extraction-accuracy))

User-submitted phone screenshots are compressed, variably sized, sometimes screenshots-of-screenshots. **Robustness to degraded input is the binding requirement**, not peak accuracy on clean images — which argues against a small self-hosted model despite the cost appeal.

Favourably: Gemini's known weaknesses are "multi-column layouts, dense tables, small fonts, overlays and noise." Chat screenshots are single-column and high-contrast — the good case.

---

# PART III — WHAT CHANGES

## 8. Delete from the spec

| Specified | Replace with |
|---|---|
| Hand-rolled staleness and decay | **Graphiti edge invalidation** (§2) |
| Trend detection as a feature to build | **A query over validity intervals** (§2) |
| Custom distillation and pruning | Framework-provided |
| `MATCH.md` as the stored artifact | **Graph is the store; markdown becomes a rendered *view*** — for the prompt, and for the user-facing "would you be comfortable if she read this" surface |
| Bespoke relevance retrieval | Framework-provided, with our scope policy on top |

That last row is a genuine simplification. I specced markdown files as storage, which conflated store and view **in the very document that warned against conflating store and view.**

## 9. Still build — these are the product

| Component | Why no framework has it |
|---|---|
| **Goal / belief state, staged schema** | Domain-specific. Nothing off the shelf models dating intent |
| **Signals** — latency deltas, ratios, counts | Arithmetic. Never outsource, never hand to an LLM |
| **Perception + stage detection** | Domain-specific |
| **Skills and tactics registry** | **This is the asset** (`SKILLS.md`) |
| **Policy layer over memory** | Caps, portraiture rules, scope isolation |
| **Match identity resolution** | Which match does this screenshot belong to |
| **Outcome loop** | The flywheel |

## 10. Summary

**Memory: adopt Graphiti.** Dating state is entirely facts that change over time, which is the exact axis where it beats Mem0 by 15 points on LongMemEval. Its bi-temporal edges solve the stale-state bug structurally and turn trend detection from a feature into a query. Zep cloud at $25/mo to start; Apache-2.0 engine self-hostable on Neo4j if needed.

**Extraction: hosted VLM with a JSON schema, plus a geometry pass.** No specialist model exists. But **sender attribution is geometry, not semantics** — get it from bubble alignment rather than inference, because getting it wrong inverts every downstream signal.

**Keep the policy, keep the domain.** Frameworks give storage, invalidation and retrieval. They don't know that match memory is capped, that portraiture is forbidden, or what a good opener looks like. **The parts worth building are the parts nobody sells** — and the correction here is that I was building several of the parts everybody sells.
