# Wing — Business Model & Go-To-Market Strategy

**v1 · July 2026 · The commercial and demographic strategy**

Companion to `MARKETING-PLAN.md` (the $200, six-week validation execution). This document is the layer above it: how the business actually makes money, who exactly we sell to on both sides, and which channel reaches each.

---

# PART I — THE BUSINESS MODEL

## 1. The problem with the obvious model

A flat 20% take on per-item purchases sounds clean until you run it:

| Item | Price | Stripe (2.9%+30¢) | Platform 20% | **Platform net** |
|---|---|---|---|---|
| One answer | $4 | $0.42 | $0.80 | **$0.38** |
| 5 openers | $18 | $0.82 | $3.60 | **$2.78** |
| Pack of 10 | $25 | $1.03 | $5.00 | **$3.97** |
| Bio makeover | $30 | $1.17 | $6.00 | **$4.83** |
| Live call | $50 | $1.75 | $10.00 | **$8.25** |

On a $4 answer, **payment processing eats 53% of your revenue**. The impulse purchase that makes the product marketable is the one that makes no money.

That's not a reason to abandon it. It's a reason to understand what it's *for*.

## 2. The model: a three-rung ladder

**The $4 answer is not the business. It is the customer acquisition cost you charge the customer.**

| Rung | Product | Price | Platform net | Its job |
|---|---|---|---|---|
| **1 · Impulse** | Single answer | $4–6 | ~$0.38 | Acquisition. Converts a stranger into a buyer at a price below their decision threshold. Effectively a paid trial that costs you nothing. |
| **2 · Commitment** | Packs, deliverables, calls | $18–50 | $2.78–8.25 | **The margin.** This is where the business actually earns. |
| **3 · Relationship** | Monthly retainer with a coach | $60–100/mo | ~$13/mo recurring | The LTV fix. Turns an episodic purchase into MRR. |

**The whole growth model is moving people up rungs.** Every metric that matters is a rung-transition rate: impulse→pack, pack→retainer.

> **Design consequence:** never optimise the $4 answer for margin. Optimise it for *conversion to rung 2*. The free-first-answer subsidy in the validation plan is the extreme version of this logic — deliberately paying to fill rung 1.

## 3. The clever part: take rate that follows who did the work

A flat take rate ignores the single most important fact about this marketplace: **you cannot afford to buy customers, so coaches must bring them.** The pricing should pay them for exactly that.

| Who sourced the client | Take rate | Why |
|---|---|---|
| **Coach brought them** (their link, their audience, their referral code) | **10%** | They did the acquisition. Charging them 20% for a client they delivered is theft, and they'll route around you. |
| **Platform brought them** (discovery, search, browse) | **25%** | You did the acquisition, and it cost you real money. |
| **Repeat purchase with the same coach** | **15%** | Neither of you re-acquired; you're providing infrastructure and trust. |

This does four things at once: it turns every coach into a distribution partner with a direct financial incentive, it makes the disintermediation temptation almost vanish at 10%, it lets you undercut Fiverr's effective ~20%+ for audience-owning creators, and it means your revenue is *highest* exactly where you added the most value.

**Blended expectation:** ~15–18% early (coach-led traffic dominates), rising toward 22% as marketplace discovery grows.

## 4. Secondary revenue, in the order to build it

1. **Coach Pro — $19/mo** *(month 6+)*. Take rate drops to 8% coach-sourced / 20% platform-sourced, plus analytics, priority placement, and custom booking links. High-volume coaches will do this arithmetic in seconds and subscribe. Converts your best supply into predictable revenue and locks them in.
2. **Client membership — $19/mo** *(month 9+)*. Three answers included, 15% off everything else, priority response. **Only ever offered after a third purchase** — offering it earlier converts a would-be $80 customer into a $19 one. This is the LTV instrument for the segments that keep dating.
3. **Pack breakage.** Unused answers past expiry, disclosed at purchase. Real margin, don't model it, don't design around it, and never make expiry aggressive — trust is the product.
4. **Placement, never sold.** Ranking is defect-rate-driven (see `ARCHITECTURE.md` §9.7). The moment ranking is purchasable the quality signal dies, and the quality signal is the entire moat. Off the table permanently.

## 5. What scale looks like

Roughly $4 net per transaction at a $25 blended AOV:

| Stage | Active coaches | Txns/mo | Platform net/mo |
|---|---|---|---|
| Validation | 5 | 40 | ~$160 |
| Early traction | 20 | 400 | ~$1,600 |
| Real business | 50 | 2,500 | ~$10,000 |
| With Pro + memberships | 50 | 2,500 | ~$14,000 |

**The unit of growth is a coach, not a customer.** Fifty productive coaches is a ~$10k/month business; the whole GTM below is therefore weighted toward supply.

---

# PART II — DEMAND: WHO WE SELL TO

Six real segments. **Do not market to all six.** Sequencing is in §12.

## Segment A — "Matches, no replies" · Men 25–35

The volume segment and the wedge.

| | |
|---|---|
| **Who** | On Hinge/Tinder/Bumble, gets matches, conversations die. Tech-literate, price-sensitive, quietly frustrated. |
| **Pain** | "She matched me and then just… stopped." Recurring, specific, and *diagnosable* — perfect for per-answer help. |
| **WTP** | $4–25. Low per purchase, **high frequency** — every new match is a new job. |
| **Best products** | Opener rescue, convo rescue packs, photo verdicts |
| **Frequency** | Weekly-to-monthly. The best repeat-purchase profile of any segment. |
| **Where** | Reddit (r/Tinder, r/hingeapp, r/dating_advice, r/Bumble), TikTok, YouTube Shorts, Discord |
| **Message** | *"Your matches aren't the problem. Your first message is."* Specific, blames the fixable thing, no shame. |
| **Objection** | "Why not just use ChatGPT?" → Answer with judgment, not text: *she's not replying because you asked two questions in a row after a three-day gap — an AI can't see that.* |
| **Careful** | Highest embarrassment sensitivity. Never use "loser", "desperate", or before/after mockery. |

## Segment B — "Starting over" · Men & women 38–55, divorced or long-single

The margin segment. Fewer people, far more money.

| | |
|---|---|
| **Who** | Out of a long relationship. Apps didn't exist last time they dated, or barely. Real income. |
| **Pain** | Not "how do I get replies" but "**I don't know how any of this works now, and I feel ridiculous.**" |
| **WTP** | **$45–150+.** The highest AOV in the market, and they want done-for-you, not tips. |
| **Best products** | Profile restart, bio makeover, photo strategy, live calls, monthly retainer |
| **Frequency** | Low but high-value; strong retainer conversion. |
| **Where** | Facebook groups (genuinely — this cohort lives there), r/datingoverthirty, r/dating_advice, r/divorce, podcasts, YouTube long-form |
| **Message** | *"Dating changed while you were away. Catch up without the cringe."* Dignity-first, zero pickup-artist vocabulary. |
| **Objection** | "Is this for young guys?" → Coaches who specialise in 40+ restarts, shown by name and age on the profile. |
| **Careful** | Sell competence and dignity. This segment leaves instantly at any whiff of manipulation. |

## Segment C — "Screening and standards" · Women 28–42

The retention and referral segment. Different product, same platform.

| | |
|---|---|
| **Who** | No shortage of matches; shortage of *worthwhile* ones. Exhausted by low-effort men, breadcrumbing, and wasted weeks. |
| **Pain** | "How do I stop wasting time on men who were never serious?" Not openers — **judgment**. |
| **WTP** | $6–60. Buys second opinions and strategy repeatedly. |
| **Best products** | Screening second opinion, conversation reads, date planning, safety-first strategy |
| **Frequency** | High. Every promising match is a decision worth $6. |
| **Where** | Instagram (Reels + Stories), TikTok, r/datingoverthirty, r/AskWomenOver30, group chats |
| **Message** | *"Send us the conversation. We'll tell you if he's worth your Thursday."* |
| **Objection** | Safety and privacy. Lead with data handling: screenshots auto-delete, coaches never touch accounts. |
| **Careful** | Never frame as "how to be more attractive to men." Frame as **filtering**, standards, and time. This segment refers friends more than any other — word of mouth is the channel. |

## Segment D — "Queer dating" · All ages

Underserved, high loyalty, small.

| | |
|---|---|
| **Who** | Queer daters getting hetero-default advice that doesn't apply. |
| **Pain** | Mainstream advice assumes scripts and dynamics that don't exist for them. |
| **WTP** | $5–40, and loyal once they find someone who gets it. |
| **Where** | TikTok, Instagram, Discord, r/actuallesbians, r/askgaybros, r/bisexual |
| **Message** | *"Dating advice that doesn't assume you're straight."* |
| **Careful** | Requires genuinely queer coaches, not straight coaches with a checkbox. Get this wrong publicly and you don't get a second chance. |

## Segment E — "Too busy" · Professionals 30–45

| | |
|---|---|
| **Pain** | Time, not skill. Wants outcomes per hour spent. |
| **WTP** | **$50–200.** Buys done-for-you deliverables and retainers without blinking. |
| **Where** | X/Twitter, LinkedIn, podcasts, newsletters |
| **Message** | *"Twenty minutes a week on dating, spent well."* |
| **Note** | Small volume, excellent economics; a natural retainer segment. |

## Segment F — "Anxious beginners" · 20–27

| | |
|---|---|
| **Pain** | Little experience, high shame, needs reassurance more than tactics. |
| **WTP** | **Low — $0–15.** |
| **Strategy** | **Serve with free content, do not spend to acquire.** They are the audience for teardowns and the source of virality, and they graduate into Segment A in two years. Monetising them now is bad business and slightly ugly. |

---

# PART III — SUPPLY: WHO WE RECRUIT

**A coach is worth ~50 customers.** Recruitment gets the same rigour as demand.

| Type | Where | Audience | Ease | Priority |
|---|---|---|---|---|
| **Creator-coaches** (TikTok/IG, 5k–50k) | TikTok, Instagram DMs | **High — this is distribution** | Medium | **1** |
| **Reddit power-answerers** | r/dating_advice, r/datingoverthirty top comments | Low | **Easy** | **2** |
| **Fiverr / Upwork sellers** | Existing dating gigs | Medium (bring clients) | Easy — pitch is fees + tooling | **3** |
| **Established coaches** ($1–3k packages) | Websites, podcasts, IG | High | Hard — must be sold on a low-ticket funnel, not a downgrade | 4 |
| **Adjacent skills** (copywriters, comedians, photographers) | X, IG, Reddit | Low | Medium — need client supply | 5 |
| **Therapists & matchmakers** | LinkedIn, directories | Medium | Slow — credentialing concerns | 6 |

### What each type actually wants (pitch accordingly)

- **Creator-coaches** want *monetisation without building a business*. They have audience and no product. Pitch: *"You keep 90% of what your own audience spends. No course to build, no funnel, no calls."* The 10% coach-sourced rate is the entire hook.
- **Reddit answerers** want *recognition and easy money*. They already do the work free. Pitch: *"You're doing this for free at 1am. Same thing, $5 an answer, from your phone."*
- **Fiverr sellers** want *better tools and lower fees*. Pitch: *metered follow-up conversations, screenshot-native chat, and repeat clients Fiverr can't give them.*
- **Established coaches** want *lead generation*. Pitch it as their **low-ticket top-of-funnel**: $4 answers introduce clients who later buy their $2,000 package. Never pitch it as a replacement.

---

# PART IV — CHANNELS

## 6. Channel scorecard

| Channel | Demand segments | Supply | Cost | Speed | Priority |
|---|---|---|---|---|---|
| **TikTok** | A, C, D, F | **Best source** | Free | Fast | **1** |
| **Reddit** | A, B, C, D | Good | Free (ads $0.50–1.85 CPC) | Fast | **2** |
| **Instagram** | C, D, B | **Best for DM recruiting** | Free | Medium | **3** |
| **YouTube Shorts** | A, B | Medium | Free | Slow, compounds | 4 |
| **Facebook Groups** | **B** (uniquely) | Low | Free | Medium | 5 |
| **X / Twitter** | E + founder narrative | Medium | Free | Medium | 6 |
| **Discord** | A, D, F | Medium | Free | Slow | 7 |
| **Podcasts** | B, E | Good | Free (guesting) | Slow | 8 |
| **LinkedIn** | E | Therapists | Free | Slow | 9 |

## 7. TikTok — the primary engine

Both sides live here, and the product's natural content format is already a proven genre.

- **Format that works:** the teardown. Screenshot on screen, coach's voice: *"He's asked three questions in a row. Watch what happens when we change one line."* 20–40 seconds.
- **Series that compound:** "Rate My Opener", "Dead Chat Rescue", "Bio Autopsy", "Would She Reply? (yes/no)".
- **Why it recruits coaches too:** every dating creator on TikTok is monetising badly. Your DM lands as an offer, not a pitch.
- **Cadence:** 1 video/day/coach, from *their* account, not a brand account. **Distributed authorship beats a brand handle** — people follow people here, and it means eight accounts posting, not one.
- **Cost:** $0. Never boost early; the algorithm rewards content, and boosting teaches you nothing about whether it's good.

## 8. Reddit — highest intent, strictest rules

- **Segment A:** r/Tinder, r/hingeapp, r/Bumble, r/dating_advice (4.8M), r/OnlineDating (~147k)
- **Segment B:** r/datingoverthirty, r/divorce, r/dating (6.2M)
- **Segment C:** r/AskWomenOver30, r/datingoverthirty
- **Segment D:** r/actuallesbians, r/askgaybros, r/bisexual

**Non-negotiable rules:** read each subreddit's policy first (many ban promotion outright), **message mods before a value thread** — several will approve a genuinely free offer, **always disclose affiliation**, never sockpuppet or fake reviews, and hold to 90/10 participation. One exposure destroys a trust product permanently, and deserves to.

**Paid:** cheapest CPC available and precise targeting, but see `MARKETING-PLAN.md` §1 — the arithmetic says use it as a measuring instrument, not a channel, until LTV is proven.

## 9. Instagram — recruiting and Segment C

- **Best coach-recruiting surface that exists.** Creators read DMs; the follower count is visible before you write.
- Reels mirror TikTok content at near-zero marginal cost.
- Stories with polls ("Would you send this opener?") are the highest-engagement format in the niche.
- Segment C over-indexes here versus TikTok.

## 10. The rest, briefly

- **YouTube Shorts** — same teardowns, third distribution, and they rank in search for years. Pure compounding.
- **Facebook Groups** — the only place Segment B congregates at scale. Divorce-support and over-40 dating groups. Participate for weeks before ever mentioning the product; these groups are protective and moderators are ruthless.
- **X/Twitter** — weak for direct acquisition, strong for the *"humans vs AI slop"* narrative, and it's where a technical co-founder and future investors read. Build the founder account, not a brand account.
- **Discord** — high trust, slow, doesn't scale. Good for early loyalists and coach community.
- **Podcasts** — guest on mid-size dating podcasts. Free, high-trust, ideal for Segments B and E.

---

# PART V — EXECUTION

## 11. Message per segment (use verbatim, then test)

| Segment | Hook | Product shown | CTA |
|---|---|---|---|
| A | *"Your matches aren't the problem. Your first message is."* | Opener rescue $4 | "Fix one message — $4" |
| B | *"Dating changed while you were away. Catch up without the cringe."* | Profile restart $45 | "Talk to someone who's done this" |
| C | *"Send us the conversation. We'll tell you if he's worth your Thursday."* | Screening $6 | "Get a second opinion" |
| D | *"Dating advice that doesn't assume you're straight."* | Convo help $5 | "Find your coach" |
| E | *"Twenty minutes a week on dating, spent well."* | Retainer $80/mo | "Hand it to someone" |
| Coaches | *"You're already answering these for free at 1am."* | 90% coach-sourced | "Apply to coach" |

**Brand guardrails, permanently:** no pickup-artist vocabulary, no manipulation tactics, no "get any girl", no before/after mockery of real people, no shame-based hooks. We sell **communicating better as yourself**. This isn't only ethics — the manipulation framing caps your market at one segment, invites platform bans, and makes Segment C impossible.

## 12. Sequencing — do not do this in parallel

**Phase 1 (months 1–2): Segment A only, TikTok + Reddit.**
Highest volume, highest frequency, cheapest to reach, clearest pain. Everything you learn about metering, answer quality, and repeat purchase comes from here. 8 coaches, all conversation/opener specialists.

**Phase 2 (months 3–4): add Segment B.**
Once the loop works, add the money. Recruit 3–4 coaches who specialise in 40+ restarts. New channels: Facebook groups, podcasts. This is where AOV triples and retainers start.

**Phase 3 (months 5–6): add Segment C.**
Requires women coaches and a distinct product framing (screening, not seduction). Instagram-led. Best referral dynamics of any segment — this is where organic growth compounds.

**Phase 4: D and E opportunistically**, driven by which coaches you happen to recruit. Never F.

**Why sequential:** each segment needs different coaches, different copy, and different channels. Running two at once with a five-coach supply means both are under-served and neither result is legible.

## 13. Metrics per segment

Track these separately from day one — a blended repeat rate will hide the real answer.

| Metric | A | B | C |
|---|---|---|---|
| Target AOV | $12 | $60 | $20 |
| Target repeat (30d) | **40%** | 15% | **35%** |
| Retainer conversion | 5% | **20%** | 10% |
| Expected CAC (organic) | <$2 | <$8 | <$3 |
| Primary channel | TikTok | FB + podcasts | Instagram |

**If Segment A's repeat rate is under 25%, the marketplace thesis is wrong** — that's the segment with the most naturally recurring need. Segment B repeating at 15% is *fine and expected*; a profile restart shouldn't need repeating.

## 14. The 90-day picture

| | Days 1–30 | Days 31–60 | Days 61–90 |
|---|---|---|---|
| **Supply** | 8 coaches (Segment A specialists) | 12 coaches, +B specialists | 20 coaches, +C |
| **Channels** | TikTok daily, Reddit teardowns | + YouTube Shorts, FB groups | + Instagram, podcasts |
| **Demand focus** | Segment A only | A + B | A + B + C |
| **Revenue goal** | Prove repeat purchase | ~$500 net | ~$1,500 net |
| **Milestone** | 40 transactions, repeat rate read | First retainer subscribers | Coach Pro launched |

---

## 15. The honest strategic summary

> **⚑ CONDITIONALLY SUPERSEDED — see `MATCH-THREADS-DESIGN.md` §11.** Everything below is correct *while the marketplace is the only product*. The two-thread design changes the shape: an AI match thread is useful with **zero coaches**, which removes the cold start, makes marketing single-sided, and inverts the coach pitch from "join my empty marketplace" to "3,000 people here are already asking." The business becomes **a product with a human-services layer** rather than an audience-aggregation play.
>
> Two things do **not** change. Paid acquisition remains arithmetically closed on take rate alone — 18% of a $25 transaction is $4.50, against a $4–15 dating-vertical CPI in an auction against companies with $17–33 RPP. It becomes *marginal* (~0.76–3.05:1 versus a 4:1 benchmark) only if the AI tier is a **paid subscription**, which produces ~$61 per payer versus $4.50 per transaction. And organic stays the channel until subscription retention is proven, since AI apps churn ~30% faster than others.
>
> **Sequencing consequence:** §12 below can collapse from parallel supply-and-demand per segment to **demand-first, supply recruited against proven demand.**

The business is **an audience-aggregation play disguised as a marketplace**. You cannot buy customers — the arithmetic forbids it — so every customer arrives through a coach who already had them. That single constraint dictates everything: the 10% coach-sourced take rate, distributed TikTok authorship instead of a brand account, supply-weighted recruiting, and sequencing by segment rather than blasting all six.

The model is a ladder: **$4 buys the customer, packs and deliverables earn the margin, retainers fix the LTV.** Any decision that improves the $4 answer's margin at the expense of its conversion rate is a mistake.

And the strategic risk worth watching from day one: if growth only ever comes through individual coaches' audiences and marketplace discovery never contributes, you haven't built a marketplace — you've built **coach infrastructure**. Take-rate economics get worse; SaaS economics get much better. That would be a pivot to embrace early rather than resist, and the differential take rate is deliberately designed so you'd see it coming in the data.
