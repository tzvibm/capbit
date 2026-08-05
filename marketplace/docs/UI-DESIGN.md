# Wing — UI Design Specification

**Version 2.1 · July 2026 · The visual and interaction reference**

Companion to `ARCHITECTURE.md` (how it's built) and `flows-and-wireframes.html` (every path). This document is what the product **looks and feels like**, screen by screen, in wireframe.

---

## 0.1 What changed in v2.1, and why

Three interface changes, each traceable to a finding in `docs/MARKET-ANALYSIS.md`. The reassessment that produced them — including the longer list of things it deliberately left alone — is `docs/PRODUCT-REVISION.md`.

1. **The Call gets its own register** (§4.7). An answer whose advice is *send nothing* no longer renders as an answer with its payload missing. Positive spine, dashed block, no Copy. It is the one output no generator will produce, so it should not look like a degraded version of the normal case.
2. **Outcome capture on the answer card** (§4.8). One tap: *Replied · No reply · Didn't send*. Reviews rate whole items; the defect-rate ranking and the human-vs-AI question both need per-answer signal.
3. **Discover leads with urgency, not category** (§3). "What's happening?" over "Who do you need?", because the situations that carry a deadline are the ones that convert — and category-first browsing buries them.

Nothing else in this document changed. In particular the answer card already put reasoning before payload, which is the right shape for selling a read rather than a reply, and *Privacy & data* was already prominent rather than buried.

---

## 0. What changed in v2, and why

Three problems drove this revision.

**1. A paid answer looked like a chat bubble.**
The whole reason to pay a human $4 is the answer — and we rendered it as a message that scrolls into oblivion. You can't copy it cleanly, can't find it later, can't feel what you bought. **Paid answers are now cards**: visually distinct, with a copy-ready text block, a save action, and a label saying what they cost. Free chat stays as bubbles. The paid/free split becomes visceral instead of a badge you have to read.

**2. The inbox was a list of conversations when it should be a home.**
It answered "who did I talk to" but not "what needs me right now" or "what have I bought." A client with a delivered bio awaiting approval and a review prompt pending had to go hunting. The tab is now **Home**: a *Needs you* strip first, then active conversations, then your coaches.

**3. There was no way back to a coach you liked, and no way back to advice you paid for.**
Two additions fix this: **favourites** (one tap to return to a coach) and **Saved** (a library of every answer you've bought). Saved is the reason the app stays installed — you're mid-conversation in Hinge, you open Wing, you copy the opener Maya wrote. Two taps.

---

## 1. Information architecture

### Client mode

```
┌──────────┬──────────┬──────────┬──────────┐
│   Home   │ Discover │  Saved   │   You    │
│    ⌂     │    ⌕     │    ♡     │    ⊙     │
└──────────┴──────────┴──────────┴──────────┘
```

| Tab | Answers the question | Contains |
|---|---|---|
| **Home** | What needs me? Who am I working with? | Needs-you strip · active threads · your coaches · past |
| **Discover** | Who can help with this? | Search, filters, coach cards, your favourites row |
| **Saved** | What did I pay for? | Every answer and deliverable, filterable, copyable |
| **You** | Everything about me | Profile, purchases & receipts, settings, *Switch to coach mode* |

**What moved:** *Purchases* was a top-level tab — too prominent for something you check monthly. It now lives under **You**. *Saved* takes its place, because it's used weekly.

### Coach mode (toggled from **You**)

```
┌──────────┬──────────┬──────────┬──────────┐
│  Queue   │   Menu   │ Earnings │   You    │
│    ⚑     │    ☰     │    $     │    ⊙     │
└──────────┴──────────┴──────────┴──────────┘
```

Mode is a persistent toggle, not a separate account. The switch lives at the top of **You** and the whole tab bar changes; a coloured hairline at the top of the screen marks coach mode so you always know which side you're on.

---

## 2. Home — the new main tab

```
┌─────────────────────────────────────┐
│  Tuesday                         ⚙  │
│  Your coaching                      │   ← serif, the one voice moment
├─────────────────────────────────────┤
│  NEEDS YOU · 2                      │   ← only rendered when non-empty
│  ┌───────────────────────────────┐  │
│  │ ✓  Approve your bio makeover  │  │
│  │    Dev K. · delivered 1h ago  │ ▸│
│  │    $30 releases on approval   │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ★  Rate your 5 custom openers │  │
│  │    Maya R. · completed today  │ ▸│
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  ACTIVE                             │
│  ┌───────────────────────────────┐  │
│  │ (M) Maya R.        6 answers  │  │
│  │     Maya: how'd the pigeon…   │  │
│  │                        2m  ●  │  │   ● = unread
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ (S) Sam T.      2 of 4 photos │  │
│  │     You: [screenshot]     1h  │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  YOUR COACHES                       │
│   ♥(M)   ♥(D)   ♥(J)    ⊕ find      │   ← horizontal, tap = profile
│   Maya    Dev     Jo                │
├─────────────────────────────────────┤
│  PAST                            ⌄  │   ← collapsed by default
└─────────────────────────────────────┘
```

**Rules**

- **Needs you** is the whole point of the screen. It contains only items with a pending client action: approve a delivery, rate a completed item, accept an offer, a late item offering a refund, a failed payment. Each row states the consequence (`$30 releases on approval`). Empty → the section disappears entirely, never shows "nothing to do."
- **Active** = threads with an open item or a message in the last 14 days. The right-hand meter is the thread's state at a glance (`6 answers`, `2 of 4 photos`, `delivered`, `Fri 7:00 PM`).
- **Your coaches** = favourites first, then coaches you've bought from. One tap goes to their profile with the menu, not the thread — because returning usually means buying again.
- **Past** collapses everything settled and quiet. Tapping expands; threads never disappear.

**Empty state (new user):**

```
┌─────────────────────────────────────┐
│  Your coaching                      │
│                                     │
│      Nothing here yet.              │
│      Find someone who's good at     │
│      the part you're stuck on.      │
│                                     │
│      [  Browse coaches  ]           │
│                                     │
│  Popular right now                  │
│  ┌───────────────────────────────┐  │
│  │ (M) Maya R.   from $4/answer  │  │
│  │     ★4.9 · first answer free  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

An empty Home immediately becomes a discovery surface. Never a dead end.

---

## 3. Discover — with favourites

```
┌─────────────────────────────────────┐
│  tell us where you are               │
│  What's happening?                  │   ← was "Who do you need?"
│  ┌──────────────┐┌──────────────┐   │
│  │They're       ││Going quiet   │ → │   ← urgency row, scrolls
│  │waiting       ││Stalled mid-  │   │
│  │Replied, ball ││conversation  │   │
│  │in your court ││              │   │
│  └──────────────┘└──────────────┘   │
│  ┌───────────────────────────────┐  │
│  │ ⌕  Search coaches…            │  │
│  └───────────────────────────────┘  │
│  [Openers][Bio][Photos][Convos][⋯]  │   ← demoted to secondary
├─────────────────────────────────────┤
│  YOUR COACHES                       │
│   ♥(M) Maya   ♥(D) Dev   ♥(J) Jo    │   ← only if favourites exist
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐  │
│  │ (M) Maya R. ✓         from $4 │♥ │   ← heart, top-right, tappable
│  │     Opener & conversation     │  │
│  │     specialist                │  │
│  │     ★4.9 (212) · ~5 min       │  │
│  │     0.3% refunds · 71% return │  │   ← the un-buyable signals
│  │     [Openers][Convos][1st free]│ │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ (D) Dev K. ✓         from $30 │♡ │
│  │     Bio doctor · ex-copywriter│  │
│  │     ★4.8 (167) · ~2 hr        │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**Urgency comes first, category second.** The five entries are situations in the client's words, ordered by how fast they need an answer:

| Entry | Means | Maps to | Sort |
|---|---|---|---|
| **They're waiting** | Replied, ball's in your court | conversations | fastest responder first |
| **Going quiet** | Stalled mid-conversation | conversations | fastest responder first |
| **New match** | Haven't written yet | openers | fastest responder first |
| **Date coming up** | Planning or deciding | dates | fastest responder first |
| **No deadline** | Profile, bio, photos | — | default ranking |

Selecting one sets the category filter *and* re-sorts by response time, because a waiting match is a deadline and a coach who replies in 5 minutes is genuinely better for it than one who replies in 4 hours. Tapping a category chip clears the urgency selection — the two are alternative ways in, not a compound filter, and stacking them would strand people in empty results.

**Why the primary question changed.** JustAnswer is the one per-question expert marketplace that works at scale, and its categories all carry a deadline and a consequence: my dog swallowed something, at 11pm. Clarity.fm, which sold open-ended expert calls, shut down in 2022. Dating is high-emotion but low-urgency — nothing bad happens if you wait, ask a friend, or send nothing — so the product has to lead with the moments that *do* have a clock on them. "Improve my profile" has no deadline and will always lose to procrastination; "they're waiting" cannot be postponed. Asking *what's happening* instead of *which category* is how the interface finds the urgent case. `urgency_selected` is instrumented so this is falsifiable: if deadline-bearing entries don't convert better, revert to the category-first row.

**Favouriting:** tap the heart on a card or profile. Filled = favourited. It's **private** — the coach is never told, so it stays an honest bookmark. Favourites surface in three places: the Discover row, the Home *Your coaches* strip, and as a filter (`♥ Favourites` chip).

**The stat line matters more than the stars.** Refund rate and return rate are the signals a coach cannot buy (see `ARCHITECTURE.md` §9.7). Showing them on the card is what makes the ranking legible rather than mysterious.

---

## 4. The thread — the overhaul

### 4.1 The idea

The thread now has **three visual registers**, and the difference is instantly readable:

| Register | Looks like | Is |
|---|---|---|
| **Message** | plain bubble | free conversation, either side |
| **Answer** | bordered card, accent spine, actions | the thing you paid for |
| **Item card** | neutral card with status timeline | a purchase in progress (delivery, call, offer) |

### 4.2 Full thread

```
┌─────────────────────────────────────┐
│ ‹  (M) Maya R. · online          ⋮  │
├─────────────────────────────────────┤
│  Using · Convo rescue pack          │   ← item bar; tap opens drawer
│  6 answers left              3 ⌄    │
├─────────────────────────────────────┤
│                                     │
│                She stopped replying │
│                mid-convo 😩         │
│                                     │
│                ┌──────────────────┐ │
│                │ ▦ hinge_convo.png│ │
│                │   tap to view    │ │   ← blurred until tapped
│                └──────────────────┘ │
│                                     │
│  ┌────────────────────────────────┐ │
│  │ How long since her last reply? │ │
│  │ Free                           │ │
│  └────────────────────────────────┘ │
│                                     │
│                3 days               │
│                                     │
│  ╭────────────────────────────────╮ │
│  │▌ANSWER 5 OF 10      Maya · 2m  │ │   ← ▌ = accent spine
│  │▌                               │ │
│  │▌Good news, this is recoverable.│ │
│  │▌Your last two texts were both  │ │
│  │▌questions — that reads needy   │ │
│  │▌after a gap.                   │ │
│  │▌                               │ │
│  │▌Send this:                     │ │
│  │▌┌─────────────────────────────┐│ │
│  │▌│ just saw a golden retriever ││ │   ← the copyable payload
│  │▌│ argue with a pigeon and     ││ │
│  │▌│ thought of your dog-park    ││ │
│  │▌│ story                       ││ │
│  │▌└─────────────────────────────┘│ │
│  │▌                               │ │
│  │▌  ⧉ Copy    ♡ Save    ⋯       │ │
│  ╰────────────────────────────────╯ │
│                                     │
├─────────────────────────────────────┤
│  ⊕   Message Maya…              ➤   │
└─────────────────────────────────────┘
```

**Why the answer card works**

1. **It looks like it cost money.** The border, the spine, the header. You can scroll a thread and see exactly where your value is.
2. **Copy is a first-class action.** The advice exists to be pasted into another app ten seconds later. A long-press-to-select bubble is a failure of design.
3. **The payload is separated from the reasoning.** Maya's explanation is context; the boxed text is the thing you send. Coaches are given this structure when composing.
4. **Save puts it in the Saved tab** (§6) — automatic for answers, but the explicit heart lets you mark the good ones.
5. **`⋯` holds the recourse:** *This didn't answer my question* → opens a targeted dispute against that specific answer. Cheap to build, and its existence is the deterrent against padding.

### 4.3 The item bar and its drawer

The bar is a single status line. Tapping it opens the drawer:

```
┌─────────────────────────────────────┐
│              ─────                  │
│  Your items with Maya               │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ ◉ Convo rescue pack           │  │
│  │   6 of 10 answers left        │  │
│  │   ─ in use ─                  │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ○ Opener rescue               │  │
│  │   2 answers left · tap to use │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ▦ Bio makeover · $30          │  │
│  │   Delivered — needs approval ▸│  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ⏱ Live call · $50             │  │
│  │   Friday 7:00 PM             ▸│  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ✓ 5 custom openers · $18      │  │
│  │   Completed · you rated ★★★★★ │  │
│  └───────────────────────────────┘  │
│                                     │
│  [  + Buy from Maya's menu  ]       │
└─────────────────────────────────────┘
```

- **Radio (◉/○)** only on chat items, because only they compete for the next answer. With one chat item there is no radio and the bar has no `⌄` — the concept never appears.
- **Icons** on card-scoped items; tapping jumps to that card in the thread.
- Completed items stay listed, dimmed — the drawer is also the receipt of the relationship.

### 4.4 Item bar states

| Situation | Bar reads |
|---|---|
| One chat item | `Using · Convo rescue pack — 6 answers left` |
| Multiple items | `Using · Convo rescue pack — 6 left    3 ⌄` |
| Exhausted | `No answers left · Buy more →` |
| Only a delivery open | `Bio makeover — in progress · due 22h` |
| Delivered, awaiting you | `Bio makeover — approve to release $30 →` |
| Late | `⚠ Late — 5 custom openers · Refund me →` |
| Call booked | `Live call — Friday 7:00 PM` |
| Nothing bought | `Browse Maya's menu →` |

### 4.5 The composer

```
Client:
┌─────────────────────────────────────┐
│  ⊕   Message Maya…              ➤   │
└─────────────────────────────────────┘
     └ screenshot · photo · ask about a match

Coach, answers available:
┌─────────────────────────────────────┐
│  Type a reply…                      │
│  ┌──────────────┬──────────────────┐│
│  │  Send free   │ Send answer 1of6 ││
│  └──────────────┴──────────────────┘│
└─────────────────────────────────────┘

Coach, nothing available:
┌─────────────────────────────────────┐
│  Type a reply…                      │
│  ┌──────────────┬──────────────────┐│
│  │  Send free   │ ░Send answer░    ││   ← disabled
│  └──────────────┴──────────────────┘│
│  Jordan has no answers left         │
│  [ Offer more help → ]              │
└─────────────────────────────────────┘
```

The client composer is always free and never mentions money. The coach's two buttons state the consequence at the moment of pressing — no hidden mode, no way to charge by accident.

### 4.6 Answer composer (coach side)

When a coach taps **Send answer**, they get structure — which is what produces the clean answer card:

```
┌─────────────────────────────────────┐
│  Your answer                        │
│  ┌───────────────────────────────┐  │
│  │ Why / what's going on…        │  │   ← reasoning
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ⧉ Copyable text (optional)    │  │   ← the payload they'll send
│  └───────────────────────────────┘  │
│  [+ another]   [+ attach]           │
│                                     │
│  Uses 1 of Jordan's 6 answers       │
│  [        Send answer        ]      │
└─────────────────────────────────────┘
```

Optional payload — a photo verdict has none, an opener is all payload. Multiple bubbles still cost **one** answer. A third option, **"Nothing to send — say why"**, produces the register in §4.7.

### 4.7 The Call — an answer that tells you to send nothing

```
╭────────────────────────────────╮
│▌THE CALL · ANSWER 4 OF 10      │   ← spine + header in --positive
│▌                               │
│▌Do nothing for 24 hours.       │
│▌Seriously. The silence does    │
│▌more work than any message I   │
│▌could write here.              │
│▌┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│▌│ NOTHING TO SEND            │ │   ← dashed, not solid
│▌│ Don't message her until    │ │
│▌│ tomorrow evening. Sending  │ │
│▌│ now reads as anxious.      │ │
│▌└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
│▌  ♡ Save        ⋯             │   ← no Copy; there is nothing to copy
╰────────────────────────────────╯
```

An answer carrying a `hold` block renders in its own register: **positive spine and header instead of accent, a dashed block instead of a solid one, no Copy action.** Everything else about the card is unchanged.

**Why this earns its own component.** It is the most differentiating artefact in the product. A tool built to generate a message will always generate a message; telling a paying customer that the right move is to send nothing — and so to not use what they just paid for — is a judgment only a person with a reputation at stake will make. Before this change that answer rendered as a normal card with its payload block simply absent, which read as an answer *missing something* rather than one *making a call*. The dashed border and the swapped colour say the absence is deliberate.

Coaches should be told in onboarding that this still consumes one answer and is not penalised. If holding costs a coach money or ranking, nobody holds, and the product quietly becomes a message generator with a human latency penalty.

### 4.8 Outcome capture on the answer card

```
│▌  ⧉ Copy      ♡ Save      ⋯                        │
│▌  Did it land?  (Replied) (No reply) (Didn't send)  │
╰────────────────────────────────────────────────────╯

…once answered:
│▌  ✓ Marked "Replied" — this counts toward Maya's score │
```

- **Only on answers with a payload, and never on the newest message.** Asking the instant an answer arrives collects noise — the client hasn't sent anything yet.
- **Three options, all safe to press.** `Didn't send` is explicitly *not* counted against the coach. Otherwise clients under-report it and the signal rots.
- **One tap. No sheet, no confirmation.** This has to be nearly free or nobody does it, and thin participation makes the dataset worthless.
- **Why it lives in the product rather than in analytics:** `ARCHITECTURE.md` §9.7 makes defect rate the basis of ranking, and the only two candidate moats are that dataset and the trust brand. Reviews rate a whole item after the fact; this rates each piece of advice. It is also the only instrument that can answer whether a paid human answer beats a free machine one — the bet everything else rests on.

---

## 5. Item cards in the thread

```
┌───────────────────────────────────┐
│ ▦  Bio makeover          $30      │
│    Dev K. · ordered Tue           │
│                                   │
│    ●───────●───────◉───────○      │
│    Paid  In prog  Deliv   Appr    │
│                                   │
│    📎 your_new_bio_2_versions.txt │
│                                   │
│    [ Approve ]  [ Request changes]│
│                                   │
│    $30 releases to Dev on approval│
│    auto-approves Friday           │
└───────────────────────────────────┘
```

One card per item for its entire life — it updates in place, never spawns duplicates. The same shape serves offers (`Accept · $35` / `Not now`), bookings, and late items (which gain a `Refund me $18` button and an amber edge).

---

## 6. Saved — the advice library

```
┌─────────────────────────────────────┐
│  Saved                              │
│  Your advice, one tap away          │
├─────────────────────────────────────┤
│  [All] [Openers] [Bio] [Photos] [⋯] │
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐  │
│  │ OPENER · Maya R. · Jul 22     │  │
│  │ "just saw a golden retriever  │  │
│  │  argue with a pigeon and      │  │
│  │  thought of your dog-park…"   │  │
│  │                       ⧉ Copy  │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ BIO · Dev K. · Jul 21         │  │
│  │ Version 1 — playful           │  │
│  │ "Potter by weekend, decent    │  │
│  │  cook by necessity…"          │  │
│  │                       ⧉ Copy  │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ PHOTOS · Sam T. · Jul 19      │  │
│  │ Keep #1 · Cut #2 · Reshoot #3 │  │
│  │                          ▸    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

- **Every paid answer lands here automatically**, tagged by the specialty it came from. The heart on an answer card just pins it to the top.
- Copy is the primary action everywhere — this tab exists to be used *inside another app's context*.
- Tapping a card jumps back to its thread for the full reasoning.
- **Why it matters commercially:** it's the retention surface. A dating app you deleted still leaves you with the bio you bought; Wing is where that lives. It's also the natural place for a future "buy another" prompt in context.

**Empty:** *"Answers you buy show up here so you can find them when you need them."*

---

## 7. You

```
┌─────────────────────────────────────┐
│  You                                │
│  ┌───────────────────────────────┐  │
│  │ (J) Jordan                    │  │
│  │     jordan@…  ·  Edit profile │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ ⚑ Switch to coach mode      ▸ │  │   ← or "Become a coach"
│  └───────────────────────────────┘  │
│                                     │
│  🧾 Purchases & receipts          ▸ │
│  ♥ Favourite coaches              ▸ │
│  🔔 Notifications                 ▸ │
│  🔒 Privacy & data                ▸ │
│  ？ Help                          ▸ │
│  ⚙ Settings · appearance         ▸ │
└─────────────────────────────────────┘
```

**Privacy & data** is deliberately prominent, not buried: delete uploaded screenshots early, see the auto-delete window, delete account. In a dating-adjacent product this is a trust surface, not a legal checkbox.

---

## 8. Coach mode

### Queue

```
┌─────────────────────────────────────┐
│ ▔▔▔▔▔▔▔ coach mode ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ │   ← persistent hairline
│  To fulfil                          │
├─────────────────────────────────────┤
│  ⚠ LATE · 1                         │
│  ┌───────────────────────────────┐  │
│  │ Alex — 5 opener bundle        │  │
│  │ Promised 2h · 3 days ago      │  │
│  │ Refunds automatically in 4h ▸ │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐  │
│  │ Jordan — answer waiting       │  │
│  │ Convo pack · screenshot       │  │
│  │ goal ~5 min · 2 min elapsed ▸ │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │ Riley — custom request        │  │
│  │ "first date in Austin"        │  │
│  │ [ Quote ]      [ Decline ]    │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  Today: 7 answers · $46 · 4 min avg │
└─────────────────────────────────────┘
```

Late work is separated and alarming on purpose — it's the one thing that damages a coach's ranking, so it should be impossible to miss.

### Menu and Earnings

Unchanged from v1 in structure (three-question composer; post-fee earnings with refund rate and repeat rate shown, since those drive ranking). See `flows-and-wireframes.html` §3.2 and §3.4.

---

## 9. Component reference

| Component | Register | Key rules |
|---|---|---|
| `AnswerCard` | paid | Accent spine, header `ANSWER n OF m`, optional copyable payload block, actions Copy / Save / `⋯`; outcome row when it has a payload and isn't the newest message |
| `AnswerCard.hold` | paid | **Positive** spine and header (`THE CALL · ANSWER n OF m`), dashed `NOTHING TO SEND` block, no Copy action |
| `OutcomeRow` | signal | Three one-tap options; `Didn't send` never counts against the coach; collapses to a confirmation line |
| `UrgencyRow` | discovery | Five situations ordered by deadline; sets category filter and sorts by response time; mutually exclusive with the category chips |
| `MessageBubble` | free | Plain; coach messages carry a small `Free` label, client messages carry nothing |
| `ItemCard` | purchase | Status timeline, files, actions, auto-approve note; updates in place |
| `ItemBar` | status | One line; `⌄` + count only when 2+ items; tap opens drawer |
| `ItemsDrawer` | status | Radios on chat items only; completed items dimmed; buy-more CTA |
| `NeedsYouRow` | action | Always states the consequence; disappears when empty |
| `CoachCard` | discovery | Price-forward, heart top-right, refund/return stats under the stars |
| `FavouriteHeart` | action | Private, optimistic, works from card and profile |
| `SavedCard` | library | Copy is the primary action; jump-to-thread secondary |

---

## 10. Interaction and motion

- **One signature motion:** the answer counter ticking down when an answer lands (`6 → 5`). Everything else is instant.
- **Optimistic everywhere else:** favouriting, saving, and sending messages update immediately and reconcile silently.
- **Sheets** (purchase, items drawer, review) slide from the bottom, dismissible by swipe or scrim tap. Never a full-page navigation for something you'll close in five seconds.
- **Copy gives feedback:** button becomes `✓ Copied` for 1.5s. This action happens more than any other in the app.
- Respect `prefers-reduced-motion`: all of the above become instant.

---

## 11. Visual language (unchanged from v1)

| Token | Light | Dark | Use |
|---|---|---|---|
| `--bg` | `#FFFBF8` | `#17131A` | ground |
| `--surface` | `#FFFFFF` | `#201A22` | cards |
| `--ink` | `#2A2230` | `#F1EAEE` | text |
| `--ink-soft` | `#6E6068` | `#C4B8C0` | secondary |
| `--line` | `#F0E4DD` | `#362E3B` | borders |
| `--accent` | `#D6456A` | `#F2718F` | money, primary CTA, answer spine — **only** |
| `--positive` | `#2F5D62` | `#7FC0C3` | free/included, success — never CTAs |
| `--gold` | `#E0A03C` | `#E0A64E` | stars only |
| `--warn` | `#B77410` | `#E0A64E` | late items only |

Serif for voice moments (`Your coaching`, `Who do you need?`); system sans for UI; **monospace for every dollar figure, meter, and status badge**. Radii: cards 14–16, sheets 22 top, pills 999.

**Accessibility:** money and status never rely on colour alone (badges carry words); 4.5:1 contrast minimum; visible focus states; the answer card is a landmark region so screen readers can jump between paid answers.

---

## 12. What this requires from the data model

Additions needed in `ARCHITECTURE.md` (§3 schema):

- `favourites(user_id, coach_id, created_at)` — private, no coach-facing read.
- `saved_items(user_id, message_id, engagement_id, specialty, pinned, created_at)` — auto-inserted for every answer; `pinned` set by the heart.
- `messages.payload` gains `{ reasoning, blocks:[{type:'copy'|'verdict'|'file', text}] }` so `AnswerCard` can render the structured answer. No new table.
- Home's *Needs you* is one query over `engagements` (status `delivered` | `late` | `settled`-without-review) plus pending offers — no new state.

Everything else in this document is presentation over the existing model.
