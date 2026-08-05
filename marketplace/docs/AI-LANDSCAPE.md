# Wing — The AI Lane: Saturation, the Agentic Question, and What to Take From It

**July 2026 · Researched assessment of building the AI version instead**

Three questions, answered in order: **how saturated is the AI dating-assistant market**, **would an agentic app with a thread per match be a better product**, and **what should Wing take from the answer**.

Short version: the category is brutally saturated and the incumbents are now shipping the feature set for free inside their own apps, so building it is a bad business. But the *architecture* the question proposes — a thread per match with aggregated context — is right, and it belongs in the human product, pointed at the coach rather than the dater. That single change is the strongest fix available for the labour-cost ceiling identified in `MARKET-ANALYSIS.md` §34.

**This document also corrects two things I got wrong earlier.** They are in §6, not buried.

---

# PART I — HOW SATURATED

## 1. The roster

These all exist today, all consume a screenshot and return a reply, and all were surfaced by ordinary searching — this is not an exhaustive census:

**Rizz** · **YourMove AI** · **Roast** · **Ari** · **RIZON** · **Keys AI** · **PlugAI** · **FireTexts** · **Datemaxx** · **Smoothspeak** · **Winggg** · **WingAI** · **WingmanX** · **Wingman.live** · **Wingman: AI Dating Coach** · **Wingman AI: Texting Guide** · **Texting Wingman** · **WRizz** · **RizzGPT** · **RIZZ PRO** · **appwingman**

Twenty-plus named products. Six of them are called some variant of "Wingman." The broader market has **8,000+ dating apps** competing ([getstream](https://getstream.io/blog/dating-app-statistics/)).

## 2. There is an established leader, and it is bigger than I previously reported

| Rizz | Figure | Source |
|---|---|---|
| Total users | **7.5M** | [Forbes](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/) |
| Monthly active | **1.5M** | [Forbes](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/) |
| Revenue | **$15M+** | [youmind](https://youmind.com/landing/x-viral-articles/rizz-app-faceless-creator-growth) |
| Price | **~$7/week or ~$20/month** | [powerusers.ai](https://powerusers.ai/ai-tool/rizz-app/) |
| Rank | **Top-5 downloaded dating app in the US** | [Forbes](https://www.forbes.com/sites/josipamajic/2024/09/09/rizz-app-how-the-5th-most-downloaded-dating-app-is-redefining-digital-relationships/) |
| Growth channel | **Faceless creator networks** | [youmind](https://youmind.com/landing/x-viral-articles/rizz-app-faceless-creator-growth) |

Note that last row. Rizz's growth engine is *the same channel Wing's go-to-market depends on* — creator-driven short-form video — run at scale with a product that has zero marginal cost per answer. That is who you are bidding against for creator attention.

## 3. Demand for AI dating help is not the constraint

AI adoption among daters, tracked across two surveys:

| Period | Share of singles using AI in dating | Source |
|---|---|---|
| 2024 | ~6% (implied by the 333% jump) | derived |
| 2025 | **26%** | [Match × Kinsey](https://match.mediaroom.com/2025-06-10-Match-and-The-Kinsey-Institute-Unveil-14th-Annual-Singles-in-America-Study) |
| 2026 | **54%** | [getstream](https://getstream.io/blog/dating-app-statistics/) |

Adoption roughly doubled again. **The problem with this market is not that nobody wants the product. It is that everybody is selling it.**

---

# PART II — THE DECISIVE FACT

## 4. Both major incumbents are shipping this, bundled

This is the finding that settles the question, and it is new since the market analysis was written.

**Match Group is rolling AI "wingmen" to both Tinder and Hinge** — to "help users select their most appealing pictures, write messages to matches, and offer *effective coaching for struggling users*" ([AllSides](https://www.allsides.com/story/technology-hinge-and-tinder-roll-out-ai-wingmen-help-users-create-profiles-message-matches), [Vice](https://www.vice.com/en/article/tinder-users-can-now-use-an-ai-wingman/)). Tinder, Bumble and Hinge all launched major AI features in early 2026. Hinge's AI Core Discovery Algorithm has reportedly lifted matches and contact exchanges **15%** since March 2025.

Read that feature list again: photo selection, message writing, coaching for struggling users. **That is Wing's entire product catalogue, free, inside the app where the customer already is.**

**Grindr is building the per-match memory product specifically.** Announced October 2024, its AI Wingman will "keep track of each user's matches and make tailored suggestions based on conversations," help users track conversations with their favourite people, and suggest places to meet. Testing reached **10,000 US users ahead of schedule**, with rollout to Grindr's full **14 million** by 2027 ([PinkNews](https://www.thepinknews.com/2024/10/14/grindr-announces-plans-for-ai-wingman-feature/), [Yahoo/Them](https://www.yahoo.com/tech/grindr-upcoming-ai-wingman-chatbot-171544661.html)).

**And Grindr found the price ceiling nobody knew existed.** Its EDGE tier pilots at **$80/week, $349.99/month and $499.99/month** — up to ~$6,000/year — on top of an Unlimited tier at $27.99–44.99. Morgan Stanley upgraded the stock to Overweight citing EDGE, projecting **18% revenue CAGR through 2028 with the new products driving ~60% of growth** ([EDGE Media](https://www.edgemedianetwork.com/story/163015/grindr-launches-pilot-for-ai-powered-edge-subscription-tier-priced-up-to-500), [PinkNews](https://www.thepinknews.com/2026/02/09/grindr-trials-premium-500-per-month-plan-to-become-ai-first-app/), [BigGo](https://finance.biggo.com/news/3405af1a-2330-4461-bdc3-43548d87d13b)).

Two conclusions from that last one, pulling in opposite directions. It proves **willingness to pay for dating help runs far above the $20/month the AI apps assumed** — which supports going up-market, not down. And it proves the incumbents will capture that willingness themselves, from inside the app, with the match data already in their database.

## 5. The strategic inversion

The instinct is that incumbents entering makes this market more attractive — validation. It is the opposite, and the direction matters:

> **Match Group and Grindr shipping free AI assistance commoditises the machine answer. That is the strongest argument against building the AI version, and simultaneously the strongest argument for the human one — because when a competent generated reply is free and built in, the only scarce thing left is judgment somebody will stake their name on.**

You cannot win "a good reply, instantly, for cheap" against a company that owns the conversation, the match data, and the distribution, and can bundle it at zero marginal price. You can still win "a person who read your specific situation and told you something a machine won't."

---

# PART III — THE AGENTIC / PER-MATCH-THREAD IDEA, ASSESSED

## 6. It is a genuinely better product than what the category ships

Credit where due: a thread per match, holding aggregated context — who she is, what's been said, what was advised, what happened — **is a real improvement over the entire screenshot→reply category**, which is stateless. Every one of those twenty apps forgets your match the moment you close it. Memory is the obvious next step and the reason the category monetises badly: there is nothing to retain a user with.

So the product intuition is correct. The problem is everything around it.

## 7. Four reasons not to build it as a business

**1 · It is already built, twice over, at both ends of the market.**

*Inside the apps:* Grindr's AI Wingman is precisely this, reaching 14M users by 2027, bundled.

*As standalone products:* a dating-CRM niche already exists —
- **MatchMGT** — "one profile per person, history, notes, smart reminders and an availability calendar… analyses WhatsApp chats with AI and pulls out interests, personality traits and concrete date ideas" ([matchmgt.com](https://matchmgt.com/blog/best-apps-to-organize-your-dating-life-2026/))
- **RosterNote** — personal CRM and dating tracker; paste a bio or chat log and AI extracts details to build profiles; keyword tracking, milestone reminders ([rosternote.com](https://rosternote.com/en))
- **DatingTrack: Note**, **Date Diary Tracker** — logging, calendars, per-person notes
- General personal CRMs (Monica, Clay, Dex) used for the same job

The gap you would be building into has two occupants already and a third arriving with 14 million users attached.

**2 · It puts you inside subscription economics you cannot win.**

AI apps convert better and retain worse, and the gap is measured:

| | AI apps | Non-AI apps |
|---|---|---|
| Trial-to-paid conversion | **+52%** | baseline |
| Realised LTV per user | **+39–41%** | baseline |
| **Annual retention** | **21.1%** | **30.7%** |

Annual subscription churn runs **~30% faster** for AI apps ([TechCrunch](https://techcrunch.com/2026/03/10/ai-powered-apps-struggle-with-long-term-retention-new-report-shows), [RevenueCat State of Subscription Apps 2026](https://www.revenuecat.com/state-of-subscription-apps)).

Now stack that on the category you would be in. Dating apps already have **the worst retention in mobile**: 7% day-30 activation, a 3.3% retention rate, and **fewer than 5% of monthly subscribers still active twelve months later** ([Business of Apps](https://www.businessofapps.com/data/dating-app-benchmarks/)). AI churn multiplied by dating churn is the worst retention profile available in consumer software. You would be buying users through the same creator channel as Rizz, at Rizz's prices, and losing them faster.

**3 · Real agency — an agent acting inside the app — gets your customer banned.**

Tinder's terms prohibit third-party services that interact with its Services or Member Content, "including artificial intelligence or machine learning systems" ([Tinder Terms](https://policies.tinder.com/terms)). And 2026 enforcement is not theoretical: platforms now model *how* you do things, flagging uniform intervals and UI navigation lacking human variance; browser-based automation "looks identical to a bot from the platform's perspective." LinkedIn alone blocked 78.2M fake accounts and flagged 23.5M automated sessions in one quarter.

The distinction is sharp and it is the whole ballgame. **A user manually sharing a screenshot is the same act as showing a friend. An agent reading and writing inside their account is a terms violation whose penalty lands on the customer, not on you.** You would be selling a product whose most impressive feature risks the account it runs on.

**4 · It abandons both of the only defensible assets.**

`MARKET-ANALYSIS.md` §22 scored every asset on VRIO and found exactly two candidate moats: the **defect-rate quality dataset** and the **trust brand**. An AI assistant has neither and can build neither — there is no coach whose accuracy you are measuring, and "we generate your messages" is the opposite of the trust position. You would trade two potential moats for a commodity with 20+ competitors, zero switching cost, and two incumbents giving it away.

## 8. What the numbers say about margin, since it is usually the counter-argument

The pitch for AI is zero marginal cost. That is less true than it sounds. At a $10/month ARPU with mid-tier model pricing, LLM cost runs about **$0.60/user/month → ~79% gross margin** — fine. But an agentic product with memory does retrieval, summarisation and multi-turn reasoning on every interaction, and **a $20/month user can generate $18–25 in inference during heavy reasoning**. GitHub Copilot lost roughly **$20/user/month** early on, with power users costing **$80 against a $10 subscription** ([TMH](https://the-marketinghub.com/blog/hidden-economics-ai-saas-2026/), [CRV](https://www.crv.com/content/llm-inference)).

Agentic is the expensive end of AI, sold to the worst-retaining category in mobile, against free bundled competition. The margin story is not the rescue.

---

# PART IV — WHAT TO ACTUALLY TAKE FROM THE IDEA

## 9. Keep the architecture. Change who it serves.

Here is the move, and it is the most valuable thing in this document.

**The per-match thread with aggregated context is right. Point it at the coach, not at the dater.**

Wing's binding constraint is not demand and not market size — it is the labour ceiling in `MARKET-ANALYSIS.md` §34: at $5 an answer and four answers an hour, Wing can only buy labour whose alternative wage is under about $20/hour, against an ICF US coach average of ~$119/hour effective. That arithmetic is what caps answer quality.

But look at where a coach's time actually goes on a repeat client. Not judgment — **reconstruction.** Who is this match, what was already tried, what did I tell them last time, did it work, what did they say about their job three weeks ago. The judgment is the fast part; the archaeology is the slow part.

**If the system assembles the context, the coach answers faster, and their effective hourly rate rises without the price rising.** Six answers an hour instead of four takes a coach from $18/hr to $27/hr at the same $5 price. That is the difference between "only sub-$20/hr labour" and "a competent freelancer's rate" — bought with software rather than with margin.

## 10. This is the validated split, not a novel bet

Hybrid coaching is a solved pattern outside dating. In executive and leadership coaching, the consistent finding is that **AI handles reflection, preparation and follow-up while the human contributes depth, judgment and accountability** — and that hybrid outperforms AI-only where behaviour change is the goal. Platforms like Coachello and Sharpist are built on exactly that division: AI transcription and between-session prep, human for the session ([Pinnacle](https://www.heypinnacle.com/blog/how-to-build-a-hybrid-ai-human-coaching-strategy-that-actually-scales), [Sharpist](https://www.sharpist.com/blog/ai-coach-or-human-coach), [Thought Leadership Institute](https://thoughtleadership.org/casting-a-vision-for-human-ai-hybrid-coaching-practice/)). The human's specific contribution is described as "reading the room and providing hard truths a model won't volunteer" — which is, word for word, The Call register in `UI-DESIGN.md` §4.7.

And within dating specifically, **Roast already runs the hybrid**: AI scoring plus a human expert review, priced $6.99 / $12.99 / **$97** — with the observation that the Expert tier is where the revenue sits, "because the coaching call and curated pictures are what moves the needle for someone who does not already know what to do" ([VIDA](https://www.vidaselect.com/roast-dating-review), [GetMatches](https://getmatches.ai/en/blog/roast-dating-review)). The AI is the funnel; the human is the product. That is the shape.

## 11. It does not violate the no-AI constraint — and the line is precise

`ARCHITECTURE.md` §13.1 permanently excludes AI features, and I wrote that citation. This proposal does not breach it, because the exclusion has a specific rationale: AI in the **answer path** collapses the trust brand, which is one of only two moats. The line to hold:

| | Allowed | Forbidden |
|---|---|---|
| **Who reads the output** | The coach | The dater |
| **What it produces** | A briefing: this match, this history, what was advised, what happened | Any part of an answer, opener, or suggested message |
| **Whose words reach a dater** | Only the coach's | — |
| **If it fails** | The coach notices and ignores it | A dater is chatfished by the platform |

**No generated text ever reaches a client or the person they are talking to.** The dater-facing promise — a human wrote this, and will tell you why — remains literally true. If that line is ever crossed, even quietly, the trust asset is gone and §7's fourth objection applies to Wing too.

This is a genuine amendment to §13.1 rather than a loophole, and it should be written as one: **AI is excluded from the answer path, permanently. Coach-side context assembly is a separate question with a different answer.**

## 12. What it would change in the build

A `matches` entity, which the product currently lacks entirely — today a thread is per *coach*, and everything about a specific person the client is dating lives loose in message history.

```
matches            (id, client_id, label, platform, created_at, archived_at)
match_context      (match_id, screenshot refs, coach notes, advice given, outcome)
```

- **Client-side:** a match is a lightweight label on screenshots and answers — "which person is this about?" No CRM, no dossier, no profile-building on a third party. Deliberately thin, for the §28 reasons: every screenshot contains a non-consenting person, and building a durable file on them is exactly what the privacy commitments forbid. **This is the hard constraint that separates Wing from MatchMGT and RosterNote, and it is not a limitation to engineer around — it is why Wing can be a trust brand and they cannot.**
- **Coach-side:** answering surfaces a brief — prior advice on this match, recorded outcomes, what the client said about themselves. Deterministic assembly first; summarisation only if the brief gets too long to skim.
- **It compounds with `answer_outcomes`** (added in the product revision). Per-answer outcomes tied to a match make the brief say *"you advised waiting two days; she replied"* — which is both better context and better ranking data.
- **Sequencing:** this is not an M1 change. It earns its place only once a coach has repeat clients, so **M3 at the earliest**, and only if bet 3 (repeat purchase) clears.

---

# PART V — CORRECTIONS TO THE EARLIER ANALYSIS

Two things in `MARKET-ANALYSIS.md` need fixing, and both matter for decisions.

## 13. I understated the AI category's revenue

§9 cited a Sensor Tower-derived estimate of **~300k monthly downloads generating ~$30k revenue, about $0.10 per download**, and used it to argue the category "monetises badly." That figure is **Android-only for one app**, and I presented it as more representative than it is. Rizz as a company is reported at **$15M+ revenue** on 7.5M users and 1.5M MAU.

The corrected read: the category monetises *thinly per user* — $7/week against enormous download volume — but it is not a failure to monetise. **The AI dating-assistant category is a real business at real scale.** That makes it a *worse* market to enter, not a better one: there is an established, well-capitalised leader using your intended growth channel.

## 14. Incumbent threat has gone from low to high — but only in one lane

§15 concluded the incumbent threat was low, on the evidence that Match tried human coaching with AskMatch in 2019 and abandoned it. That inference was sound for *human* coaching and is now clearly wrong for *AI* assistance. Match Group is shipping AI wingmen across Tinder and Hinge; Grindr is shipping per-match AI memory to 14M users and testing $500/month for it.

The corrected picture separates the two lanes, and the separation is the strategic conclusion of this whole document:

| Lane | Incumbent threat | Direction |
|---|---|---|
| **AI assistance** (replies, profile help, per-match memory) | **Severe** — bundled free by Tinder, Hinge and Grindr | Worsening fast |
| **Paid human judgment** (a named person, accountable, cross-platform) | **Low** — tried in 2019, abandoned, capital now committed elsewhere | Stable |

§15's actual argument survives intact and gets stronger: the industry's best talent and capital are betting on AI-mediated dating. What changes is that the bet is now visibly funded and shipping — which raises the cost of joining it and lowers the cost of standing apart from it.

---

# PART VI — VERDICT

## 15. The options, scored

Higher is better; 5 is best.

| | Saturation head­room | Defensibility | Retention profile | Incumbent risk | ToS + privacy risk | Unit economics | Fits existing assets | **Total** |
|---|---|---|---|---|---|---|---|---|
| **A · Pure AI assistant app** | 1 | 1 | 1 | 1 | 4 | 3 | 1 | **12** |
| **B · Agentic per-match app** | 2 | 2 | 1 | 1 | 1 | 2 | 1 | **10** |
| **B′ · Typed agentic threads, durable memory** | 2 | 3 | **4** | 1 | **1** | 2 | 1 | **14** |
| **C · Human marketplace as specified** | 5 | 3 | 3 | 4 | 5 | 4 | 5 | **29** |
| **D · Human marketplace + typed memory layer** | 5 | **4** | **4** | 4 | **4** | 4 | 5 | **30** |

**A, B and B′ score badly for the same underlying reason:** they compete on output in a category where output is becoming free, against companies that own the distribution and the data. B and B′ score lowest on ToS and privacy because agency inside a dating app is what platforms are actively hunting in 2026, and because an accumulating dossier on a non-consenting third party is what fails the GDPR balancing test (§21).

**B′ is the honest best version of the AI concept**, and it scores meaningfully above B on retention — memory is the one real answer to the churn problem (§18). It still loses, because retention fixes the leak and not the acquisition, and acquisition is the binding constraint: the substitute is ChatGPT Projects (§19), not the rizz apps.

**D beats C on two axes, and D now means the typed layer from §22.** Defensibility improves because per-client advice-and-outcome history is a dataset a competitor cannot copy or buy — the defect-rate moat at higher resolution. Retention improves because accumulated context is genuine switching cost: a coach who already knows your situation is worth more than a new one, and re-explaining is the real cost of leaving. D's privacy score is 4 rather than 5 only because it introduces memory at all; typing by data subject is what keeps it there.

## 16. The answer, in three sentences

**Do not build the AI version.** The category has 20+ competitors, an established $15M leader using your intended growth channel, the worst retention profile in consumer software, and — decisively — Tinder, Hinge and Grindr are all shipping the feature set bundled and free.

**Do not build the agentic version.** Grindr is shipping it to 14M users by 2027, MatchMGT and RosterNote already ship the standalone form, and genuine agency inside a dating app is a terms violation that gets your customer banned rather than you sued.

**Do take the architecture.** A thread per match with aggregated context is the right structure — pointed at the coach as a briefing, never at the dater as generated text. It is the strongest available fix for the one constraint that actually binds this business: at $5 an answer, the only way to afford better judgment is to stop paying a human to do archaeology.

> **Refined in Part VII.** The typed-agentic-thread version of the concept — durable memory documents, one context per thread type — is a genuinely better idea than the category ships and is the pattern that won in agent engineering. It still shouldn't be a standalone product, for a reason Part III missed: **the substitute is ChatGPT Projects, not Rizz.** But it should be Wing's memory layer, **typed by whose data accumulates rather than by task** (§22) — because the durable, valuable, legally clean document was never the one about the match. It's the one about the client.

---

---

# PART VII — THE TYPED AGENTIC THREAD, ASSESSED PROPERLY

## 17. Why there is a Part VII

Parts I–VI assessed the AI dating-assistant *market* and a generic per-match-thread product. The concept was then stated more precisely: **each thread is an agentic thread with its own durable memory — a document that accumulates — and there are distinct thread types, each carrying its own context.**

That is a different proposal from the one Part III benchmarked, and it warrants its own assessment rather than an inherited verdict. Part VII does that. It reaches the same conclusion by a better argument, finds one competitive fact Part III missed entirely (§19), and produces the one design change that improves the whole system (§22).

## 18. This is a materially better idea than what Part III assessed

Part III benchmarked the *category* — twenty stateless screenshot→reply apps. The architecture described here is not that, and it deserves separate assessment, because it is the pattern that actually won in agent engineering.

The dominant memory design in production coding agents in 2026 is exactly this: **a markdown file injected into context at session start**, with agents reading and writing it explicitly. OpenClaw keeps `MEMORY.md` for durable facts plus `memory/YYYY-MM-DD.md` daily notes, exposing `memory_search` and `memory_get` over them, and runs an automatic "memory flush" before compaction so nothing important is lost — with an optional consolidation pass promoting short-term notes into long-term memory. LangChain's documented split of **short-term thread-scoped state from long-term cross-thread stores** is described as "not an advanced pattern; it's the standard starting point" ([Zylos](https://zylos.ai/research/2026-04-05-ai-agent-memory-architectures-persistent-knowledge/), [agent-memory](https://github.com/Defiladeboarfish90/agent-memory), [Red Hat](https://next.redhat.com/2026/06/01/from-context-to-dreams-architecting-memory-for-ai-agents/)).

So the instinct is architecturally mainstream and correct. Typed threads with their own context, durable accumulating memory, two tiers — that is how this is built.

**And it targets the category's actual cause of death.** Part III's strongest objection was retention: AI apps churn ~30% faster, 21.1% vs 30.7% annual, stacked on dating's sub-5% twelve-month survival. Memory is the **only known antidote**, because accumulated state is switching cost. A user six months into a thread that knows their history does not restart elsewhere. Every one of those twenty apps is disposable precisely because it remembers nothing. That objection is genuinely weakened by this design, and I should say so plainly rather than restate the earlier conclusion.

Three things still bite. The third one changes the design rather than killing it.

## 19. Constraint 1 — the substitute is ChatGPT Projects, not Rizz

This is the competitive fact that matters, and it is not in Part I.

**ChatGPT Projects are persistent workspaces grouping related chats, files, and custom instructions, with context that persists session to session.** Project memories are isolated from global memories. Users can view, edit or delete individual memory entries. Live since December 2024 and fully rolled out ([Suprmind](https://suprmind.ai/hub/chatgpt/features/), [DataStudios](https://www.datastudios.org/post/can-chatgpt-remember-previous-conversations-memory-behavior-session-limits-and-persistence)).

Read that against the concept: *typed threads, each with its own context, accumulating durable memory, user-inspectable.* **A ChatGPT Project per match, with a dating-specific instruction block, is approximately the described product** — already built, already in the hands of hundreds of millions of people, at $20/month, with file upload and image understanding included.

So the differentiation question is not "is this better than the twenty rizz apps" — it plainly is. It is **"what does this do that a ChatGPT Project with a good system prompt does not?"** The honest answers are narrow: a purpose-built schema per thread type, a mobile capture flow tuned for screenshots, and defaults a civilian will never configure themselves. Those are real, and they are a *product*, not a moat — the same list describes every ChatGPT wrapper that has been commoditised in the last two years.

## 20. Constraint 2 — this architecture's cost curve bends the wrong way

Accumulating memory into a document you inject is cheap early and expensive exactly when you succeed.

| Memory size | In-context cost per query | Retrieval-based |
|---|---|---|
| ~7,000 facts | **$0.57** | $0.002 |
| ~100,000 facts | **$8+** | $0.002 |

Cost scales linearly with window size, while retrieval stays flat regardless of corpus ([arXiv 2603.17781](https://arxiv.org/pdf/2603.17781)). Worse, naive agent loops append tool output to history every iteration, producing a **triangular series where a 10-step run re-bills every prior step** ([Augment](https://www.augmentcode.com/guides/ai-agent-loop-token-cost-context-constraints)). Agentic memory systems scale **super-linearly**, diverging steeply past 256K tokens.

Run it against a $7/week price:

- **Month 1** — 10k tokens of context per query, ~$0.0125/query. 100 queries = **$1.25/month.** Comfortable.
- **Month 12, engaged user** — six match threads, accumulated memory, 7,000-fact scale. At $0.57/query, 100 queries = **$57/month against a $30 subscription.**

That is the GitHub Copilot failure mode with a name on it: losing ~$20/user/month, power users at $80 against a $10 plan. **Your best users become your biggest losses, and they are the ones memory retains.**

It's solvable — structured distillation reports **11× token reduction with retrieval preserved**, and token-optimisation practice claims 3–4× cuts ([arXiv 2603.13017](https://arxiv.org/pdf/2603.13017), [mem0](https://mem0.ai/blog/the-2026-token-optimization-playbook-cut-ai-agent-memory-costs-3%E2%80%934x)). But note what that means: **retrieval and distillation are not optimisations you add later, they are the product.** The naive md-file-injection version works for a demo and breaks in month twelve. And context drift is the other side of the same coin — roughly **65% of enterprise AI failures in 2025 were attributed to context drift or memory loss** in multi-step reasoning, so unbounded accumulation degrades answer quality even where you can afford it.

## 21. Constraint 3 — the memory is about someone who never consented, and *accumulation* is the legal variable

This is the one that should change the design, and it is specific to this domain rather than to AI.

In coding, `MEMORY.md` accumulates facts about *your codebase*. Here, a match thread's memory document accumulates facts about **a real person who does not know the file exists** — her job, her dog, what she said about her ex, how she responds to being teased, her stated boundaries.

GDPR permits profiling without consent under Article 6(1)(f) legitimate interests, but only via a balancing test — and the EDPB's stated factors are **"the level of detail of the profile, the comprehensiveness of the profile, and the impact of the profiling on the data subject"** ([EDPB Guidelines 1/2024](https://www.edpb.europa.eu/system/files/2024-10/edpb_guidelines_202401_legitimateinterest_en.pdf), [IAPP](https://iapp.org/news/a/wp29-releases-guidelines-on-profiling-under-the-gdpr)). Data subjects hold an Article 21 right to object. Separately, a lawful basis for *collecting* does not extend to *disclosing* — disclosure is its own processing act requiring its own basis.

So the legal exposure is not a function of using AI. **It is a direct function of how much the document accumulates.** A thin, decaying note plausibly passes the balancing test. A comprehensive dossier that deepens for months is precisely what fails it — and depth is the feature.

This is also why the incumbent can do it and you cannot do it the same way. **Grindr owns the platform; both people in the conversation agreed to its terms.** A third-party app building durable files on non-users has no such footing. MatchMGT and RosterNote do it anyway — AI-extracted profiles with "interests, personality traits," keyword tracking on people's posts. That is a posture, not a precedent, and it is incompatible with being a trust brand.

## 22. The reframe: type threads by whose data accumulates, not by what task they do

Here is the fix, and it makes the architecture *better* rather than merely legal.

The instinct is to type threads by task — match thread, profile thread, photo thread. Type them instead by **data subject**, and let that determine the memory policy:

| Thread type | Subject | Memory policy | What accumulates | What never does |
|---|---|---|---|---|
| **You** (one per user, permanent) | The client | **Accumulate freely, forever** | Their voice and phrasing, what advice worked, recurring patterns, goals, dealbreakers, what they're bad at | — |
| **Artifact** (bio, photos, profile) | The client's own content | **Accumulate freely** | Version history, what was changed and why, results per version | — |
| **Match** (one per person) | **A third party** | **Thin, capped, decaying** | Situation state, advice given, outcome. Facts only as needed for the current decision | Personality assessments, inferred traits, anything about their appearance, anything not needed to answer the question in front of you |
| **Debrief** (post-date) | Mixed | **Client-side only** | What the client learned about themselves | Anything about the other person beyond "it ended" |

Three things follow, and the first is the important one.

**The valuable memory was never the match anyway.** A dossier on one woman is worth something for a few weeks and then she is gone. A document that knows *how this client writes, what they always get wrong, which advice has actually worked for them, and what they are looking for* compounds for years and travels across every match they ever have. **The client-subject thread is the high-value asset, the durable one, and the one with no third-party problem at all.** Typing by subject doesn't sacrifice the good part to satisfy the lawyer — it identifies which part was good.

**It solves the cost curve in the same move.** Match threads are the ones that would multiply — five, ten, thirty over a year, each accumulating. Capping them bounds the expensive dimension. The `You` thread grows, but there is exactly one per user and structured distillation applies cleanly to it.

**And it becomes the marketable difference.** Against ChatGPT and against MatchMGT alike, the claim is: *your file is about you, not about them; you can read it, edit it, export it, delete it.* That is user-owned, human-readable, portable memory as the trust feature — the one thing neither an incumbent bundling AI nor a general assistant will offer, and it is only credible because of the constraint, not despite it.

## 23. What this means for Wing concretely

The typed-thread pattern is a **better version of the `matches` proposal in §12**, and it maps onto Wing's existing model more cleanly than a generic per-match table.

Wing already sells typed products — opener rescue, convo rescue, bio makeover, photo verdicts, screening second opinion, date plan. **Each product type is a thread type with its own context schema.** That was already latent in the design; the memory policy is what was missing.

```
threads/
  you/<client_id>/MEMORY.md          durable, grows, client-subject, coach-readable
  artifact/<item_id>/NOTES.md        version history of their own bio/photos
  match/<match_id>/STATE.md          capped, decaying, situation + advice + outcome
```

- **The `You` document is the coach briefing.** This is the answer to §9's labour-ceiling fix, made concrete: the thing that raises a coach's effective hourly rate is not a dossier on the match, it is a one-page file on the client that means the coach never re-reads three weeks of history. Assembled deterministically first; distilled only when it outgrows a skim.
- **`answer_outcomes` is the input that makes it worth reading.** Per-answer outcomes turn the file from notes into evidence: *"you advised waiting two days; she replied."*
- **The AI line in `ARCHITECTURE.md` §13.1 holds unchanged.** These documents are read by the coach. No generated text reaches a dater. The memory is infrastructure for human judgment, not a substitute for it.
- **Still M3, still gated on bet 3.** None of this matters until repeat purchase is proven, because memory has no value without a second visit.

## 24. The honest steelman, and where it lands

The strongest standalone version of this concept is not "an AI that writes your replies with memory." It is:

> **A single durable, human-readable document about you and your dating life — that you own, can edit, and can hand to a person.**

That framing beats ChatGPT Projects on portability and trust, beats the rizz apps on everything, and side-steps the third-party problem by construction. It is a genuinely good product idea.

But notice where it arrives. A document about you, maintained by software, **whose highest-value use is being read by someone who can then tell you something true** — that is Wing with a better context layer. The architecture's best expression is the hybrid: **the memory document is the interface between the machine that assembles and the human who judges.**

So the verdict from Part VI stands, with the reasoning upgraded rather than repeated. Do not ship this as a consumer AI product — ChatGPT Projects is the incumbent substitute, the cost curve punishes your best users, and the match-dossier version is the part that fails the GDPR balancing test. **Do build it as Wing's memory layer, typed by data subject, with the `You` thread as the asset.** That is the version where the accumulation is legal, cheap, compounding, and worth paying a human to read.

---

## 25. Concrete next steps

1. **Amend `ARCHITECTURE.md` §13.1** to state the AI line precisely: excluded from the answer path permanently; coach-side context assembly is a separate decision, gated on M3 and on bet 3 clearing. *(Done.)*
2. **Type threads by data subject, not by task** (§22). Three memory policies: `You` accumulates freely, `Artifact` accumulates freely, `Match` is capped and decaying. This is what replaces the generic `matches` table proposed in §12 — the privacy commitments in §9.2 are load-bearing and this is how they're enforced in the schema rather than in a policy document.
3. **Correct §9 and §15 of `MARKET-ANALYSIS.md`** per §13–14 above. *(Done.)*
4. **Re-run the price question with Grindr's data in hand.** EDGE at $349–500/month against Tinder's $17.56 RPP says the ceiling for dating help is set by positioning, not by category. This strengthens both the up-market pivot option and the $8–10 answer test already recommended.
5. **If the memory layer is ever built, build retrieval and distillation from day one** (§20). They are not optimisations — naive accumulate-and-inject works for a demo and loses money on your best users by month twelve.
6. **Leave bet 1 first.** None of this changes the ordering: if a human answer does not measurably beat a machine answer on real threads, the human marketplace is the wrong business and no architecture rescues it.
