# Wing — Product Design Reassessment

**July 2026 · Auditing the built design against `MARKET-ANALYSIS.md`**

The market analysis produced fourteen findings with product implications. This document walks each one against what is actually specified and prototyped, and records the verdict. It exists because the useful output of a reassessment is not a list of changes — it is a list of *decisions*, including the ones that were to leave something alone.

**Result: five changes. Nine findings needed nothing.**

That ratio is the headline. The existing design already answered most of what the research surfaced, usually for the right reasons and occasionally by luck. Where it already answered correctly, the fix was to **write down the mechanism** so the decision survives a contributor who hasn't read the research.

---

> **⚑ PARKED — the product pivoted to a standalone agentic coach with no human marketplace. See `AGENT-PRODUCT.md`.**
> This document's research stands and is still cited elsewhere: market sizing, competitor pricing, regulatory findings, unit economics. What no longer applies is the assumption that coaches are the product. The human layer may return later as a premium verification tier rather than as the business.

## 1. The audit

| # | Finding | What the design already does | Verdict |
|---|---|---|---|
| 1 | Async answers on native iOS take Apple's 30% and lose money | Stack is Next.js on Vercel, mobile web; "native apps" already in §13 out-of-scope | **Cite it** → §13.1 |
| 2 | Micro-transactions: Stripe eats 10.4% of a $4 order | §4.2 already models processor fees and states the 10.5% figure | **No change** |
| 3 | The payments fix is basket aggregation | §4.8 already sells prepaid quantities (1/3/5) in one charge — but preselects **1** | **Change the default** → §4.9 |
| 4 | Sell a read and a verdict, not a reply | The answer card already separates reasoning from payload, reasoning first | **No change** |
| 5 | AI will never tell you to send nothing | The capability existed (`verdict` block type; a seeded "do nothing for 24 hours" answer) but rendered as a card with its payload *missing* | **Change** → new register |
| 6 | Urgency converts; open-ended browsing doesn't | Discover led with category chips (Openers/Bio/Photos) | **Change** → urgency-first |
| 7 | The defect-rate dataset is one of only two candidate moats | §9.7 already ranks on defect rate; reviews carry `outcome_tags` | **Partly** → per-answer capture |
| 8 | Never match or introduce anyone (NY GBL §394-c, CA Civ §1694) | "intros/matchmaking" already in §13 out-of-scope | **Cite it** → §13.1 |
| 9 | Never add AI to the answer path | "AI features of any kind" already in §13 out-of-scope | **Cite it** → §13.1 |
| 10 | Coaches are unlicensed, unprivileged, and subpoenable | §9.2 had the coaching line and a `report` path, but no scope boundary, no disclosure, no escalation | **Change** → §9.2 |
| 11 | Every screenshot contains a non-consenting third party | §9.1 had attachment TTL and self-serve deletion | **Change** → §9.2 commitments |
| 12 | Hinge is the only growing surface and is text-centric | Product is platform-agnostic; Hinge appears only as a filename | **No change** — see §3 |
| 13 | ROSCA / click-to-cancel enforcement continues | Subscriptions are enum-only, phase 2 | **No change** — gate recorded |
| 14 | Test $8–10 answers | Coaches already set their own prices; all five schemes support it | **No change** — it's an experiment |

---

## 2. The five changes

### 2.1 `PurchaseSheet` preselects the middle quantity

The single highest-value-per-character change in the codebase. One expression:

```js
state.pendingQty = o.qty ? o.qty[Math.min(1, o.qty.length - 1)] : 1;
```

| Purchase | Processor cost | Effective rate | Platform net/answer @ 80% | @ 90% coach-sourced |
|---|---|---|---|---|
| 1 × $4 | $0.42 | 10.4% | +$0.38 | **−$0.02** |
| 3 × $4 = $12 | $0.65 | 5.4% | +$0.58 | +$0.18 |
| 5 × $4 = $20 | $0.88 | 4.4% | +$0.62 | **+$0.22** |

The 90% column is the one that matters, because coach-sourced traffic is designed to dominate early. **At a default of one, rung 1 loses money on exactly the traffic the go-to-market plan is built to attract.** At a default of three it does not.

Not a dark pattern: per-answer price is equal or better at higher quantities, the single stays visible and one tap away, and unused answers refund at the à-la-carte rate (§4.6).

### 2.2 The Call — an answer that says send nothing

Maya's fourth seeded answer already read *"Do nothing for 24 hours. Seriously. The silence does more work than any message I could write here."* — with a null payload. It rendered as a normal answer card whose payload block was simply absent.

That is the most differentiating artefact in the product, and it looked like a bug.

It now has its own register: **positive spine and header (`THE CALL · ANSWER 4 OF 10`), a dashed `NOTHING TO SEND` block carrying the reasoning, and no Copy action** — because there is nothing to copy. The composer offers it explicitly as a third option.

Why this is worth a component rather than a CSS tweak: a tool built to generate a message will always generate a message. Telling a paying customer that the right move is to send nothing — and therefore not to use what they just paid for — is a judgment only a person with a reputation at stake will make. It is the demonstration of the entire differentiation thesis, so it should look deliberate. One operational note that matters more than the design: **holding must still consume one answer and must never be penalised.** If it costs a coach money or ranking, nobody holds, and the product quietly becomes a message generator with a human latency penalty.

### 2.3 Discover leads with urgency

`What's happening?` replaces `Who do you need?`. Five situations in the client's words — *They're waiting · Going quiet · New match · Date coming up · No deadline* — set the category filter and re-sort by response time. Category chips demote to secondary and clear the urgency selection when tapped.

The reasoning is the JustAnswer/Clarity.fm split. JustAnswer is the one per-question expert marketplace working at scale, and every category it serves carries a deadline and a consequence. Clarity.fm sold open-ended expert calls and shut down in 2022. Dating is high-emotion but low-urgency — nothing bad happens if you wait, ask a friend, or send nothing — so the interface has to go looking for the moments that do have a clock on them. `urgency_selected` is instrumented, so if deadline-bearing entries don't convert better, this reverts.

### 2.4 Per-answer outcome capture

`Did it land? (Replied) (No reply) (Didn't send)` — one tap on the answer card, shown only on answers with a payload and never on the newest message, because asking before the client has sent anything collects noise. New `answer_outcomes` table and `answer_outcome_recorded` event.

Two reasons this is product rather than analytics. First, `ARCHITECTURE.md` §9.7 ranks coaches on defect rate and VRIO named that dataset one of only two candidate moats — and reviews rate a whole *item*, not each piece of advice. Second, it is the only instrument that can answer the existential question: does a paid human answer actually beat a free machine one? Without it, bet 1 has no numerator.

`Didn't send` is explicitly not counted against the coach. Otherwise clients under-report it and the signal rots.

### 2.5 The coaching line gets teeth

§9.2 gains four things it was missing: an explicit **scope boundary** (coaching is not therapy), a **no-privilege disclosure** (coaches can be subpoenaed — stated in *Privacy & data* and coach ToS, in plain words), a required **`escalate(threadId, reason)`** path for disclosures involving self-harm, abuse or coercive control that freezes the item and opens an admin ticket rather than asking the coach to cope, and permanent **third-party-data commitments**: no facial analysis ever, no training on user content, no retention past TTL, manual mask/crop in the uploader.

All of it is cheap now and very expensive later. Utah's SB48 funds investigation of life coaches practising therapy; Bumble settled £32M over biometric consent.

---

## 3. Considered and rejected

The more useful half of the reassessment.

**A wallet or cross-coach credit balance.** The analysis recommended prepaid credit; §4.8 already had it, per-coach. A wallet would have added stored-value liability, escheatment exposure in several states, and breakage optics that sit badly with a trust brand — to capture a benefit a one-line default change already captures. **Rejected: the arithmetic justified a default, not a subsystem.**

**A "written by a human" badge or no-AI seal.** Tempting given 60% of daters believe they've been chatfished. Rejected on three grounds: it is unverifiable, so it is a promise the platform cannot enforce; it collapses catastrophically if any coach is ever caught using AI; and it competes on the axis where Wing is 20–40× overpriced per unit. The analysis is explicit that the anti-AI norm is a *marketing* asset, not a demand asset. **The Call register demonstrates humanity instead of asserting it**, which is the stronger move and needs no policing.

**Restructuring around Hinge.** Hinge is the only growing surface (+28% revenue, +15% payers) and its own research says text prompts beat photos by 47%. But hard-coding a platform is precisely what AskMatch did — Match-only human coaching, bundled, gone without trace — and cross-platform coverage is a real advantage over it. **Hinge-first belongs in marketing copy, coach specialisms and seed data, not in the data model.** No product change; the recommendation stands where it was made, in the go-to-market plan.

**Repricing answers to $8–10.** The labour ceiling says $5 answers can only buy sub-$20/hr labour. But coaches already set their own prices and every scheme supports any amount — the product needs nothing. This is a pricing experiment and a recruiting-guidance change, not a design change. **Left to the experiment deliberately**, so the result isn't confounded by an interface change shipped at the same time.

**De-emphasising price on coach cards.** `from $4/answer` invites exactly the per-unit comparison against $5/week unlimited AI that Wing loses. Hiding it was considered and rejected: price transparency is a trust property, and obscuring price to avoid comparison is the kind of move that erodes the only durable moat available. **The fix is in positioning copy — landing page, hooks, objection handling — not in the marketplace UI.**

**Face-blur tooling in v1.** The *policy* binds from M1. The *tool* stays M4: every phone already has a screenshot editor, and the commitments carry most of the risk reduction at none of the build cost.

**Anything touching a dating app's API.** Never on the roadmap, now recorded as permanent with the reason. Tinder's terms prohibit third-party services interacting with its Services or Member Content, "including artificial intelligence or machine learning systems." Screenshot-native is not a limitation to engineer around — it is the compliant architecture.

---

## 4. Still open

Not product-design gaps, but the things this revision leaves for someone else to do:

1. **Bet 1 is now instrumented but not run.** `answer_outcomes` makes the blind human-vs-AI comparison possible. Running it — 30 real threads, blind-scored on recipient reply rate — is an operational task for week 6, and the analysis argues it should precede further supply recruiting.
2. **The coach-side composer needs the third option built.** Specified in UI-DESIGN §4.6; the client-side register is prototyped, the coach's "Nothing to send — say why" input is not.
3. **`escalate()` needs a screen.** The behaviour is specified; the coach-facing flow and the fixed resource message are not designed.
4. **ROSCA compliance is a launch gate for subscriptions, not now.** When rung 3 ships, in-product one-click cancellation ships with it. Build to the vacated click-to-cancel rule anyway — the FTC restarted rulemaking in March 2026 and it is table stakes for a trust brand regardless of what the rule says.

---

## 5. What this reassessment concluded about the design overall

The design held up. Nine of fourteen findings needed no change, and the three that became citations were already correct decisions that simply had no stated reason — which is a documentation debt, not a design flaw.

The two changes that genuinely improve the product are the two smallest. Preselecting three answers instead of one moves rung 1 from loss-making to profitable on the traffic the business is built to attract. Giving *"send nothing"* its own visual register takes the clearest expression of the product's thesis and stops it looking like a rendering bug.

Both were already latent in the build. The research didn't reveal that the design was wrong; it revealed which parts of it were load-bearing.
