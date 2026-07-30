# Wing — Match Threads, Job Types, and the Handoff

**July 2026 · Design assessment of the two-thread architecture**

## 0. The proposal, restated

- A **Matches** surface where each match has its own thread, **AI-maintained**, which the user talks to.
- **Job types** as first-class units of work — photo rating, bio review, opener, convo rescue, screening — each with **its own UI, context schema and behaviour**, not just its own price.
- A **Coaches** surface where human threads live, as today.
- A **handoff**: any match thread can be sent sideways to a coach for real help. The match then exists in both places — maintained by the AI, visible to the coach.

**Verdict: this is the strongest version of the product proposed so far, and it should be adopted — with one hard constraint and one economic correction.** It solves the problem none of the previous designs solved, which is acquisition. The constraint is that the AI must never produce send-ready text (§3). The correction is that the AI thread cannot be free (§8).

---

## 1. Why this is stronger than everything before it

Every previous version of the plan had the same unresolved hole. `MARKET-ANALYSIS.md` §3 established that Wing cannot buy customers — a $4 answer nets cents, so paid acquisition is arithmetically closed. `GO-TO-MARKET.md` §15 concluded the business is therefore "an audience-aggregation play disguised as a marketplace": every customer has to arrive through a coach who already had them. And `AI-LANDSCAPE.md` §2 found that the channel this depends on is already dominated by a $15M competitor using it at scale.

So the plan's growth model was: recruit creators, hope they bring audiences, compete with Rizz for their attention. That is a real strategy and a fragile one.

**The AI match thread is the first proposed mechanism that acquires users without a coach.** It is self-serve, always-on, useful on day one with zero supply, and owned rather than rented. That changes the shape of the business from "aggregate other people's audiences" to "own a funnel and monetise it with humans."

Three further things it fixes, each of which was a named problem in earlier documents:

| Problem | Where it was raised | How this fixes it |
|---|---|---|
| Coaches spend their time on reconstruction, not judgment, which caps quality at sub-$20/hr labour | `MARKET-ANALYSIS.md` §34, `AI-LANDSCAPE.md` §9 | **The handoff *is* the briefing.** Context arrives as the order. No separate briefing system needed. |
| The purchase decision happens cold, on a profile page, with no context | `UI-DESIGN.md` §3 | The purchase happens at a **specific stuck point in a specific thread** — the highest-intent moment the product can manufacture. |
| "Saved" exists because advice had nowhere to live | `UI-DESIGN.md` §6 | Advice lives on the match it was about. Saved's job is done better by the thread. |

## 2. The metric this design lives or dies on

Borrowed from customer support, then inverted.

In CX, human escalation is a failure: **target handoff is 15–30%**, below 10% means the bot isn't escalating when it should, and **"nuanced complaints rarely break 25%" deflection** ([HappySupport](https://happysupport.ai/blog/support-ticket-deflection-rate-benchmarks), [Agentkit](https://agentkit.ai/blog/chatbot-kpis-metrics)).

In Wing, **the handoff is the revenue.** The AI thread's job is not containment. It is retention, qualification, and context assembly — and then getting out of the way.

That inverts the tuning target and gives the product a single health metric:

> **Handoff rate.** Too low and the AI is either useless (nobody stays) or too good (nobody pays). Too high and the AI isn't earning its inference cost.

And note where dating sits on the CX curve: nuanced, subjective, emotionally loaded — the exact category that "rarely breaks 25%" deflection. **The natural escalation rate for this problem class is high, which is precisely what this business wants.**

**This makes bet 1 more load-bearing, not less.** `MARKET-ANALYSIS.md` §35 bet that human judgment beats frontier AI on real threads. This design depends on a *narrow band*: the AI must be good enough that people stay, and not so good that nobody hands off. If AI ≈ human on these threads, the handoff never happens and there is no marketplace — just another AI app. Run bet 1 first, unchanged.

---

## 3. The one constraint: the AI never produces send-ready text

This is the line, and crossing it collapses the product into rizz app #21.

**Hinge already drew it, deliberately, with far more resources.** Prompt Feedback "doesn't tell the dater exactly what to say, or provide suggested language" — it coaches through editing, guided by their PhD behavioural scientists, with three tiers (*Great Answer* / *Try a Small Change* / *Go a Little Deeper*). Commentary noted it is "distinct from AI services that write the message for you" ([Hinge](https://hinge.co/newsroom/prompt-feedback), [DatingNews](https://www.datingnews.com/apps-and-sites/hinge-releases-prompt-feedback-ai-tool/)).

If Hinge — with every commercial incentive to ship a generator to 2M payers — chose coaching over prescribing to protect authenticity, that is the strongest available precedent for the same call here. Add the brand arithmetic: ~60% of daters believe they have received AI-written messages and roughly 80% call it a dealbreaker. A product that generates the message is on the wrong side of the norm it should be selling against.

So, precisely:

| The match AI **does** | The match AI **never does** |
|---|---|
| Maintain state: who, what was tried, what happened, when | Write a message to send |
| Ask clarifying questions before anything else | Offer a copy-paste block |
| Name patterns: *"that's three questions in a row"* | Rewrite the user's draft into its own words |
| Structure the situation so it's legible | Speak as the user |
| Say *"this one needs a person"* and offer the handoff | Claim to be a coach |
| Help the user find **their own** words | Assess the match's personality or looks |

The user's phrase was that the AI "formats response." Read as **formatting the situation** — structuring what's happening so both the user and later a coach can see it — that is exactly right and is the whole value. Read as *formatting a reply to send*, it is the one thing that must not ship.

**Test for any proposed AI feature:** if the output could be pasted into a dating app, a human wrote it or it doesn't exist.

---

## 4. Information architecture

Four tabs, same count as today. **Saved dissolves into Matches**, because that is where advice belongs.

```
┌──────────┬──────────┬──────────┬──────────┐
│ Matches  │ Coaches  │ Discover │   You    │
│    ◇     │    ☺     │    ⌕     │    ⊙     │
└──────────┴──────────┴──────────┴──────────┘
```

| Tab | Answers | Threads inside | Maintained by |
|---|---|---|---|
| **Matches** | What's going on with each person? | `My profile` (pinned) + one per match | **AI** |
| **Coaches** | Who am I working with, what do I owe them? | One per coach — today's Home | **Human** |
| **Discover** | Who can help with this? | — | urgency-first (unchanged) |
| **You** | Everything about me | — | — |

### Matches tab

```
┌─────────────────────────────────────┐
│  Matches                         ⚙  │
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐  │
│  │ ◈ My profile                  │  │  ← pinned, always first
│  │   Bio · 4 photos · 2 jobs   ▸ │  │     (the artifact scope)
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  NEEDS YOU · 1                      │
│  ┌───────────────────────────────┐  │
│  │ ✓ Maya answered about Priya   │  │  ← handoff came back
│  │   Convo rescue · 2m ago     ▸ │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  ACTIVE                             │
│  ┌───────────────────────────────┐  │
│  │ P  Priya                      │  │
│  │    Hinge · waiting on you 2d  │  │  ← state, not a summary of her
│  │    ⚑ with Maya R.          ▸  │  │  ← handed off; lives in both tabs
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ J  Jamie                      │  │
│  │    Hinge · gone quiet 5d    ▸ │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  ARCHIVED · 6                    ⌄  │  ← auto-archives at 30d idle
└─────────────────────────────────────┘

        [ + Add a match ]
```

**`My profile` pinned first is doing real work.** It is where bio, photos and profile jobs live — the `artifact` scope from `AI-LANDSCAPE.md` §22 — and it is the only entry that exists before the user has a single match. That makes the tab useful on install rather than an empty list.

### The handoff, in the match thread

```
│  ── AI thread with Priya ──         │
│                                     │
│  ⟨AI⟩ Two things I can see: the gap │
│  is 2 days, and your last three     │
│  messages all ended in questions.   │
│                                     │
│  ⟨AI⟩ I can keep helping you think  │
│  this through. But you've asked me  │
│  the same thing three ways — that   │
│  usually means you want someone to  │
│  just tell you. Want a person?      │
│                                     │
│  ┌───────────────────────────────┐  │
│  │  ⚑ Send this to a coach       │  │  ← the purchase moment
│  │  They'll see the whole thread │  │
│  │  Maya R. · ~5 min · $6        │  │
│  └───────────────────────────────┘  │
```

The AI offering the handoff **when it recognises its own limit** is the most trust-building moment available to this product, and no incumbent will build it — Grindr and Match have no human layer to escalate to, so their AI must always answer.

---

## 5. Data model

Additive. Nothing existing changes shape.

```sql
-- The match: a thin label, never a profile (ARCHITECTURE.md §13.1)
create table matches (
  id uuid primary key default gen_random_uuid(),
  client_id uuid not null references profiles(id),
  label text not null,                    -- first name or nickname, user-supplied
  platform text,                          -- 'hinge' | 'bumble' | 'tinder' | other
  state text not null default 'active',   -- active | archived
  last_activity_at timestamptz not null default now(),
  archived_at timestamptz
);

-- One AI thread per match, plus one per artifact scope
create type ai_thread_scope as enum ('match','profile');
create table ai_threads (
  id uuid primary key default gen_random_uuid(),
  client_id uuid not null references profiles(id),
  scope ai_thread_scope not null,
  match_id uuid references matches(id),   -- null when scope = 'profile'
  memory jsonb not null default '{}',      -- capped; see §7
  tokens_spent_cents int not null default 0,
  created_at timestamptz not null default now()
);

-- The handoff: attaches a match's context to a paid coach engagement
create table handoffs (
  id uuid primary key default gen_random_uuid(),
  match_id uuid not null references matches(id),
  engagement_id uuid not null references engagements(id),
  job_type text not null,                  -- see §6
  context_snapshot jsonb not null,         -- what the coach sees, frozen at purchase
  created_at timestamptz not null default now()
);
create index on handoffs (engagement_id);
```

Three deliberate choices:

- **`context_snapshot` is frozen at purchase.** The coach is paid to answer a specific situation; if the AI thread keeps moving underneath them, the terms of the job change after the fact. This is the same reasoning as snapshotting `pricing.quote()` onto the engagement (§2.4).
- **Coach threads stay coach-scoped.** The relationship and the money belong to the coach, not the match. A match attaches to a coach thread as context; it does not become its own coach thread.
- **`matches.label` is user-supplied and free-text.** Wing never derives a name, never enriches, never looks anyone up.

---

## 6. Job types

This is the part of the proposal that is most clearly right and most cheaply built, because Wing already has typed products — what is missing is that the **UI differs per type.** A photo rating rendered as a chat is a bad photo rating.

| Job type | Scope | Its own UI is | AI thread can | Human job is |
|---|---|---|---|---|
| **Photo rating** | profile | A grid: each photo with keep / cut / reshoot and one reason | Order them, flag obvious problems, ask what they're going for | The verdict, with reasons — see below |
| **Bio review** | profile | A diff: current vs proposed, version history | Ask what they do, surface what's vague | Rewrite in their voice |
| **Opener** | match | Screenshot + the profile detail being used | Point at what's usable in her profile | Write it |
| **Convo rescue** | match | The thread, with the stall marked | Name the pattern, timeline the gap | The next move, or **The Call** |
| **Screening** | match | Signal list: green / amber / red with evidence | Assemble the signals | Is he worth the Thursday |
| **Date plan** | match | Itinerary with times and fallbacks | Ask constraints, budget, city | The plan |

**Photo rating deserves specific attention, because the naive version is already free.** Photofeeler gives human trait ratings free via reciprocal voting, and its own **Photofeeler-D3 neural net is "as accurate as 10 human votes"** on attractiveness/trust/smart ([ResearchGate](https://www.researchgate.net/publication/332463196_Photofeeler-D3_A_Neural_Network_with_Voter_Modeling_for_Dating_Photo_Rating), [Roast](https://roast.dating/blog/photofeeler-review)). So a *score* is a commodity — free, and machine-replicable.

What isn't commoditised is the judgment, and Photofeeler's own limitations say why: scores are "often too subjective to be truly useful," and "a high score tells you the photo landed well on a first pass, but it doesn't tell you whether the whole profile closes the deal." **So the job type is not "rate my photos." It is "which of these six, in what order, and what's missing" — a lineup decision, which is what Sam T.'s existing offering already is.** Never ship a numeric score; it invites comparison against a free product that does it better.

---

## 7. The economic correction: the AI thread cannot be free

This is the part of the proposal that needs changing, and the arithmetic is unambiguous.

Freemium conversion benchmarks: **general freemium 2–5%**, B2B median 2.6%, **consumer apps lowest at ~2.1%**; AI-native "good" is 6–8%. Free *trials* convert far better at **5–20%** ([Product Growth](https://www.productgrowth.blog/calculators/freemium), [Prems](https://prems.ai/blog/free-to-paid-conversion-saas-2026)).

Take a generous 4% handoff-to-purchase rate and work backwards from the plan's own $10k/month target:

| | Figure |
|---|---|
| Platform net target | **$10,000/mo** |
| At 18% blended take, $25 AOV | ~2,200 paid handoffs/mo |
| At 4% conversion | **55,000 free AI users** |
| Inference at $0.30/user/mo | **−$16,500/mo** |
| **Net** | **−$6,600/mo** |

**A free AI tier loses money at the scale required to make the marketplace work**, and it loses it faster the better the AI thread is at retaining people who will never hand off. This is the `AI-LANDSCAPE.md` §20 cost curve arriving through a different door.

Three fixes, in order of preference:

1. **Included with any purchase; hard-capped for everyone else.** Free users get a bounded trial — say two match threads and twenty exchanges — which converts on *free-trial* economics (5–20%) rather than freemium (2–5%). Anyone who has ever bought anything gets the AI thread unlimited, which also makes the first purchase more attractive.
2. **Cheap paid tier, $5–8/month.** Covers inference with margin, filters non-buyers, and a user who has already paid $5 is dramatically more likely to pay $25 for a human. This also sidesteps the freemium trap entirely.
3. **Cap inference per thread from day one** — retrieval and distillation, not accumulate-and-inject (`AI-LANDSCAPE.md` §20). `ai_threads.tokens_spent_cents` exists in the schema above so this is measurable per user from the first day rather than discovered in month twelve.

Do **1 and 3** at launch. Hold 2 in reserve if trial conversion disappoints.

---

## 8. What this changes in the existing specs

Being explicit, because it moves a line I wrote three commits ago.

**`ARCHITECTURE.md` §13.1 needs amending — genuinely, not cosmetically.** I previously drew the AI line at "coach-side context assembly only; nothing client-facing." This design puts AI in a client-facing chat, which is past that line. The amended line is **narrower but better**, because it targets the actual risk rather than the surface:

| Old line | New line |
|---|---|
| AI is coach-side only | **AI never produces text that could be pasted into a dating app** |
| Protects: the trust brand | Protects: the trust brand, *and* permits the acquisition funnel |
| Precedent: none | Precedent: Hinge Prompt Feedback |

The reason the new line is defensible is that the risk was never "AI touches the client." It was "a dater receives machine-written words believing they're human." A thread that structures a situation and refuses to write the message does not create that risk. A generator does, whoever it is facing.

**Also changing:**
- `Saved` tab is removed; `saved_items` becomes match-scoped. (`UI-DESIGN.md` §6, `ARCHITECTURE.md` §3.)
- Client tabs become Matches · Coaches · Discover · You.
- `AI-LANDSCAPE.md` §22 typing survives intact and is now enforced by `ai_threads.scope` — `profile` accumulates freely, `match` is capped and decays via `matches.state` auto-archiving at 30 days idle.
- The §12/§23 "coach briefing" proposal is **superseded**: the briefing is `handoffs.context_snapshot`, delivered with the order. Simpler, and it arrives at the moment it is needed.

---

## 9. Scope, honestly

This is roughly a **40–60% larger build** than the specified marketplace, and it adds a second product with its own UI, its own cost model and its own failure modes. Two things follow.

**It cannot be M1, and it must not delay bet 1.** The whole design assumes a narrow band where AI retains but doesn't satisfy (§2). If bet 1 shows AI answers match human ones on real threads, this architecture produces an AI app with a vestigial marketplace attached — which is option A from `AI-LANDSCAPE.md` §15, scoring 12 out of 30. **Run the 30-thread blind test before building any of this.**

**Suggested sequencing**, replacing nothing in M0–M2:

| Stage | Build |
|---|---|
| **Now** | Bet 1: blind human-vs-AI on 30 real threads |
| **M0–M2** | Unchanged. Marketplace, coach threads, metering, delivery |
| **M2.5** | `matches` + `handoffs`, **no AI at all** — matches as user-created labels, manual context, "attach this match" on purchase. Tests whether match-scoping helps *before* paying for inference |
| **M3** | Reputation, queue, recourse (unchanged) |
| **M4** | The AI match thread, capped trial, one job type only (convo rescue), retrieval from day one — harness spec in **`AGENT-HARNESS.md`** |
| **M5** | Remaining job types with typed UI |

**Two pieces of the agent can be tested at M2.5 with no AI at all** (`AGENT-HARNESS.md` §19): the goal schema is ordinary code, and `ask_choice` with hand-written questions on a hand-written decision tree can be trialled with **human coaches answering**. That establishes whether structured goal elicitation actually improves advice before any inference machinery is built.

**M2.5 is the cheap experiment inside the expensive idea.** Match-scoping, the handoff, and typed job UI deliver most of the structural benefit — better purchase context, advice that lives somewhere, coaches who arrive briefed — with **zero inference cost and no AI risk**. If match-scoping doesn't lift conversion or repeat purchase with humans doing the work, the AI layer would not have saved it.

## 10. What I would cut from the proposal

- **Any AI-authored message text.** §3. Non-negotiable, and the reason the rest is viable.
- **Numeric photo scores.** §6. Free elsewhere and beaten by a neural net.
- **A free unlimited AI tier.** §7. Capped trial instead.
- **AI in more than one job type at launch.** Convo rescue only. It is the highest-frequency, most deadline-bearing job (`AI-LANDSCAPE.md` §22, urgency-first), so it is where the funnel logic gets tested honestly.
- **Match threads that describe the person.** State and history only. The moment a match thread reads like a profile of her, it is a dossier on a non-consenting third party and §21's balancing test applies.

---

## 11. What this does to go-to-market

This is the largest strategic consequence of the design, and it is worth stating separately because it invalidates the central thesis of `GO-TO-MARKET.md` §15.

### 11.1 Three things it genuinely fixes

**The cold start is gone.** A marketplace with no coaches is worthless; an AI match thread with no coaches is useful on day one. That is the difference between launching a two-sided market and launching a product. Every prior version of the plan required solving supply and demand simultaneously with a $200 budget.

**Marketing becomes single-sided.** `GO-TO-MARKET.md` runs two parallel campaigns — Part II sells to six demand segments, Part III recruits six supply types, and §12 sequences them together. With a standalone AI product you market to users only, and recruit supply later against proven demand. That halves the GTM surface, and halving the GTM surface is worth more than any individual tactic in that document.

**Recruiting inverts from a favour into an offer.** This is the underrated one. Today the coach pitch is *"join my empty marketplace."* After a demand base exists it becomes *"there are 3,000 people here asking for help with conversations; you can start earning this week."* `COACH-RECRUITING.md` was built around cold outreach to creators for their audiences. Demand-first means you no longer need their audience at all — which removes the dependency on the single channel `AI-LANDSCAPE.md` §2 found is already dominated by a $15M competitor.

**Net effect:** the business stops being *"an audience-aggregation play disguised as a marketplace"* (`GO-TO-MARKET.md` §15) and becomes *a product with a human-services layer.* That is a materially better shape, and it is the right reason to adopt this design.

### 11.2 The correction: paid acquisition still does not close on take rate

Where the reasoning needs adjusting is the leap from "we only need user marketing" to "we can buy users." Performance marketing is a different claim from single-sided marketing, and the arithmetic does not support it *unless the AI tier is itself monetised.*

Benchmarks, all 2026:

| Input | Figure | Source |
|---|---|---|
| iOS CPI, Q1 2026 | **$5.84** (+19% YoY); Android $1.92 | [DigitalApplied](https://www.digitalapplied.com/blog/mobile-app-marketing-statistics-2026-install-data) |
| Dating vertical CPI | **$4.00–$15.00** | [Admiral Media](https://admiral.media/mobile-app-marketing-benchmarks-2026/) |
| Cost per **paying** user, general apps | **$20–$80** | [Insert Affiliate](https://insertaffiliate.com/blog/mobile-app-user-acquisition-cost-benchmarks/) |
| TikTok CPI | $0.50–1.80 avg, target $1.50–5.00 | [TikAdTools](https://tikadtools.com/blog/tiktok-ads-app-install/) |
| Dating **engaged-user** CAC vs install cost | **up to 33× higher** | [Linkrunner](https://linkrunner.io/blog/metrics-that-matter-dating-community-edition) |
| Median Y1 RLTV per payer, North America | **$32** | [RevenueCat](https://www.revenuecat.com/state-of-subscription-apps) |
| Expected LTV:CAC for subscription apps | **4:1 to 5:1** | [Foundry CRO](https://foundrycro.com/blog/ltv-cac-ratio-benchmarks-2026/) |

Now run Wing's own numbers per acquired user:

| Monetisation model | Revenue per acquired user | Realistic CAC | **LTV:CAC** |
|---|---|---|---|
| **Free AI, handoffs only** — 4% convert, $25 AOV, 18% take, 2 lifetime purchases | **$0.36** | $1.50–5.84 | **0.06–0.24 : 1** |
| Same, wildly optimistic — 15% convert, 3 purchases | **$2.03** | $1.50–5.84 | **0.35–1.35 : 1** |
| **Paid AI at $8/mo** — 8.2-month average life at AI-app churn, less inference, plus handoffs | **~$61 per payer** | $20–80 per payer | **0.76–3.05 : 1** |
| **Organic / content-led** — either model | as above | **~$0** | — |

Two conclusions, and the first is uncomfortable.

**On marketplace take rate alone, paid acquisition is arithmetically closed — again.** This is `MARKET-ANALYSIS.md` §3 arriving through a new door. The problem is that an 18% take on a $25 transaction leaves **$4.50**, and you cannot buy a customer for $4.50 in a vertical where CPI is $4–15 because you are bidding against Match Group and Bumble, whose RPP is $17–33. Adding a free AI tier does not fix this; it makes it worse, because the free tier has a cost per user and most of those users never hand off (§7).

**The AI subscription is the only revenue line per user large enough to fund paid acquisition.** At $8/month it produces ~$61 per payer against $4.50 for a marketplace transaction — a 13× difference — because subscription revenue is 100% yours while marketplace revenue is 18% of someone else's. That takes paid acquisition from *impossible* (0.24:1) to *marginal* (0.76–3.05:1), still short of the 4:1 investors expect but inside the range where careful targeting could work.

So the accurate version of the strategic claim is:

> **Single-sided marketing: yes, and it's the biggest win in this design. Paid single-sided marketing: only if the AI tier is a paid subscription, and even then it is marginal rather than proven.**

This strengthens the §7 recommendation considerably. The AI tier being paid is not just about covering inference — **it is the difference between having a viable acquisition channel and not having one.**

### 11.3 Two things that help, both already decided

**Web-first is an acquisition advantage, not just a compliance constraint.** `ARCHITECTURE.md` §13.1 makes Wing a PWA because Apple's guideline 3.1.3(d) makes native async answers loss-making. The side effect is that Wing never pays the $5.84 iOS CPI, never enters the App Store install auction against Match Group, and never surrenders 30%. A constraint that looked purely defensive turns out to lower the cost of the one channel this design depends on.

**Organic remains the channel, and `MARKETING-PLAN.md` was already right about this.** That document treats paid as "a measuring instrument, not a channel, until LTV is proven." Nothing here changes that — it sharpens *what* has to be proven. The number to establish before spending on ads is **subscription retention**, because that is the entire LTV, and AI apps churn ~30% faster than others (21.1% vs 30.7% annual). If Wing's AI tier retains at AI-app benchmark, paid is marginal. If it retains at 30%+, paid opens up.

### 11.4 What to change in the other documents

- **`GO-TO-MARKET.md` §15** — the audience-aggregation thesis is superseded *conditionally*. Add the amendment rather than rewriting: the thesis held while the marketplace was the only product; a monetised AI tier changes the shape to product-plus-services.
- **`GO-TO-MARKET.md` §12** — sequencing can collapse from "supply and demand in parallel per segment" to demand-first, supply recruited against proven demand.
- **`COACH-RECRUITING.md`** — the pitch changes from *audience-for-monetisation* to *clients-are-already-here*. The 10% coach-sourced rate stays, but it stops being the primary hook.
- **`MARKETING-PLAN.md`** — unchanged in policy; add subscription retention as the metric that gates any paid spend.

---

## 12. Summary

**Adopt it.** The two-thread architecture with typed job UI and a coach handoff is the first design in this project that answers the acquisition question, and it makes several previously-separate problems disappear at once — the cold purchase decision, the coach briefing, and where saved advice lives. Its largest effect is on go-to-market: **the product works with zero coaches, so marketing goes single-sided and supply gets recruited against proven demand** (§11).

**Three conditions.** The AI never writes anything sendable (§3); the AI tier is **paid or hard-capped, never free** (§7, §11.2); and match memory stays state rather than portraiture (§10).

The middle condition carries more weight than it first appears. It began as a way to cover inference cost, but §11.2 shows it is also the only revenue line per user large enough to fund acquisition at all — ~$61 per payer against $4.50 per marketplace transaction. **A free AI tier does not just leak money; it forecloses the channel the whole design exists to open.**

**And the ordering doesn't change.** This design needs the AI to be genuinely useful and genuinely insufficient. That is a narrow band, it is exactly what bet 1 measures, and it is 30 threads and a week of work to find out. Everything here is downstream of that answer.
