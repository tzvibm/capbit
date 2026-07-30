# Wing — Market Analysis & Strategic Position

**v1 · July 2026 · Researched market model, competitive landscape, and strategic frameworks**

Companion to `GO-TO-MARKET.md` (business model and segments) and `MARKETING-PLAN.md` (the $200 validation run). Those two documents assert a strategy. This one tests it against the market as it actually exists in July 2026.

**On the data.** Every number below is sourced and linked. Where sources conflict — and on market size they conflict badly — both figures are shown with the definitional reason. Where a figure is an estimate of mine built from sourced inputs, it is labelled **[derived]**. Nothing here is invented, and the arithmetic is shown so you can re-run it when the inputs move.

**Four findings changed my view of the plan while writing this.** They are flagged **⚑ REVISION** in place and collected in §33.

---

> **⚑ PARKED — the product pivoted to a standalone agentic coach with no human marketplace. See `AGENT-PRODUCT.md`.**
> This document's research stands and is still cited elsewhere: market sizing, competitor pricing, regulatory findings, unit economics. What no longer applies is the assumption that coaches are the product. The human layer may return later as a premium verification tier rather than as the business.

# PART I — WHAT MARKET IS THIS, ACTUALLY

## 1. The market Wing is in is not the market it sounds like

The instinct is to size this against online dating. That is wrong by an order of magnitude in both directions, and the error matters because it sets the wrong expectations for both revenue and competition.

| Market | 2026 size | Is Wing in it? |
|---|---|---|
| Global online dating | **$11.8–15.5B** ([Fortune](https://www.fortunebusinessinsights.com/online-dating-market-115542), [Straits](https://straitsresearch.com/report/online-dating-market)) | **No.** Wing does not match people. Zero of this revenue is addressable. |
| Global dating app revenue | **$3.24B**, US $1.45B ([Statista via getstream](https://getstream.io/blog/dating-app-statistics/)) | **No** — but this is the *wallet* Wing shares. |
| US Dating Services industry | **$3.2B**, 380 businesses ([IBISWorld](https://www.ibisworld.com/united-states/market-size/dating-services/1723/)) | **Adjacent.** Matchmaking and services, not advice. |
| Global professional coaching | **$5.34B**, 122,974 practitioners ([ICF 2025](https://coachingfederation.org/blog/coaching-industry-continues-global-growth-with-5-34-billion-usd-revenue-new-research-reveals/)) | **Adjacent.** Wing's supply pool, not its demand. |
| Dating coaching specifically | **$363M (2015)**, ~400 coaches ([Marketdata via marketresearch.com](https://blog.marketresearch.com/american-singles-fuel-the-2.5-billion-dating-market)) | **Yes** — and this is the only honest label. |

That last row is the uncomfortable one. The only market segment that describes what Wing sells was measured at **$363 million with roughly 400 practitioners**, and the most recent public sizing of it is a decade old. There is no current, credible, independent sizing of the dating-advice market. That is itself a finding: **Wing is not entering a market, it is trying to formalise a cottage industry.**

That cuts both ways. A $363M segment with 400 named practitioners is too small to interest Match Group and too unstructured to have an incumbent — which is exactly the condition under which a marketplace can be built. It is also too small to support the revenue ramp that a venture investor would want, which is fine for a bootstrap and disqualifying for a raise. Know which one you are doing.

## 2. Bottom-up sizing — TAM / SAM / SOM

Top-down market reports are useless here because none of them have a line item for "someone tells you what to text back." So build it from population and demonstrated payment behaviour.

### TAM — everyone who could conceivably buy

| Step | Figure | Source |
|---|---|---|
| US adults | ~262M | Census baseline |
| Unmarried adults 18+ | **117.6M** | [Census](https://www.census.gov/newsroom/stories/unmarried-single-americans-week.html) |
| Have ever used a dating site/app | **30% of adults ≈ 79M** | [Pew](https://www.pewresearch.org/short-reads/2023/02/02/key-findings-about-online-dating-in-the-u-s/) |
| Currently using | **7% of adults ≈ 18M** | [Pew](https://www.pewresearch.org/short-reads/2023/02/02/key-findings-about-online-dating-in-the-u-s/) |
| Broader "online dating users" measure | **60.5M (2024)** | [Statista](https://www.statista.com/topics/2158/online-dating/) |

The 18M vs 60.5M gap is a definitional fight (last-30-days active vs registered-in-year), not a data error. **TAM ≈ 79M US adults with dating-app experience.** Use it as a ceiling and nothing else.

### SAM — the pool with demonstrated willingness to pay

This is the number that matters, because Wing's whole thesis rests on people who have already proven they will spend money on their dating life.

| Step | Figure | Source / math |
|---|---|---|
| Of ever-users, share who have ever paid | **35%** | [Pew](https://www.pewresearch.org/internet/2023/02/02/from-looking-for-love-to-swiping-the-field-online-dating-in-the-u-s/) |
| → US adults who have ever paid for dating | **79M × 35% ≈ 27.6M** | **[derived]** |
| Concurrent payers, top two platforms, global | **16.7M** (Match 13.5M + Bumble 3.2M) | [Match Q1'26](https://www.prnewswire.com/news-releases/match-group-announces-first-quarter-results-302763209.html), [Bumble Q1'26](https://ppc.land/bumbles-paying-users-fall-21-but-profits-jump-165-in-q1-2026/) |
| → US concurrent payers, all platforms | **~7M** | **[derived]** — Americas ≈ 45–50% of Match revenue, plus long tail |
| In Wing's segments A+B+C (25–55, hetero + Segment C) | **~40%** | **[derived]** from Pew age distribution (37% of 30–49 have used apps) |

**SAM ≈ 11M US adults** who have paid for dating before and sit in a target segment. **[derived]**

Revenue SAM, at the plan's own $25 blended AOV and a deliberately conservative 5% ever-try rate:

> 11M × 5% = **550,000 buyers** × $30/yr spend = **$16.5M consumer GMV** → at 18% blended take = **~$3.0M/yr platform revenue.** **[derived]**

### SOM — what the plan actually targets

`GO-TO-MARKET.md` §5 targets 50 coaches, 2,500 txns/month, ~$10k/month net → **$120k/yr, which is 4% of the revenue SAM.**

## 3. The sizing conclusion, stated plainly

**Market size is not the binding constraint on this business.** The plan needs 4% of a conservatively-derived SAM to hit its own stated goal. You could be wrong about the SAM by 10× in the pessimistic direction and the 90-day plan would still be reachable.

That is genuinely good news, and it reframes the entire strategic question. Stop asking "is the market big enough." Start asking the two questions that the rest of this document is about: **can you get supply that is good enough at these prices, and can you survive the price anchor that AI has already set?** Both answers are in Part III.

---

# PART II — DEMAND-SIDE MACRO

## 4. The incumbents are contracting — and that is not straightforwardly good for you

| Platform | Q1 2026 payers | YoY | Revenue | Source |
|---|---|---|---|---|
| Tinder | 8.6M | **−5%** | $455M (+2%) | [Match Q1'26](https://finance.biggo.com/news/US_MTCH_2026-05-05) |
| Match Group total | 13.5M | **−5%** | $864M (+3.9%) | [PRNewswire](https://www.prnewswire.com/news-releases/match-group-announces-first-quarter-results-302763209.html) |
| Bumble total | 3.2M | **−21%** | $212M (−14%) | [ppc.land](https://ppc.land/bumbles-paying-users-fall-21-but-profits-jump-165-in-q1-2026/) |
| **Hinge** | **2.0M (+15%)** | **+15%** | **$194M (+28.3%)** | [Match Q1'26](https://finance.yahoo.com/markets/stocks/articles/match-group-q1-2026-earnings-145613629.html) |

Tinder's MAU decline slowed to −7% in March, the slowest in 31 months, and registrations grew for the first time in nearly two years. Bumble cut 30% of staff ([Fox Business](https://www.foxbusiness.com/technology/bumble-announces-major-layoffs-affecting-30-employees-company-restructures)) and is [exploring a sale](https://www.inc.com/georgia-fearn/bumble-explores-a-sale-amid-user-slump/91365975).

**Read it carefully.** Three things are true at once:

1. **The pool of app payers is shrinking.** Wing's SAM has a −5% to −21% annual drag on it. Sizing this market as growing is wrong.
2. **Spend per remaining user is rising fast.** Tinder RPP $17.56 (+7%), Hinge RPP $33.13 (+11%), Bumble ARPPU $22.04 (+8.9%). The people who stay are paying more. Bumble is explicitly executing "shrink the base, extract more from who remains."
3. **Hinge is the exception and it is where your customer lives.** +28% revenue, +15% payers, and its own data says likes on text prompts were **47% more likely to lead to a date than likes on photos** ([Hinge](https://hinge.co/newsroom/prompt-feedback)).

That third point is the most actionable demand fact in this document. Hinge is the one growing surface, it is text-and-prompt-centric rather than photo-centric, and its own research says *words drive outcomes*. **⚑ REVISION: Wing should be Hinge-first, not app-agnostic.** The current plan treats Tinder/Hinge/Bumble as interchangeable. They are not: one of them is growing 28%, is structurally about writing, and has published research proving writing is the lever. Lead every asset with Hinge.

## 5. The burnout data is real, large, and double-edged

- **78%** of Gen Z report dating app fatigue ([Forbes Health](https://www.forbes.com/health/dating/dating-app-fatigue/))
- **79%** have experienced exhaustion from online dating; **55%** report fatigue within 6 months ([zipdo](https://zipdo.co/gen-z-dating-statistics/))
- **41%** have been ghosted; **40%** cite inability to find a good connection ([Forbes Health](https://www.forbes.com/health/dating/dating-app-fatigue/))
- **72%** of Gen Z singles question profile authenticity ([zipdo](https://zipdo.co/gen-z-dating-statistics/))
- Nearly **80%** of US college students have stopped using dating apps entirely ([GRASS](https://grass.camp/en-US/blog/gen-z-friendship-revolution-2026))

Burnout is the demand driver *and* the churn mechanism. Every one of these numbers describes someone about to become either a customer or a non-dater. Ghosting at 41% is precisely Segment A's job-to-be-done, and it is the single most defensible demand statistic Wing has.

But note what burnout actually predicts: **exit, not spend.** People who are exhausted by a product's core loop are a hard cohort to upsell within that loop. Wing's honest position is that it sells to the sub-population that is frustrated *but still trying* — and nobody publishes the size of that slice.

## 6. Revealed willingness to pay

| Anchor | Figure | Source |
|---|---|---|
| Hinge revenue per payer | **$33.13/mo** | [Match Q1'26](https://finance.yahoo.com/markets/stocks/articles/match-group-q1-2026-earnings-145613629.html) |
| Bumble ARPPU | **$22.04/mo** | [ppc.land](https://ppc.land/bumbles-paying-users-fall-21-but-profits-jump-165-in-q1-2026/) |
| Tinder RPP | **$17.56/mo** | [Match Q1'26](https://finance.biggo.com/news/US_MTCH_2026-05-05) |
| Ever paid for dating apps | **35%** of users; **41%** of 30+; **45%** of upper-income | [Pew](https://www.pewresearch.org/internet/2023/02/02/from-looking-for-love-to-swiping-the-field-online-dating-in-the-u-s/) |
| Willing to pay for premium features (2018) | **19%** of men, **6%** of women | [Statista](https://www.statista.com/statistics/809411/us-spending-willingness-for-adult-dating-website-or-app-membership/) |

**The $17–33/month band is the sanity check on Wing's entire price ladder.** A rung-3 retainer at $60–100/mo asks the customer to spend 2–3× what they spend on the app itself. That is not impossible — matchmaking clients pay $4,900–70,000 ([Tawkify](https://www.vidaselect.com/tawkify-cost)) — but it means the retainer is a Segment B/E product exclusively, and pitching it to Segment A will fail on price alone. The existing plan's retainer-conversion targets (5% for A, 20% for B) are correctly ordered. Good.

The 19%/6% male-female WTP split also independently validates sequencing Segment A (men) before Segment C (women), for commercial rather than editorial reasons.

## 7. AI adoption and the backlash — the most important demand dynamic

| Metric | Figure | Source |
|---|---|---|
| Singles using AI in dating | **26%**, up **333% YoY** | [Match / Kinsey Singles in America 2025](https://match.mediaroom.com/2025-06-10-Match-and-The-Kinsey-Institute-Unveil-14th-Annual-Singles-in-America-Study) |
| Gen Z singles using AI | **~49%** | [Psychology Today](https://www.psychologytoday.com/us/blog/a-funny-bone-to-pick/202506/ai-use-in-dating-jumps-333) |
| Of AI users: wrote their profile | **43%** | [IU News](https://news.iu.edu/live/news/34137-singles-in-america-study-daters-breaking-the-ice) |
| Of AI users: wrote a first message | **37%** | [IU News](https://news.iu.edu/live/news/34137-singles-in-america-study-daters-breaking-the-ice) |
| Daters who believe they've received AI-written messages | **60%** | [Norton via Scientific American](https://www.scientificamerican.com/article/the-rise-of-ai-chatfishing-in-online-dating-poses-a-modern-turing-test/) |
| Call AI-written messages an instant dealbreaker | **~80–81%** (HNW sample) | [Luxy](https://millionairedating.onluxy.com/ai-generated-dating-messages-dealbreaker.html) |
| "Chatfishing" search interest | **+5,000%** | [Global Dating Insights](https://www.globaldatinginsights.com/featured/chatfishing-becomes-increasingly-common-among-singles/) |

This is the central tension in Wing's market, and it must be read honestly rather than optimistically.

**The optimistic read** — the one to build marketing on — is that a norm is forming against AI-written dating messages while adoption of them explodes. 60% of daters think they've been chatfished. A term for the offence went from nowhere to +5,000% search interest. Wing sells the exact opposite of the thing the market is developing an immune response to. That is a real, timely, defensible wedge: **"a person wrote this, and they'll tell you why."**

**The pessimistic read** — the one to plan on — is that this is a stated-versus-revealed preference gap, and revealed preference is winning. 26% of singles *use* AI (49% of Gen Z) while ~80% call it a dealbreaker *in others*. Both groups are the same people. The disapproval is directed outward; the wallet points at the AI apps. The Luxy 81% figure is also from a self-selected high-net-worth dating platform, not a representative sample — treat it as directional colour, not evidence.

**The strategic consequence:** the anti-AI positioning is a *marketing* asset and must not be mistaken for a *demand* asset. Do not build the business on the assumption that people will pay a premium to avoid AI. Build it on the assumption they will pay for a judgment call, and use the anti-AI norm as the hook that makes them listen.

---

# PART III — COMPETITIVE LANDSCAPE

## 8. Seven tiers of competitor, with real prices

Wing's competitive set is not other dating-advice marketplaces (there are effectively none at scale). It is a price ladder running from free to five figures, and Wing is inserting itself at the bottom rung.

| Tier | Examples | Real price | Human? | Judgment? | Threat to Wing |
|---|---|---|---|---|---|
| **0 · Free human** | r/dating_advice (**4.7M**), r/datingoverthirty (**1.2M**), r/OnlineDating (~105k) ([Hive Index](https://thehiveindex.com/communities/r-dating_advice/)) | **$0** | Yes | Variable | **Highest.** Same product, free, instant, anonymous. |
| **1 · General AI** | ChatGPT, Claude, Gemini | **$0–20/mo** bundled | No | No | **Very high.** Zero marginal cost, already installed. |
| **2 · AI wingman apps** | [Rizz](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/) (7.5M downloads, 1.5M MAU), [YourMove](https://aitools.inc/tools/yourmove-ai) (300k users), Texting Wingman, WRizz, Winggg, Wingman.live | **$4.99–6.99/wk**, $99.99/yr, ~$35/mo | No | No | **Very high — see §9.** |
| **3 · AI+human hybrid** | [Roast](https://roast.dating/) — AI score + expert review in 24–48h | **$6.99 → $19.99/mo**, one-offs $7–97 | Partly | Partly | Direct. Proves the hybrid price point. |
| **4 · Gig marketplaces** | Fiverr dating coaching (~48 gigs listed), Upwork | **$10–150/gig** | Yes | Yes | Moderate — this is also Wing's supply. |
| **5 · Done-for-you agencies** | [VIDA Select](https://blog.photofeeler.com/vida-select-review/), Virtual Dating Assistants, ProfileHelper | **$995–2,995/mo** | Yes (ghostwriters) | Yes | Low overlap, huge margin envy. |
| **6 · Coaches & matchmakers** | Independent coaches; [Tawkify](https://www.vidaselect.com/tawkify-cost), Three Day Rule | **$90–500/hr**; packages $1,000–3,000; matchmaking **$4,900–70,000** | Yes | Yes | Low — different buyer entirely. |

Two structural observations.

**Tier 0 is the real competitor and the plan under-weights it.** 4.7M people in r/dating_advice get exactly Wing's product — a human reading their situation and telling them what to do — for free, instantly, anonymously, at any hour. `GO-TO-MARKET.md` treats Reddit as a *channel*. It is also the incumbent, and it beats Wing on price, speed, and embarrassment. The only axes Wing can win on are **accountability** (a named person whose ranking depends on being right), **specificity** (someone who reads *your* screenshots properly rather than skimming), and **discretion** (not posting your dating life to 4.7M strangers). Those three are the entire product thesis and they should be the entire homepage.

**There is no competitor in the middle.** Between Fiverr's $10–150 one-shot gigs and VIDA's $995/mo, there is nothing structured. Wing's $4–50 metered band with repeat purchase is genuinely unoccupied. That white space is real. It is also unoccupied for a reason, which §9 explains.

## 9. The price anchor problem — the single most important competitive fact

Set Wing's core product against the substitute directly:

| | Wing | AI wingman app |
|---|---|---|
| Unit | **1 answer** | **Unlimited answers** |
| Price | **$4–6** | **$4.99–6.99 / week** ([Texting Wingman](https://apps.apple.com/jm/app/texting-wingman-rizz-assistant/id6737630517), [WRizz](https://apps.apple.com/us/app/wrizz-texting-coach/id6743861531)) |
| Latency | minutes–hours | seconds |
| Input | screenshot | screenshot |
| Marginal cost to provider | a human's time | ~$0.001 |

**At the point of purchase, a customer facing "what do I say back?" can pay ~$5 for one human answer or ~$5 for a week of unlimited machine answers.** Per unit of output, Wing is 20–40× more expensive than the substitute, consuming the identical input (a screenshot) and answering the identical question.

This is not in the current strategy documents and it invalidates one specific line of reasoning in them. `GO-TO-MARKET.md` §2 treats the $4 answer as "a price below their decision threshold" — a frictionless impulse buy. Against a blank slate, true. Against a market where $4.99 buys a week of unlimited, **the $4 answer is not cheap, it is expensively priced per unit, and the buyer's alternative is one tap away.**

The consequence is not to cut the price. Cutting it makes the margin problem worse and the positioning weaker. The consequence is that **Wing can never win on "help with this message" framing, because that is a category where it is 30× overpriced.** It must sell a different unit of value:

- Not *a reply* → **a read.** "Here's what's actually happening in this conversation, and here's the one line that changes it."
- Not *text* → **a verdict.** "Don't send anything for two days" is advice no AI will give and no AI-optimised-for-engagement product can afford to give.
- Not *output* → **accountability.** A named human whose ranking falls if they were wrong.

> **⚠ CORRECTED — see `AI-LANDSCAPE.md` §13.** An earlier version of this section cited an Android-only estimate (~300k monthly downloads, ~$30k revenue, ~$0.10/download) and concluded the AI category "cannot monetise." That understated it materially. **Rizz as a company is reported at $15M+ revenue** on 7.5M users and 1.5M MAU, at ~$7/week or ~$20/month ([youmind](https://youmind.com/landing/x-viral-articles/rizz-app-faceless-creator-growth), [Forbes](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/)).

The accurate read is narrower and less comfortable: the category monetises **thinly per user but successfully in aggregate**. It has trained the market to expect this help for ~$5/week, which is bad for Wing's price — and it has an established, well-capitalised leader growing through *faceless creator networks*, which is the same channel Wing's go-to-market depends on. That is competition for creator attention, not an open field.

**⚑ REVISION: rewrite the Segment A objection handling.** The current answer to "why not just use ChatGPT?" is good but incomplete, because the real objection in 2026 is not ChatGPT — it is "I already pay $5 a week for an app that does this." Wing needs an answer to *that*, and the answer is that the app will always tell you to send something.

## 10. Positioning map — where the white space is

Two axes that actually separate the field: **price per interaction** and **degree of human judgment.**

```
                        HIGH JUDGMENT
                             │
   Matchmakers ($4.9k–70k) ● │
                             │
      VIDA ($995–2,995/mo) ● │
                             │
        Coaches ($90–500/hr) ●
                             │
           Fiverr ($10–150) ●│
                             │      ← ← ← WING ($4–50, metered, repeat)
    Reddit (free) ●          │
─────────────────────────────┼─────────────────────────────
  FREE                       │                    EXPENSIVE
                             │
              Roast ($7–20) ●│
                             │
        AI apps ($5/wk) ●    │
                             │
      ChatGPT (free) ●       │
                             │
                        NO JUDGMENT
```

Wing's intended position — **low price, high judgment, metered, repeatable** — is genuinely vacant. Nobody occupies it. The strategic question is whether it is vacant because it is valuable and hard, or vacant because the economics do not close. §29 and §30 answer that: **the economics close only on the web, only with prepaid credit, and only with sub-$20/hr supply.** All three are solvable. None are optional.

---

# PART IV — PORTER'S FIVE FORCES

Scored against the evidence above.

## 11. Threat of substitutes — **SEVERE**

The defining force in this market. Free human advice at 4.7M-member scale, general AI at zero marginal cost, and purpose-built AI apps at $5/week unlimited. Every one of them consumes the same input and answers the same question. Substitution is not a future risk; it is the status quo Wing must displace.

*Mitigation:* compete on the attributes substitutes structurally cannot deliver — accountability, discretion, and advice against the user's short-term impulse. Not on output.

## 12. Buyer power — **HIGH**

Zero switching cost, zero contract, zero lock-in, purchase-by-purchase. Buyers are price-sensitive (Segment A explicitly so) and have a free alternative. There is one genuine mitigant, and it is the most underrated asset in the model: **relationship continuity.** A coach who already knows your situation has real, compounding value to a repeat buyer, because re-explaining context is the actual cost of switching. That is why rung 3 exists, and it is the only place buyer power meaningfully weakens.

## 13. Supplier (coach) power — **MODERATE, rising with success**

Individually weak — thousands of potential coaches, none essential at launch. Structurally strong, because in a model where coaches bring their own audiences, **the coach owns the customer relationship and can leave with it.** Service marketplaces routinely see **>20% leakage early**, with estimates ranging **30% to 80%** ([Hokodo](https://www.hokodo.co/resources/how-to-prevent-disintermediation-on-your-b2b-marketplace)).

The 10% coach-sourced take rate is the correct and deliberate answer — it prices disintermediation out of rationality. At 10%, leaving to invoice by hand costs a coach more in payment friction and lost trust signal than it saves. **This is the strongest single design decision in the existing plan, and §26 shows it is the exact fix for what killed the closest comparable company.**

## 14. Threat of new entry — **HIGH**

There is no meaningful barrier. The prototype is one HTML file. A competitor can replicate the product in weeks and the rate card in an afternoon. Capital is not a barrier; neither is technology. The only barriers Wing can build are the two slow ones: **a quality dataset nobody else has** (defect-rate ranking, per `ARCHITECTURE.md` §9.7) and **a trust brand**. Both take 18–24 months. Neither exists yet. See §22.

## 15. Competitive rivalry — **LOW today, and this is the opportunity**

Remarkably, there is no direct rival. No structured marketplace for metered human dating advice appears to exist at scale — extensive searching surfaced none. The adjacent players are non-overlapping: agencies serve a $1,000/mo buyer, matchmakers a $5,000 buyer, Fiverr is unstructured, AI apps are unstaffed.

Most tellingly, **the incumbents have vacated this lane.** Match launched [AskMatch](https://techcrunch.com/2019/05/14/match-now-offers-dating-coaches-who-help-its-members-with-profiles-dating-challenges/) in 2019 — real human coaches, phone sessions, bundled into a ~$35/mo subscription — and it has left no public trace since. Meanwhile Hinge's founder Justin McLeod left in 2026 to launch [Overtone](https://www.fastcompany.com/91574215/hinge-founders-new-dating-app-lets-ai-be-your-matchmaker-theres-already-a-waitlist-overtone), a **$18M Match-Group-backed voice-first AI matchmaker with no profiles or swipes**.

The best talent and capital in dating is moving toward AI-mediated matching and away from human advice. **Low rivalry in Wing's lane is not an accident — it is a consequence of everyone else betting the other way.** That is the opportunity and simultaneously the thing that should worry you: they may be right.

> **⚠ REFINED — see `AI-LANDSCAPE.md` §14.** This section originally concluded incumbent threat was low *overall*, inferring from AskMatch's abandonment. That holds for human coaching and is now clearly wrong for AI assistance: Match Group is rolling AI "wingmen" across Tinder and Hinge (photo selection, message writing, "coaching for struggling users"), and Grindr is shipping per-match AI memory to 14M users by 2027 while testing **$349–500/month** for it. Split the lanes:
>
> | Lane | Incumbent threat | Direction |
> |---|---|---|
> | AI assistance — replies, profile help, per-match memory | **Severe**, bundled free | Worsening fast |
> | Paid human judgment — named, accountable, cross-platform | **Low** | Stable |
>
> The argument above survives and strengthens. What changed is that the industry's bet is now funded and shipping, which raises the cost of joining it and lowers the cost of standing apart. Grindr's $500/month tier also proves the ceiling on dating-help pricing is set by positioning, not by category — which strengthens both the up-market pivot (§36) and the $8–10 answer test (§34).

## 16. Five Forces summary

| Force | Intensity | Direction |
|---|---|---|
| Substitutes | **Severe** | Worsening |
| Buyer power | **High** | Stable |
| Supplier power | **Moderate** | Worsens with scale |
| New entry | **High** | Stable |
| Rivalry | **Low** | Worsens if Wing proves the model |

**Four of five forces are unfavourable.** That is normal for a marketplace at inception and it does not condemn the idea — it dictates where the effort goes. The only force that favours Wing is the empty lane, and empty lanes close. **Everything in this analysis argues for speed on one thing: accumulating the quality dataset that is the only defensible asset available.**

---

# PART V — SWOT

Restricted to items with evidence behind them.

## 17. Strengths

| Strength | Evidence |
|---|---|
| **Occupies genuine white space** | No structured competitor found between Fiverr ($10–150 one-shot) and VIDA ($995/mo). §8. |
| **Take-rate design pre-empts the known failure mode** | Cameo lost creators to a 25% fee and 90% of its valuation. Wing's 10% coach-sourced rate directly addresses it. §26. |
| **Supply is underpaid and reachable** | VIDA-style ghostwriters earn **$13–17/hr** writing these exact messages while the agency charges **$995–2,995/mo**. Wing at 4 answers/hr ≈ $18/hr beats their wage with autonomy and attribution. |
| **Rides a forming social norm** | 60% of daters believe they've been chatfished; "chatfishing" +5,000% search. Wing is the only positioning that benefits. §7. |
| **Product format is already a proven content genre** | Teardown content is native to TikTok/Reels; distribution and product are the same artefact. |
| **Timing on Hinge** | +28% revenue, +15% payers, and Hinge's own data: text prompts **47%** more likely to lead to a date than photos. §4. |
| **Capital efficiency** | $200 validation budget against a business needing 4% of derived SAM to hit target. §3. |

## 18. Weaknesses

| Weakness | Evidence |
|---|---|
| **Priced 20–40× above the substitute per unit** | $4–6/answer vs $4.99–6.99/week unlimited. §9. **The central strategic problem.** |
| **Payment processing eats 10.4% of a $4 order** | Stripe $0.416 on $4. Micro-transactions are structurally unprofitable without basket aggregation. §30. |
| **Cannot afford expert labour at these prices** | ICF: US coaches average **$71,719/yr on 11.6 hrs/wk ≈ $119/hr effective** ([ICF 2025](https://coachingfederation.org/resource/2025-icf-global-coaching-study-executive-summary/)). Wing offers ~$18/hr. Supply is structurally capped at sub-$20/hr labour or coaches using Wing as lead-gen. §31. |
| **Native iOS is economically closed** | Apple 3.1.3(d) exempts only **real-time** person-to-person services. Async answers require IAP at 30%, which makes a $4 answer loss-making. §29. |
| **No moat exists yet** | VRIO (§22): every current asset is competitive parity or a temporary advantage. |
| **Brand name collides with the AI category it opposes** | "Wingman: AI Dating Coach", "Wingman.live", "Wingman AI: Texting Guide", "Texting Wingman", "Winggg" all exist and all are AI products. §32. |
| **Trust product with unlicensed practitioners** | Coaches have no confidentiality privilege and can be [subpoenaed](https://www.goodtherapy.org/blog/Psychotherapy-vs-Coaching-Legal-Distinction). One bad actor is an existential brand event. |
| **Shrinking addressable base** | App payers −5% (Match) to −21% (Bumble) YoY. §4. |

## 19. Opportunities

| Opportunity | Evidence |
|---|---|
| **Incumbents have vacated human advice** | AskMatch abandoned; Match's capital is behind Overtone's AI matchmaker. §15. |
| **AI apps have huge funnels and no monetisation** | Rizz ~300k downloads/mo → ~$30k revenue (~$0.10/download). A demand pool nobody is converting. §9. |
| **Ghostwriter arbitrage is large and legal to pursue** | $13–17/hr labour producing $995–2,995/mo service revenue. Recruit the labour, not the clients. |
| **Hinge-native positioning is unclaimed** | Fastest-growing surface, text-centric, publishes research that supports Wing's thesis. |
| **Segment C is structurally underserved** | Screening/second-opinion is a distinct job nobody sells, with the best referral dynamics in the plan. |
| **Prepaid credit doubles rung-1 margin** | $20 pack = 4.4% payment cost vs 10.4% on singles. A payments fix disguised as an upsell. §30. |
| **The pivot to coach infrastructure is live** | If discovery never contributes, SaaS-for-coaches has better economics than the marketplace. Already flagged in `GO-TO-MARKET.md` §15 — the data will show it early. |

## 20. Threats

| Threat | Evidence | Severity |
|---|---|---|
| **The Chegg scenario** | Paid human answers to questions AI can answer: subscribers **−31%**, revenue **−30%**, **45%** of staff cut, stock **−99%** ([Forbes](https://www.forbes.com/sites/petercohan/2025/10/29/chegg-stock-down-99-learn-whether-ai-45-layoffs-make-chgg-a-buy/)). | **Existential** |
| **App-store tax on the core product** | 30% IAP on async answers makes rung 1 loss-making. §29. | **Severe (avoidable)** |
| **State dating-service statutes** | NY caps "social referral service" contracts at **$1,000**, max **2 years**, 3-day cooling-off, bans ancillary bundling ([NY GBL §394-c](https://www.nysenate.gov/legislation/laws/GBS/394-C)). CA parallel ([Civ. §1694](https://codes.findlaw.com/ca/civil-code/civ-sect-1694/)). Wing escapes only by never matching or introducing anyone. §25. | **Severe (avoidable)** |
| **Disintermediation** | >20% leakage typical early; 30–80% claimed. §13. | **High** |
| **Coach misconduct / unlicensed practice** | Utah SB48 now funds investigation of life coaches practising therapy ([KSL](https://www.ksl.com/article/51274140/new-law-aims-to-crack-down-on-unlicensed-life-coaches-practicing-mental-health-therapy)). | **High** |
| **Third-party data in screenshots** | Screenshots contain a non-consenting person's photos and messages. Bumble settled **£32M** over biometric consent; Grindr fined over data sharing. | **High** |
| **Dating-app ToS** | Tinder prohibits third-party services interacting with its Services or Member Content, "including artificial intelligence or machine learning systems" ([Tinder Terms](https://policies.tinder.com/terms)). | **Moderate** |
| **AI closes the judgment gap** | Hinge already ships AI coaching (Prompt Feedback, GPT-4o mini; Convo Starters). If AI gets good at *reading* rather than *writing*, the differentiation evaporates. | **High** |
| **Overtone-class AI matching succeeds** | If AI matching works, "matches but no replies" — Segment A's entire job — shrinks. | **Moderate** |

## 21. SWOT verdict

The strengths are mostly **timing and design**; the weaknesses are mostly **arithmetic**. That is the right way round — arithmetic problems have engineering fixes (web-first, prepaid credit, supply tier selection), whereas timing problems have none. But it means the plan must not be executed as written: three of the weaknesses (§29 Apple, §30 payments, §31 labour ceiling) are things you can only fix *before* launch, not after.

---

# PART VI — VRIO: IS THE DIFFERENTIATION DEFENSIBLE?

SWOT says what you have. VRIO says whether you keep it. Each asset scored on Value, Rarity, Imitability, and Organisation.

| Asset | Valuable | Rare | Costly to imitate | Organised to capture | **Verdict** |
|---|---|---|---|---|---|
| **Coach roster** | Yes | Somewhat | **No** — Fiverr sellers and ghostwriters are poachable by anyone | Partly | **Competitive parity** |
| **10% coach-sourced take rate** | Yes | **Yes** — Cameo 25%, Fiverr 20% | **No** — copyable in a day | Yes | **Temporary advantage** |
| **Screenshot-native metered product** | Yes | Somewhat | **No** — it is one HTML file | Yes | **Temporary advantage** |
| **Defect-rate quality ranking + dataset** | Yes | **Yes** | **Yes** — requires transaction volume that cannot be bought | Yes, by design | **Sustained advantage — unrealised** |
| **Trust brand / no-manipulation stance** | Yes | **Yes** in this category | **Yes** — reputation is slow and asymmetric | Yes, guardrails written | **Sustained advantage — unrealised** |
| **Coach relationship continuity (rung 3)** | Yes | Somewhat | **Yes** — accumulated context cannot be copied | Not yet built | **Potential sustained advantage** |

**The VRIO conclusion is the sharpest strategic statement in this document:**

> **Everything Wing has today is a head start, not a moat. The only two assets that could become moats — the quality dataset and the trust brand — both require transaction volume Wing does not yet have. Therefore the correct objective for the first 12 months is not revenue. It is accumulating rated transactions and an unblemished trust record, as fast as capital allows.**

That reorders priorities. A month spent on a feature that does not produce rated transactions is a month spent on nothing durable. It also means the differential take rate — the cleverest thing in the business model — should be understood as a *customer-acquisition subsidy for volume*, not as a margin decision.

---

# PART VII — DIFFERENTIATION & STRATEGY CANVAS

## 22. Value curve

Scored 1–5 on the attributes buyers actually choose on. The point of a strategy canvas is not to score high everywhere — it is to be **deliberately, visibly low** somewhere.

| Attribute | Reddit (free) | AI apps | Fiverr | VIDA | Matchmakers | **Wing (target)** |
|---|---|---|---|---|---|---|
| Low price | **5** | 4 | 3 | 1 | 1 | **4** |
| Speed / immediacy | 3 | **5** | 2 | 3 | 1 | **3** |
| Human judgment | 3 | 1 | 4 | 4 | **5** | **5** |
| Accountability for being wrong | 1 | 1 | 3 | 3 | 4 | **5** |
| Discretion / privacy | 2 | 4 | 3 | 3 | 4 | **5** |
| Specific to *your* situation | 2 | 3 | 3 | 4 | **5** | **5** |
| Repeatable in small bites | 4 | **5** | 1 | 1 | 1 | **5** |
| Breadth of service | 2 | 2 | 3 | 4 | **5** | **2** |
| Prestige / credentials | 1 | 1 | 2 | 3 | **5** | **2** |

**Wing wins on four attributes and deliberately loses on two.** The two sacrifices — breadth and prestige — are correct and should be defended against internal pressure. Adding breadth turns Wing into VIDA at a tenth the price and destroys the metered model. Chasing prestige means recruiting established coaches who will not work for $18/hr and whose credentials invite the licensing scrutiny in §27.

The three attributes where Wing scores 5 and every substitute scores ≤4 — **accountability, discretion, situational specificity** — are the differentiation. They should be the literal words on the landing page, in place of anything about openers or replies.

## 23. The differentiation statement

Derived from the canvas, not from aspiration:

> **For someone who is getting matches but not getting anywhere, Wing is the only place to get a named human being's read on your actual conversation — for the price of a coffee, without posting it to the internet, from someone whose standing depends on being right.**

Test it against the alternatives: Reddit fails on *named* and *without posting it*. AI fails on *human being's read* and *depends on being right*. Fiverr fails on *price of a coffee*. VIDA and matchmakers fail on price by two orders of magnitude. The statement is discriminating, which is the only test a positioning statement has to pass.

## 24. What Wing must never say

Each of these is a positioning error with a specific cost, and the data above says why:

- **"Better replies than AI"** — puts Wing in a category where it is 30× overpriced per unit. §9.
- **"Get more matches"** — that is the app's job and Wing has no lever on it; it invites a promise Wing cannot keep.
- **Anything pickup-artist** — caps the market at one segment, makes Segment C impossible, and is the fastest way to lose the trust asset that VRIO says is one of only two moat candidates.
- **"Coaching" as the primary noun** — attracts licensing scrutiny (§27) and sets a $100/hr price expectation that collides with a $5 product.
- **Any promise of introductions or matching** — converts Wing into a regulated dating service in multiple states. §25.

---

# PART VIII — PESTEL & THE FOUR LEGAL LANDMINES

## 25. Landmine 1 — state dating-service statutes ⚑

**New York GBL §394-c** governs "social referral service" contracts — any service, for a fee, providing *matching of members* for dating or social contact. Where it applies: **3 business day cooling-off with full refund in 10 days; contracts capped at $1,000; maximum 2-year term; ancillary services cannot be required** ([NY Senate](https://www.nysenate.gov/legislation/laws/GBS/394-C)). **California Civil Code §1694** covers "dating, matrimonial, or social referral services" delivered by exchange of contact details, photo/video selection, or personal introductions, with its own 3-day right to cancel ([FindLaw](https://codes.findlaw.com/ca/civil-code/civ-sect-1694/)).

**Wing is outside both definitions today, for one reason only: it never matches or introduces anyone.** It advises a person about a match they found themselves.

**⚑ REVISION — this becomes a permanent product guardrail with a legal basis, not a taste preference.** The moment Wing adds any member-to-member introduction, matching, or "we'll find you someone" feature, it plausibly becomes a regulated dating service in the two largest states, and the $1,000 contract cap and 2-year limit land directly on any premium tier. Write it into the product constitution alongside "ranking is never for sale."

## 26. Landmine 2 — the app-store tax

Covered in full in §29. Summary: **Apple guideline 3.1.3(d) exempts only *real-time* person-to-person services** ([Apple](https://developer.apple.com/app-store/review/guidelines/)). Wing's core product is asynchronous, therefore not exempt, therefore 30%. Web-first is a survival constraint. Note the perverse detail: rung 3 (live calls) *is* exempt while rung 1 (async answers) is not — the ladder inverts on iOS.

## 27. Landmine 3 — unlicensed practice

Coaching is unregulated but the boundary is being enforced. Utah's SB48 expands therapists' scope and **funds investigation of life coaches acting unlawfully** ([KSL](https://www.ksl.com/article/51274140/new-law-aims-to-crack-down-on-unlicensed-life-coaches-practicing-mental-health-therapy)). Any coach delivering services mirroring a licensed psychotherapist's scope risks charges. Coaches have **no confidentiality privilege and can be subpoenaed** about client conversations ([GoodTherapy](https://www.goodtherapy.org/blog/Psychotherapy-vs-Coaching-Legal-Distinction)).

*Required:* an explicit scope boundary in coach terms; a hard-stop escalation path for disclosures involving self-harm, abuse, or coercive control; no therapeutic language in marketing; disclosure that conversations are not privileged. This is cheap now and very expensive later.

## 28. Landmine 4 — third-party data in screenshots

Every screenshot Wing processes contains a **non-consenting third party's** photos and words. Enforcement in adjacent contexts is live and expensive: **Bumble settled £32M** over biometric consent under UK GDPR; Grindr was fined for unlawful data sharing. GDPR Art. 9 treats biometric identification data as special category. Separately, Tinder's terms prohibit third-party services that interact with its Services or Member Content, "including artificial intelligence or machine learning systems" ([Tinder](https://policies.tinder.com/terms)).

Wing's position is defensible — no API access, no scraping, no automation; a user manually shares a screenshot the same way they would with a friend — but it is not risk-free. *Required:* auto-deletion with a stated short retention window, no facial analysis of any kind ever, no training on user content, an obvious face-blur affordance, and a policy of never storing the third party's images beyond the session. The privacy stance is also a **feature** for Segment C, whose stated primary objection is exactly this.

## 29. Remaining PESTEL factors

| | Factor | Implication |
|---|---|---|
| **P** | FTC restarted negative-option rulemaking (ANPRM March 2026) after the 8th Circuit vacated click-to-cancel in July 2025; ROSCA enforcement never stopped ([Gibson Dunn](https://www.gibsondunn.com/ftc-restarts-negative-option-rulemaking-after-eighth-circuit-vacatur-enforcement-under-rosca-continues/)) | Rung-3 retainers and Coach Pro must ship with frictionless in-product cancellation from day one. Build to the vacated rule anyway; it is coming back and it is table stakes for a trust brand. |
| **P** | Ninth Circuit (Dec 2025) allows Apple a "reasonable commission" on external links, framework TBD ([MacRumors](https://www.macrumors.com/2025/12/11/apple-app-store-fees-external-payment-links/)) | The web-payment escape hatch is narrowing. Lock in web-first now; do not build a business model that assumes 0% forever. |
| **E** | App payers −5% to −21% YoY; spend per payer +7% to +11% | Shrinking base, richer survivors. Favours premium framing over volume framing. |
| **S** | 78% Gen Z fatigue; 41% ghosted; anti-AI norm forming; stigma around paid dating help persists (83% of Americans concealed something about themselves for fear of judgment — [ZINE](https://zine.kleinkleinklein.com/p/state-of-shame-research)) | Discretion is a product requirement, not a nicety. Never require a public profile or real name. |
| **T** | Hinge ships AI coaching (GPT-4o mini Prompt Feedback, Convo Starters); Overtone raises $18M for AI matchmaking | The judgment gap is narrowing from above. Assume 24 months, not 5 years. |
| **L** | 1099-K threshold restored to **$20,000 / 200 transactions** ([IRS](https://irs.gov/newsroom/irs-issues-faqs-on-form-1099-k-threshold-under-the-one-big-beautiful-bill-dollar-limit-reverts-to-20000)) | Materially reduces coach onboarding friction. Almost no coach will hit it early. Small but real tailwind — say so in recruiting. |

---

# PART IX — WHAT THE COMPARABLES PROVE

Five companies have already run parts of this experiment. Their outcomes are the base rates.

## 30. The five post-mortems

### Chegg — the existential case
Paid human/expert answers to questions AI learned to answer. Q1 2025: subscribers **−31% to 3.2M**, revenue **−30% to $121M**. May 2025: 248 laid off (22%). October 2025: a further 388 (**45% of remaining staff**). Stock **−99%**, trading near $1 and fighting delisting. The CEO named ChatGPT and Google's AI answers as the causes ([Forbes](https://www.forbes.com/sites/petercohan/2025/10/29/chegg-stock-down-99-learn-whether-ai-45-layoffs-make-chgg-a-buy/), [FinalRound](https://www.finalroundai.com/blog/chegg-layoffs-2025)).

**Why Wing is not automatically Chegg:** Chegg's answers were *objective and verifiable*. A calculus answer is right or wrong, which is precisely the class of problem LLMs mastered. Wing's answers are *subjective, contextual, and social* — dependent on reading one specific person's tone across one specific thread. **But this is a hypothesis, not a fact, and it is the hypothesis on which the entire business rests.** It should be tested explicitly and early: run blind A/B tests of coach answers versus frontier-model answers on real threads, scored by the recipients' actual replies. If coaches do not measurably win, Wing is Chegg with a two-year lag.

> **Partial early evidence, and it is stage-dependent.** Third-party studies report AI openers outperforming an average person (~60% vs 48% positive response), that **"AI's advantage disappeared as conversations got deeper,"** and that at the commitment stage — asking someone out — **"a genuine, vulnerable human message beat both AI and a professional dating coach"** ([Anketta](https://anketta.app/blog/ai-messaging-assistants-2026)). Provenance is weak (one brand-commissioned study, the rest vendor marketing), so treat the shape rather than the numbers as informative.
>
> If that shape holds, bet 1's answer is not yes or no but **"only at high stakes"** — which is survivable and in fact the basis of the tiering in `MATCH-THREADS-DESIGN.md` §3.5, but it narrows the marketplace to low-frequency high-consequence work and makes the AI tier's revenue essential rather than supplementary. Note the third clause carefully: at the commitment stage the *coach lost too*. The coach's job there is to get the user to write something real, not to write it for them — which is what The Call register does (`UI-DESIGN.md` §4.7).

### Cameo — the supply-side case
Peak revenue >$100M and a >$1B valuation in 2021. By March 2024, valuation **−90%** to under $100M, revenue **−80%** from peak, workforce from ~400 to **fewer than 50**, and it could not pay a $600k FTC fine. Reporting attributes the **creator exodus largely to the 25% fee** ([Startups.co.uk](https://startups.co.uk/news/what-happened-to-cameo/), [TechStartups](https://techstartups.com/2024/07/26/cameo-a-unicorn-tech-startup-once-valued-at-over-1-billion-is-now-broke-and-cant-pay-a-600000-fine/)).

**This is the closest structural analog to Wing that exists**: a marketplace whose customers arrive entirely through individual creators' own audiences, taking a percentage of transactions those creators sourced themselves. It died of exactly the disease Wing's differential take rate was designed to prevent. **The 10% coach-sourced rate is not a nice gesture — it is the single learned lesson from the nearest comparable failure.** Never raise it. Everything else in the model is negotiable; this is not.

### Clarity.fm — the format case
Per-minute expert-call marketplace. **Shut down in 2022**, while search demand for it persists ([Talkspresso](https://talkspresso.com/blog/clarity-fm-alternatives-2026)). Expert-advice marketplaces have a graveyard, and thin transaction volume against high trust requirements is what fills it.

### JustAnswer — the survival case
The one that worked: **13,000+ experts, ~10,000 questions/day, 4M solutions/yr, 650+ staff**, experts earning $2,000–7,000/month ([Sharetribe](https://www.sharetribe.com/create/how-to-build-website-like-justanswer/), [WalletHacks](https://wallethacks.com/justanswer-review/)).

**What JustAnswer has that Wing does not: urgency and stakes.** Its categories are vet, medical, legal, tax — my dog swallowed something, at 11pm, and being wrong is expensive. That urgency is what makes someone pay a stranger immediately, and it is what JustAnswer converts into subscription capture. Dating is high-*emotion* but low-*urgency*: nothing bad happens if you wait, or ask a friend, or send nothing.

**⚑ REVISION: this is the strongest argument yet for a specific product decision.** Wing should aggressively favour the moments in dating that *do* have urgency and a decision deadline — she replied and is waiting; the date is Thursday and you need to decide by tonight; he asked something you don't know how to answer. Time-bounded questions convert. "Improve my profile" does not have a deadline and will always lose to procrastination. Reorder the product catalogue around urgency, not around price.

### AskMatch — the incumbent case
Match Group offered human dating coaches by phone, bundled into a ~$35/mo subscription, launched NYC May 2019, expanded to 18 states with plans for nationwide by January 2020 ([TechCrunch](https://techcrunch.com/2019/05/14/match-now-offers-dating-coaches-who-help-its-members-with-profiles-dating-challenges/)). It has left no public trace since, and Match's coaching investment is now AI.

**Two readings, and both are useful.** The threat of Match building Wing is *low* — they tried human coaching, moved on, and have now spun their best product leader out into an AI matchmaker. But it is also weak evidence that bundled human coaching failed to move an app's numbers. Wing's structural differences from AskMatch are real and worth stating: standalone rather than bundled (so the coach's value is visible and paid-for, not hidden inside a subscription), cross-platform rather than Match-only, per-item rather than a phone call, and choice-of-coach rather than assignment.

## 31. Base rates, summarised

| Comparable | What it tested | Outcome | Lesson for Wing |
|---|---|---|---|
| Chegg | Human answers vs AI | **−99% equity** | Prove judgment beats generation, measurably, early |
| Cameo | Creator-sourced marketplace at 25% | **−90% valuation** | The 10% rate is the whole defence. Never raise it |
| Clarity.fm | Per-unit expert marketplace | **Dead 2022** | Thin volume kills advice marketplaces |
| JustAnswer | Per-question expert marketplace | **Works at scale** | Urgency and stakes are the missing ingredient |
| AskMatch | Human coaching inside a dating app | **Quietly gone** | Incumbent threat is low; bundling is the wrong form |

---

# PART X — UNIT ECONOMICS STRESS TEST

## 32. The iOS arithmetic, which closes a door

If Wing ships as a native iOS app and the async answer requires IAP:

| | Web (Stripe) | Native iOS (IAP 30%) |
|---|---|---|
| Customer pays | $4.00 | $4.00 |
| Platform tax | — | −$1.20 |
| Payment processing | −$0.42 | included |
| Platform receives | $3.58 | **$2.80** |
| Coach share @ 80% | −$3.20 | −$3.20 |
| **Platform net** | **+$0.38** | **−$0.40** |
| Coach share @ 90% (coach-sourced) | −$3.60 | −$3.60 |
| **Platform net** | **−$0.02** | **−$0.80** |

**A $4 async answer on native iOS is loss-making at any take rate that keeps coaches.** Breaking even would require a take rate above 30% — worse than Fiverr's 20%, which destroys the supply pitch that the entire GTM depends on. And note the second-order effect in the last two rows: **even on the web, a $4 answer at the 10% coach-sourced rate loses two cents.** The coach-sourced subsidy is real money, which is the correct trade for acquisition, but it means rung 1 is a *pure* loss leader, not a thin-margin product.

**⚑ REVISION: web-first (PWA) is a survival constraint, not a preference — and it should be written into `ARCHITECTURE.md` with the reason attached**, so nobody proposes an App Store launch in six months without re-reading Apple 3.1.3(d). If native ever ships, only live calls (exempt under 3.1.3(d)) and web-purchased credit may fund async answers.

## 33. The prepaid-credit fix

Stripe's fixed 30¢ is what makes micro-transactions unprofitable. Aggregate the basket and it goes away:

| Purchase | Stripe fee | Effective rate |
|---|---|---|
| $4 single answer | $0.42 | **10.4%** |
| $20 credit pack | $0.88 | **4.4%** |
| $50 credit pack | $1.75 | **3.5%** |

**Moving a customer from single answers to a $20 prepaid pack cuts payment cost per $4 of purchase by 58%** ($0.416 → $0.176) **and lifts rung-1 platform net by 64%** ($0.38 → $0.62 per answer, at the 80% coach share). **[derived]** At the 90% coach-sourced share it takes rung 1 from −$0.02 to **+$0.22 — from loss-making to marginally profitable.**

**⚑ REVISION: credit packs are not an upsell, they are a payments-cost fix, and they should ship in v1 rather than being deferred.** The current plan positions packs as rung 2 for margin reasons. They are also the only way rung 1 stops losing money. Two cautions: disclose expiry honestly (`GO-TO-MARKET.md` §4.3 is right that breakage must never be designed for), and remember that prepaid balances are a consumer-protection surface — several states treat unredeemed balances as escheatable property. Keep expiry generous and the accounting clean.

## 34. The labour-cost ceiling

The uncomfortable arithmetic on supply:

| Coach tier | Alternative wage | Wing @ 4 answers/hr, 90% of $5 = **$18/hr** |
|---|---|---|
| VIDA-style ghostwriter | **$13–17/hr** | **Beats it** — plus autonomy, attribution, own clients |
| Dating profile writer (ZipRecruiter avg) | **$38.94/hr** | Loses 2:1 |
| ICF coach, US average | **$71,719/yr ÷ 11.6 hrs/wk ≈ $119/hr** | Loses 6:1 |
| Established coach with packages | $90–500/hr | Loses 5–28:1 |

**At $5/answer, Wing can only attract labour whose alternative wage is under roughly $20/hour — or coaches who treat it as lead generation for their own higher-ticket work.** There is no third option, and this is a hard constraint, not a recruiting-effort problem.

This independently validates the priority order in `COACH-RECRUITING.md` (ghostwriters and Fiverr sellers first, established coaches at #4) — but it also means answer quality is capped by what sub-$20/hr labour produces, which is exactly the variable the Chegg test in §30 measures. The three escapes, in order of preference:

1. **Raise the answer price to $8–10** for genuinely diagnostic work. $10 at 90% and 4/hr = $36/hr, which reaches the profile-writer tier. This may be the single highest-leverage untested change in the model, and it costs nothing to test.
2. **Lean into lead-gen framing** for coaches with their own funnels — already in the plan, and the right pitch for tiers 5–6.
3. **Accept ghostwriter-tier supply** and compete on *format and accountability* rather than expertise depth. Viable, but it is the version most exposed to Chegg risk.

---

# PART XI — THE BETS, THE KILL CRITERIA, AND WHAT CHANGES

## 35. The five falsifiable bets

The business is exactly these five propositions. Each has a number and a date.

| # | Bet | Falsified if | Read by |
|---|---|---|---|
| **1** | Human judgment beats frontier AI on real threads | Blind A/B: coach answers do not beat model answers on recipient reply rate — **method in `AGENT-HARNESS.md` §17** | Week 6 |
| **2** | People pay for judgment despite a $5/week unlimited substitute | Rung-1 → rung-2 conversion <10% | Day 60 |
| **3** | The need recurs often enough to matter | Segment A 30-day repeat <25% (already the plan's own tripwire) | Day 60 |
| **4** | Coaches bring their own audiences | Coach-sourced share of transactions <50% | Day 90 |
| **5** | Sub-$20/hr labour produces answers customers rate highly | Defect rate >10% at steady state | Day 90 |

Bet 1 is the one nobody has tested and the only one that is existential. **It is also cheap: 30 real threads, blind-scored, one week.** Do it before spending another dollar on supply recruiting — if it fails, everything downstream is wasted motion, and the finding is worth knowing in week 6 rather than month 18.

## 36. Kill and pivot criteria

- **Kill** if bet 1 fails clearly and repeatedly. Selling humans against AI when humans do not measurably win is the Chegg trajectory with a known ending.
- **Pivot to coach infrastructure (SaaS)** if bet 4 succeeds but marketplace discovery never contributes — already anticipated in `GO-TO-MARKET.md` §15, and the differential take rate makes it visible in the data early. Take-rate economics worsen; SaaS economics improve markedly. Embrace it rather than resisting it.
- **Pivot up-market** if bet 2 fails at $4–6 but succeeds at $25+. The evidence for this path is strong: Segment B WTP is $45–150, matchmaking clears $4,900+, and §34 says higher prices are what buy better supply. A smaller, higher-priced, Segment-B-led business is the most likely successful shape if the impulse tier does not convert.
- **Do not pivot to AI-assisted answers to fix margin.** It collapses the only differentiation VRIO identified as defensible, and the anti-chatfishing norm in §7 means getting caught doing it quietly is a brand-ending event in this specific category.

## 37. What this analysis changes in the existing plan

Seven concrete revisions, in priority order:

1. **Web-first is a hard constraint.** Apple 3.1.3(d) makes native async answers loss-making at any viable take rate. Write it into `ARCHITECTURE.md` with the reasoning. §32.
2. **Ship prepaid credit packs in v1.** They cut payment costs 58% and are the only route to a non-negative rung 1. §33.
3. **Run the blind human-vs-AI test in week 6, before scaling supply.** It is the existential bet and it is cheap. §35.
4. **Reposition from "better replies" to "a read you can hold someone to."** The $5/week unlimited substitute means the reply-generation framing is unwinnable. Rewrite the Segment A objection handling to answer the AI *app*, not ChatGPT. §9, §23.
5. **Go Hinge-first, not app-agnostic.** It is the only growing surface, it is text-centric, and its own research says prompts beat photos 47%. §4.
6. **Reorder the catalogue by urgency, not price.** JustAnswer works because its questions have deadlines; profile makeovers do not. Lead with "she replied and is waiting." §30.
7. **Test $8–10 answers alongside $4–6.** The labour ceiling, not demand, may be the real pricing constraint, and this is free to test. §34.

Add two permanent entries to the product constitution, alongside "ranking is never for sale":
- **Never match or introduce users to each other** — it converts Wing into a regulated dating service under NY GBL §394-c and CA Civ. §1694, with a $1,000 contract cap and a 2-year term limit in New York. §25.
- **Never raise the coach-sourced take rate above 10%** — it is the documented cause of death of the nearest structural comparable. §30.

## 38. The honest strategic verdict

**The market is real but small and shrinking at the top; the white space is real and genuinely unoccupied; the economics are viable only on the web with aggregated payments; and the differentiation is currently a head start rather than a moat.**

Four of Porter's five forces are unfavourable. The single favourable one — an empty competitive lane — exists because the industry's best talent and capital are betting the other way, on AI-mediated matching. Wing's entire thesis is that they are wrong about one specific thing: that reading a situation and telling someone an uncomfortable truth is a different job from generating a plausible message, and that the first job is worth paying a person for.

That thesis is testable in six weeks for almost nothing. Everything else in the plan — the segments, the channels, the recruiting, the ladder — is downstream of it and well-designed. **The correct next action is not to execute the plan. It is to run bet 1.**

---

## Appendix — Source index

**Market sizing:** [Fortune Business Insights](https://www.fortunebusinessinsights.com/online-dating-market-115542) · [Straits Research](https://straitsresearch.com/report/online-dating-market) · [Statista Dating Services](https://www.statista.com/outlook/emo/dating-services/worldwide) · [IBISWorld US Dating Services](https://www.ibisworld.com/united-states/market-size/dating-services/1723/) · [GetStream app revenue](https://getstream.io/blog/dating-app-statistics/) · [Marketdata / marketresearch.com](https://blog.marketresearch.com/american-singles-fuel-the-2.5-billion-dating-market) · [ICF 2025 Global Coaching Study](https://coachingfederation.org/resource/2025-icf-global-coaching-study-executive-summary/)

**Demand & population:** [Pew key findings](https://www.pewresearch.org/short-reads/2023/02/02/key-findings-about-online-dating-in-the-u-s/) · [Pew full report](https://www.pewresearch.org/internet/2023/02/02/from-looking-for-love-to-swiping-the-field-online-dating-in-the-u-s/) · [Census unmarried Americans](https://www.census.gov/newsroom/stories/unmarried-single-americans-week.html) · [Forbes Health burnout](https://www.forbes.com/health/dating/dating-app-fatigue/) · [zipdo Gen Z](https://zipdo.co/gen-z-dating-statistics/) · [GRASS 2026](https://grass.camp/en-US/blog/gen-z-friendship-revolution-2026)

**Incumbent financials:** [Match Group Q1 2026](https://www.prnewswire.com/news-releases/match-group-announces-first-quarter-results-302763209.html) · [BigGo MTCH detail](https://finance.biggo.com/news/US_MTCH_2026-05-05) · [Yahoo/Zacks MTCH](https://finance.yahoo.com/markets/stocks/articles/match-group-q1-2026-earnings-145613629.html) · [Bumble Q1 2026](https://ppc.land/bumbles-paying-users-fall-21-but-profits-jump-165-in-q1-2026/) · [Bumble layoffs](https://www.foxbusiness.com/technology/bumble-announces-major-layoffs-affecting-30-employees-company-restructures) · [Bumble sale exploration](https://www.inc.com/georgia-fearn/bumble-explores-a-sale-amid-user-slump/91365975)

**AI in dating:** [Singles in America 2025](https://match.mediaroom.com/2025-06-10-Match-and-The-Kinsey-Institute-Unveil-14th-Annual-Singles-in-America-Study) · [IU News](https://news.iu.edu/live/news/34137-singles-in-america-study-daters-breaking-the-ice) · [Psychology Today](https://www.psychologytoday.com/us/blog/a-funny-bone-to-pick/202506/ai-use-in-dating-jumps-333) · [Scientific American chatfishing](https://www.scientificamerican.com/article/the-rise-of-ai-chatfishing-in-online-dating-poses-a-modern-turing-test/) · [Global Dating Insights](https://www.globaldatinginsights.com/featured/chatfishing-becomes-increasingly-common-among-singles/) · [Luxy dealbreaker survey](https://millionairedating.onluxy.com/ai-generated-dating-messages-dealbreaker.html) · [Hinge Prompt Feedback](https://hinge.co/newsroom/prompt-feedback) · [TechCrunch Hinge AI](https://techcrunch.com/2025/01/15/hinge-new-ai-feature-determines-if-your-prompt-response-is-too-basic/) · [Overtone / Fast Company](https://www.fastcompany.com/91574215/hinge-founders-new-dating-app-lets-ai-be-your-matchmaker-theres-already-a-waitlist-overtone)

**Competitors & pricing:** [Rizz / Forbes](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/) · [Rizz revenue estimate](https://trendapps.dev/app/android/com-rizzlabs-rizz/) · [AIbase case study](https://www.aibase.com/cases/156) · [YourMove](https://aitools.inc/tools/yourmove-ai) · [Texting Wingman](https://apps.apple.com/jm/app/texting-wingman-rizz-assistant/id6737630517) · [WRizz](https://apps.apple.com/us/app/wrizz-texting-coach/id6743861531) · [Roast](https://roast.dating/) · [Roast review](https://blog.photofeeler.com/roast-dating-review/) · [VIDA Select review](https://blog.photofeeler.com/vida-select-review/) · [Tawkify cost](https://www.vidaselect.com/tawkify-cost) · [Fiverr dating coaching](https://www.fiverr.com/gigs/dating-coaching) · [Dating coach rates](https://www.ziprecruiter.com/Salaries/Dating-Coach-Salary)

**Marketplace economics:** [Fiverr/Upwork fees 2026](https://www.jobbers.io/fiverr-vs-upwork-vs-freelancer-vs-jobbers-complete-comparison-2026/) · [Cameo 25% fee](https://talkspresso.com/blog/why-cameo-is-dying) · [Cameo collapse](https://startups.co.uk/news/what-happened-to-cameo/) · [Cameo insolvency](https://techstartups.com/2024/07/26/cameo-a-unicorn-tech-startup-once-valued-at-over-1-billion-is-now-broke-and-cant-pay-a-600000-fine/) · [Chegg / Forbes](https://www.forbes.com/sites/petercohan/2025/10/29/chegg-stock-down-99-learn-whether-ai-45-layoffs-make-chgg-a-buy/) · [Chegg layoffs](https://www.finalroundai.com/blog/chegg-layoffs-2025) · [JustAnswer model](https://www.sharetribe.com/create/how-to-build-website-like-justanswer/) · [JustAnswer expert earnings](https://wallethacks.com/justanswer-review/) · [Clarity.fm shutdown](https://talkspresso.com/blog/clarity-fm-alternatives-2026) · [AskMatch](https://techcrunch.com/2019/05/14/match-now-offers-dating-coaches-who-help-its-members-with-profiles-dating-challenges/) · [Leakage benchmarks](https://www.hokodo.co/resources/how-to-prevent-disintermediation-on-your-b2b-marketplace) · [Take-rate benchmarks](https://origami-marketplace.com/en-gb/marketplace-take-rate-a-guide-for-marketplace-operators/) · [App retention benchmarks](https://www.businessofapps.com/data/dating-app-benchmarks/)

**Legal & regulatory:** [Apple App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/) · [Epic ruling / MacRumors](https://www.macrumors.com/2025/12/11/apple-app-store-fees-external-payment-links/) · [Epic ruling / TechCrunch](https://techcrunch.com/2025/05/02/apple-changes-us-app-store-rules-to-let-apps-redirect-users-to-their-own-websites-for-payments) · [NY GBL §394-c](https://www.nysenate.gov/legislation/laws/GBS/394-C) · [CA Civ. §1694](https://codes.findlaw.com/ca/civil-code/civ-sect-1694/) · [FTC negative option / Gibson Dunn](https://www.gibsondunn.com/ftc-restarts-negative-option-rulemaking-after-eighth-circuit-vacatur-enforcement-under-rosca-continues/) · [FTC Negative Option Rule](https://www.ftc.gov/legal-library/browse/rules/negative-option-rule) · [Utah SB48 / KSL](https://www.ksl.com/article/51274140/new-law-aims-to-crack-down-on-unlicensed-life-coaches-practicing-mental-health-therapy) · [Coaching vs therapy legal distinction](https://www.goodtherapy.org/blog/Psychotherapy-vs-Coaching-Legal-Distinction) · [Dating app GDPR](https://gdprlocal.com/privacy-dating-sites-and-apps/) · [EFF on dating app consent](https://www.eff.org/deeplinks/2025/07/dating-apps-need-learn-how-consent-works) · [Tinder Terms of Use](https://policies.tinder.com/terms) · [IRS 1099-K threshold](https://irs.gov/newsroom/irs-issues-faqs-on-form-1099-k-threshold-under-the-one-big-beautiful-bill-dollar-limit-reverts-to-20000)
