# Wing — Competitor Architecture and the Context Question

**July 2026 · How agentic the wingman apps actually are, and whether stuffing beats structure**

Two questions: **what is the state of the art in wingman apps** (memory, state, goal handling — or just one stuffed prompt), and **is stuffing actually the better strategy anyway.**

Answers up front:

1. **The state of the art is one prompt with a screenshot and a tone dropdown.** Nobody in the top tier does goal inference, per-match state, or anything agentic. Exactly one product claims per-match memory, and it's a minor player.
2. **Stuffing is not better, and it fails earlier than people expect** — but the reason it fails is domain-specific and worse for dating than for almost any other use case (§6).
3. **Wing's answer is neither RAG nor stuffing.** The corpus is far too small to justify retrieval infrastructure and too semantically self-similar to stuff safely. The answer is **strict scoping plus distillation** — and the per-match thread structure already proposed *is* the context strategy (§8).

---

# PART I — WHAT THE COMPETITORS ACTUALLY DO

## 1. The teardown

| Product | Scale | Architecture | Goal handling | Per-match state |
|---|---|---|---|---|
| **Rizz** | 7.5M users, $15M rev, top-5 downloaded | Screenshot → LLM → replies. "Advanced LLMs… analysing the context provided by a user — either a screenshot of a chat or a match's bio" ([Fast Company](https://www.fastcompany.com/91187375/rizz-dating-app-ai-co-founder-roman-khaves-explains)) | **Tone dropdown**: funny / sweet / bold / romantic / savage | **None found** |
| **RIZZER** | — | Pick a vibe → generate. Optional custom prompt, or a **"Random"** button | Vibe selector | None |
| **YourMove AI** | 300k users | "Real-time learning… tailored to each user's style" — marketing language, no mechanism described | Unclear | None found |
| **Roast** | — | AI scoring + human expert review | Profile-scoped, not conversational | N/A |
| **Wingman.live** | Minor | Markets "each conversation in its own memory bank" and "memory-aware chat support" ([TextVibe](https://textvibe.app/blog/best-dating-text-helper-apps/)). **Direct testing found this means assistant-chat history only** — see §1.5 | Unclear | **No** — screenshot facts are discarded |

## 1.5 "Do they maintain thread context?" — three different questions

Calling these products "stateless" is imprecise and undersells them. There are three distinct senses of *thread context*, and the category scores very differently on each.

| Sense | Rizz et al. | Notes |
|---|---|---|
| **1 · Within-request context** — does the model see the conversation? | **Yes, richly** | The screenshot *is* the thread. The model sees who said what, order, lengths, often timestamps. "RIZZ AI chat processes conversation history to understand tone, context, and the best response approach" |
| **2 · Conversation memory** — does it remember what *you* said to *it*? | **Sometimes** | Ordinary assistant chat history. Rizz's "History Tracking" is an archive for the user to browse; Wingman.live persists the user's dialogue with the AI |
| **3 · Subject memory** — does it remember what's true about *the match*, extracted from screenshots? | **No. Nobody.** | See below — including Wingman.live, the one product that appeared to claim it |

### The distinction that actually matters: conversation memory vs subject memory

**Corrected by direct product testing, which supersedes the desk research.** I had recorded Wingman.live as the one exception, on the strength of a third-party listicle claiming it "keeps each conversation in its own memory bank." Hands-on testing found otherwise: **it persists the user's dialogue with the assistant, but not the match — not the messages, not the facts extracted from screenshots.** The screenshot is consumed to generate a reply and then discarded.

So the corrected finding is cleaner and stronger than what it replaces: **no product in the category holds subject memory.** And a secondary lesson worth carrying — *memory claims in this category are marketing copy, not architecture.* This is the second time a vendor's stated feature (Rizz's "history tracking", Wingman's "memory bank") turned out to describe something materially weaker than it sounds. Treat any future competitor memory claim as unverified until someone uses the product.

**Why conversation memory is nearly worthless here**, which is what makes the distinction load-bearing rather than pedantic:

- The valuable state is about **the match**, not about the user's chat with the assistant.
- Users don't have long conversations with these tools. They upload, take the reply, and leave. There is very little dialogue to remember.
- So remembering the assistant chat while discarding the screenshot is **remembering the wrapper and throwing away the contents.**

**What discarding the screenshot costs**, concretely:

| Capability | Requires subject memory? |
|---|---|
| Generate a reply to *this* screenshot | No — sense 1 suffices |
| Notice a trend — *is this getting better or worse?* | **Yes** |
| **Gap flagging** — *"four weeks and she has never asked you a question"* | **Yes, absolutely** |
| Link advice to outcome — *did what I suggested last time work?* | **Yes** |
| Anything in the Insights surface | **Yes** |

That third row matters most. **Gap flagging — the strongest idea taken from sales AI (`SALES-PARALLEL.md` §3) — is structurally impossible without subject memory.** You cannot observe that she has never asked a question if every screenshot is evaluated in isolation and then thrown away. The best available differentiator and the one universally missing capability turn out to be the same thing.

**Implementation note — match identity.** Subject memory needs to know which match a screenshot belongs to. The simple answer is that the user picks from their match list, which is one tap and is how the Matches tab works anyway. A cheaper trick is available as a refinement: consecutive screenshots of the same thread **overlap**, since dating apps render recent history, so a new upload can often be matched to an existing thread by content overlap with the stored tail. Treat that as a friction-remover with a manual fallback, not as the primary mechanism.

**Sense 1 is the reason one prompt works well enough to build a $15M business, and it deserves respect.** A screenshot is a dense, well-structured context payload that the user assembles for free. Any claim that these products "have no context" is wrong.

**Sense 3 is where the gap actually is**, and it is a narrower claim than I first made: they don't know it's the same person, they don't know what happened after the advice, and they know nothing outside the frame — goals, constraints, offline events, what the user is habitually bad at.

**The evidence is stronger in the failure symptoms than the feature lists**, because nobody publishes their architecture. Independent reviews describe exactly what missing sense-3 state predicts:

- output is "generic and repetitive — does not consistently sound like the user or **stay in tune with the specific match**"
- suggestions read "clever, performative, slightly too smooth — exactly the trying-to-impress energy that gets ignored"
- "in more complex or emotionally charged conversations, the AI's limitations become more apparent"
- and the deepest one: **"even the better outputs do not teach you anything, which means six months later you are still dependent on the tool and still flat the moment a conversation goes somewhere unexpected"** ([GetMatches](https://getmatches.ai/en/blog/rizz-ai-review), [SwipeStats](https://www.swipestats.io/blog/rizz-app-review))

That last line is the retention thesis written by a reviewer: no accumulation → no learning → dependency without improvement → churn. It is the precise gap `you/MEMORY.md` and the Insights surface target (`AGENT-PRODUCT.md` §3).

**Honest caveat:** from outside it is not possible to confirm whether any of these apps feed saved history back into the prompt. What is observable is that the marketing describes a user-facing archive rather than a context mechanism, that no product exposes a per-match record, and that the reported failure modes match the absence of one.

## 2. What "goal" means to the market leader

This is the finding worth sitting with. Rizz — 7.5M users, $15M revenue, top-five downloaded dating app — handles the user's goal as a **five-option tone selector**. Funny, sweet, bold, romantic, savage. Some variants offer Persuasive / Empathetic / Professional / Flirty / Explanatory, which reads like a generic writing-assistant menu that was never adapted for dating at all.

That is not a goal model. It is a style parameter. There is no representation of *what this user wants from this person*, no intent, no reciprocity read, no timeline, no investment level — the six dimensions in `AGENT-HARNESS.md` §1 are collapsed into "how should this sound."

**Two opposite conclusions follow, and both are true.**

**The optimistic one:** the bar is on the floor. Every apparent sophistication in the category is marketing language over a single-shot prompt. Genuine goal inference and per-match state would be a real, defensible product difference — not a marginal one.

**The sobering one, which matters more:** *a tone dropdown is sufficient to build a $15M business.* The market has already revealed that users will pay $7/week for one stuffed prompt with a style selector. That is strong evidence that the ceiling on this feature set is high and the floor on required sophistication is low — which means **added sophistication has to earn its keep on retention and handoff rate, not on being obviously better engineering.** It is entirely possible to build a far better system and lose to Rizz on distribution.

## 3. What nobody does

Across the whole category, absent:

- **Goal inference of any kind.** No intent model, no elicitation, no questions asked before answering.
- **Reciprocity assessment** — the highest-leverage variable per `AGENT-HARNESS.md` §4, and it's free from screenshot metadata. Nobody computes it.
- **Cross-session subject memory.** No exceptions — Wingman.live markets it and does not have it (§1.5).
- **Any escalation path.** No product in the category can say "this one needs a person," because none has a person.
- **Outcome capture.** Nobody asks whether the suggested message actually worked, so nobody can learn from it or prove it works.

That last one is striking. The entire category ships advice and never measures whether it landed.

---

# PART II — IS STUFFING ACTUALLY BETTER?

## 4. The honest case for stuffing

It deserves a fair hearing, because it is stronger than it was two years ago:

- **Retrieval infrastructure is real cost and real failure modes.** A vector DB over a 20k-token corpus is unnecessary infrastructure; the 2026 consensus is that "for the long tail of developers building internal tools, support bots, and document Q&A over modest corpora, the vector database is unnecessary."
- **Prompt caching removed most of the cost objection.** 90% off cached reads (`AGENT-HARNESS.md` §15) means a large stable prefix is cheap to re-send.
- **Long context wins on some tasks.** Long-context models "match or beat RAG on most benchmarks when the data fits in the window."
- **It's simpler**, and simpler systems ship and stay debuggable.

So the intuition is not naive. It's just wrong past a threshold that is much lower than most people assume.

## 5. The evidence against

Chroma tested **18 frontier models** — GPT-4.1, Claude 4 Opus and Sonnet, Gemini 2.5 Pro and Flash, Qwen3 — on extended needle-in-a-haystack tasks ([Chroma](https://www.trychroma.com/research/context-rot)):

| Finding | Detail |
|---|---|
| **Universality** | **Every single one of the 18 models gets worse as input length increases** — including on trivial tasks like retrieval and text replication |
| **Magnitude** | 20–50% accuracy drops from 10k to 100k+ tokens on NIAH |
| **Shape** | Degradation is **non-uniform — models hit cliffs**, not a predictable linear slope |
| **Position** | Lost-in-the-middle is a *separate* effect: U-shaped, **20–30 points lower** for evidence in the middle |
| **Cost** | RAG is reported **~1,250× cheaper per query** at scale |

The 2026 consensus architecture is explicitly hybrid: *"retrieve 50K–200K relevant tokens, then long-context-reason over them. A 1M-token window does not reliably reason across 1M tokens."*

One frequently-cited figure — reasoning accuracy falling from **0.92 to 0.68 as inputs grew from a few hundred to ~3,000 tokens** — should be treated carefully, since it's task-specific and appears in secondary sources. But even discounting it heavily, the direction is unambiguous and the onset is far earlier than "we have a 1M window, just put everything in."

## 6. The finding that matters most for this domain

Buried in the Chroma results is the one that should decide Wing's architecture:

> **Semantic similarity drives decay more than length does.** When the needle is semantically *distinct* from the haystack, models find it fine. When distractors are semantically *similar* to the answer, accuracy drops sharply — and the drop worsens with length.

Now consider what a stuffed dating context actually contains:

```
"…she said she's free Thursday…"          ← this match, current
"…we talked about getting coffee…"         ← this match, three weeks ago
"…suggested coffee Thursday, she said…"    ← a DIFFERENT match
"…advised him to propose a specific day…"  ← prior coach advice, another thread
"…she mentioned her dog at the park…"      ← this match
"…her dog came up again…"                  ← a different match's dog
```

**A dating context is close to the worst case the research describes.** Every fragment is semantically near-identical to every other fragment: plans to meet, references to pets and jobs, expressions of interest, scheduling. There are no lexical anchors and no topical separation. Distinguishing "the coffee plan with Priya" from "the coffee plan with Jamie" is precisely the semantically-similar-distractor condition that produces the sharp accuracy drops.

**So "stuff everything in" is not merely generically suboptimal here — it is bad in the specific way this domain is structured.** And the failure is silent: the model won't error, it will confidently attribute Jamie's dog to Priya. That is the single most damaging failure a dating assistant can have, because it destroys the one thing the product is selling — that someone was paying attention.

## 7. But retrieval is also the wrong answer

Having argued against stuffing, the opposite over-correction is worse.

Wing's entire relevant corpus for one request is **a few thousand tokens.** Retrieval exists to select from corpora too large to fit. There is nothing to select from here. Building embeddings, a vector store, and a retrieval pipeline over 5,000 tokens adds latency, an extra failure mode, a dependency `ARCHITECTURE.md` §2.1 explicitly prohibits, and no accuracy — RAG's advantage appears "as repository size and complexity increase," and this repository does neither.

---

# PART III — WING'S ANSWER

## 8. Strict scoping, then distillation

The right architecture is neither. **Scope hard, keep it small, and never let two matches share a context window.**

The pleasing part: **the product design and the engineering answer are the same decision.** A thread per match isn't only a UX choice — it is the context-management strategy, because it makes cross-match contamination structurally impossible rather than something the model has to get right.

**Per-request budget:**

| Component | Budget | Cached? |
|---|---|---|
| System prompt + tool definitions | ~2,000 | **Yes** — stable prefix |
| `you/` distilled memory (the client) | ~1,000 | **Yes** — changes rarely |
| **This match's** state | 800 | No — appended |
| Screenshot facts, structured | 300 | No |
| Recent turns in this thread | 800 | No |
| **Total** | **~4,900** | ~60% cached |

**Under 5k tokens per request**, which sits below where NIAH degradation becomes measurable and far below the cliffs. And roughly 60% of it is a stable prefix, so caching applies where it pays.

**Three hard rules:**

1. **One match per context. Ever.** No request ever contains two matches' histories. This is the rule that avoids §6 entirely, and it should be enforced by the query layer, not by prompt instruction.
2. **Distil, never truncate.** When `you/` exceeds its budget, compress it — structured distillation reports ~11× token reduction with retrieval preserved (`AI-LANDSCAPE.md` §20). Truncating the middle is the worst option available, since the middle is already where lost-in-the-middle costs 20–30 points.
3. **Budget is a hard cap with a test.** Assert the token count in CI. A budget that isn't asserted becomes a suggestion, and context creep is the default direction of every agent codebase.

## 9. What to do when it doesn't fit

Ordered by preference:

1. **Distil `you/`.** It's the largest compressible component and the most repetitive.
2. **Decay match state.** Old advice on a match that's been quiet for weeks has near-zero value — `matches.state` already auto-archives at 30 days idle (`MATCH-THREADS-DESIGN.md` §5).
3. **Summarise older turns**, keeping the most recent verbatim. Recency matters most and the tail is where degradation bites.
4. **Only then consider retrieval** — and if you get here, the memory design is wrong, not the retrieval strategy.

## 10. Where this leaves the competitive picture

| | Category norm | Wingman.live (tested) | **Wing (proposed)** |
|---|---|---|---|
| Within-request context | One stuffed prompt | Same | Scoped per match, budgeted, distilled |
| Conversation memory | Sometimes, as an archive | **Yes** — assistant chat only | Yes |
| **Subject memory** | **None** | **None** — screenshot facts discarded | **Structured facts persisted per match** |
| Goal model | **Tone dropdown** | Unclear | Staged belief, EVOI-elicited |
| Reciprocity read | None | None | Inferred from screenshot metadata |
| Trend and gap detection | **Impossible** | **Impossible** | Gap flagging, Insights |
| Outcome capture | **None** | None | Per-answer, links advice to result |

**The single row that matters is subject memory**, because the four rows beneath it are all downstream of it. Reciprocity trends, gap flagging, and outcome linking are not separate features — they are what becomes *possible* once screenshot facts are persisted against a match instead of discarded after generating a reply.

That reframes the differentiation more usefully than "we have memory and they don't." The whole category **extracts signal from a screenshot, uses it once, and throws it away.** Everything Wing intends to do that is distinctive follows from keeping it.

§2's sober conclusion still stands — **Rizz built a $15M business on the leftmost column**, so this is necessary rather than sufficient. But note what changed: the earlier version of this table said the copyable rows were "a few weeks of engineering for anyone who decides to do it." That remains true of the *mechanism*. What compounds is the **accumulated data**, and a competitor who starts later starts empty.

## 11. Summary

**On the state of the art:** the category is one stuffed prompt plus a tone dropdown. The market leader's entire goal model is *funny / sweet / bold / romantic / savage*. One minor player claims per-match memory. Nobody infers goals, computes reciprocity, captures outcomes, or can escalate to a human.

**On stuffing:** no, and it fails earlier than expected — all 18 frontier models tested degrade with length, non-uniformly, hitting cliffs. Critically, **semantic similarity drives decay more than length**, and dating context is unusually self-similar, so the failure mode here is confidently attributing one match's details to another. Silent, and fatal to the product's core promise.

**On the architecture:** scope per match, budget to ~5k tokens, distil rather than truncate, cache the stable ~60%. Retrieval is unnecessary infrastructure at this corpus size. **The thread-per-match design is the context strategy** — it makes the dangerous failure structurally impossible instead of something the model must avoid.

**And the caution worth keeping:** better engineering is not a moat here. A $15M competitor proves the floor is low. The two rows that compound — outcome data and human escalation — are the ones worth being early on.
