# Wing — Build Plan & Architecture Specification

**Version 2.0 · July 2026 · Status: ready to implement**

Complete, self-contained specification for **Wing**: a mobile-responsive multiuser web marketplace where dating coaches sell help — chat answers, deliverables (bios, photo reviews, opener bundles), live sessions, packs, and custom-quoted work — and clients pay **plain dollars per item at the moment of purchase**.

> **How to use this document (instructions to the coding agent):**
> Build milestones **M0 → M4 in order** (§15); each has acceptance criteria that must pass before the next begins. Where this spec is silent, prefer the simplest implementation consistent with the invariants in §2.5 and §6. Anything in §16 (out of scope) must NOT be built. **All money moves through `ledger.post()` and nothing else** (§6.2). **All behaviour is recorded through `events.emit()` and nothing else** (§2.5). Use the UI vocabulary in §1 verbatim — never expose internal names in the interface. Companion documents: *Flows & Wireframes* (every screen and path) and the *UI/UX Pitch* (visual design).

**What changed in v2:** unified fulfilment strategies replacing per-scheme special-casing (§2.5); event log as a first spine (§2.5, §5); processor fees now modelled (§6.1); coach-failure SLA path added (§7); pack refunds priced à la carte (§6.6); free items excluded from review weight (§12.7); ranking contradiction resolved in favour of defect-rate (§11); `pg_trgm` replaces embeddings (§12.7); M2 split into three (§15); analytics instrumented from M1 (§2.5).

---

## 1. Product overview & vocabulary

Two roles on one account model. A user can be both. Plus a minimal **admin** surface (§12.5).

- **Client** — browses coaches, buys items, gets help in a thread, reviews what they bought.
- **Coach** — publishes a menu, fulfils purchases, gets paid out via Stripe Connect.

### Internal names → UI words (never mix these)

| Internal (code, DB) | UI word | Notes |
|---|---|---|
| `engagement` | **item** | "Your items with Maya" |
| `engagement.units_*` | **answers** (or the coach's `unit_label`: photos, minutes) | "6 answers left" |
| `active_engagement_id` | **in use** | "Using · Convo pack". Never "armed". |
| `useItem()` | **Switch** | Control appears only when 2+ chat items exist |
| consumption / `usage_event` | **answer** | Badge reads "Answer 5 of 10" |
| fulfilling message | **answer** | Coach button: "Send answer · 1 of 6" |
| non-fulfilling message | **message** | Coach button: "Send free"; badge "Free" |
| `proposal` | **offer** | "Offer from Maya" |
| `offering` | **menu item** | |

### The product invariant (enforce in code, restate in comments)

> **A coach's action can cost the client exactly what they already agreed to — or less. Never more.**
> The client picks the item and pays; the coach fulfils within those terms. Anything beyond them is an *offer* requiring explicit client acceptance and payment.

---

## 2. Architecture

### 2.1 Tech stack (pinned)

| Layer | Choice | Notes |
|---|---|---|
| Framework | **Next.js 15+ (App Router) + TypeScript** | Server actions for mutations; route handlers for webhooks |
| UI | **Tailwind + shadcn/ui** | Tokens §17, mobile-first |
| DB / Auth / Storage / Realtime | **Supabase** (Postgres 15+) | Local via `supabase start`; migrations in repo |
| Payments | **Stripe** — PaymentIntents, saved methods, **Connect Express** | Behind a provider interface with **mock mode** (§6.5) |
| Validation | **zod** + react-hook-form | One schema, both sides |
| Testing | **Vitest** (all money/lifecycle) + **Playwright** (e2e) | |
| Deploy | Vercel + Supabase cloud | No long-lived processes |

Do **not** add: Redux/Zustand, tRPC, an ORM, XState, or an embeddings/AI dependency.

### 2.2 The three spines

Everything in the system is one of three things. Nothing else is authoritative; every other table is a projection, cache, or view.

| Spine | Table | Mutability | Sole writer |
|---|---|---|---|
| **Money** | `ledger_entries` | append-only, balanced | `ledger.post()` |
| **Behaviour** | `events` | append-only | `events.emit()` |
| **Domain state** | `engagements` | mutable, one row per purchase | `lifecycle.advance()` |

Consequences worth stating plainly: balances are always *derived* by folding the ledger; audit, analytics, notifications, and fraud signals are all *readers* of one event log rather than five bespoke tables; and an engagement can only change through a guarded transition that writes to both other spines in the same transaction.

```ts
// lib/ledger.ts — the ONLY module that writes ledger_entries
post(txnKey: string, legs: {account: string; deltaCents: number}[], reason: LedgerReason, refId?: string): Promise<void>
// throws unless sum(deltaCents) === 0; unique(txnKey) makes every call idempotent

// lib/events.ts — the ONLY module that writes events
emit(kind: string, subject: {type: 'engagement'|'thread'|'profile'|'review'; id: string}, actorId: string|null, props?: object): Promise<void>
```

### 2.3 One lifecycle, three fulfilment strategies

v1 special-cased 5 pricing schemes × 3 delivery modes. That matrix collapses: **pricing decides only how much and how many units; delivery decides only who advances the item and when.** One state machine serves all combinations, calling into a strategy.

```ts
interface FulfilmentStrategy {
  onFunded(eng): Promise<void>            // chat: auto-select as in-use; scheduled: create booking
  deliverBy(eng): Date                    // SLA deadline from the coach's turnaround promise
  fulfil(eng, actor, payload): Promise<void>  // chat: consume 1 unit; delivery: mark delivered; scheduled: complete session
  isSettled(eng): boolean                 // chat: units exhausted; others: approved
  settleAbandoned(eng): SettlementPlan    // how to split escrow when it goes stale (§6.6)
}
```

Three implementations — `ChatStrategy`, `DeliveryStrategy`, `ScheduledStrategy` — and no other branching on `scheme` or `fulfilment` anywhere in the codebase. A new offering type is a new strategy, not edits across the app.

### 2.4 Pricing is a pure function

```ts
quote(offering, qty?): { amountCents; feeCents; unitsTotal; unitLabel; description }
```

No I/O, no side effects, exhaustively unit-tested; the single place any of the five schemes turns into money. `createEngagement` calls it once and **snapshots the result onto the engagement**, so later price changes never rewrite history.

### 2.5 Derived reads, not denormalised writes

Ranking and profile stats come from a **`coach_stats` materialized view** (refreshed every 15 min by job), never from columns updated inside transactions. This is what makes fraud de-weighting work: voiding a review recomputes rankings automatically on the next refresh, with no update path to forget.

### 2.6 Instrumentation (build in M1, not later)

The MVP's most valuable output is evidence about demand. `events.emit()` already captures it; these named events are **required**: `signup`, `coach_viewed`, `purchase_sheet_opened`, `purchase_completed`, `purchase_failed`, `first_answer_received`, `item_completed`, `review_submitted`, `repeat_purchase`, `refund_requested`. A single SQL view reports the kill metric: **first → second purchase rate per cohort week**.

### 2.7 Repository layout

```
wing/
  app/
    (marketing)/page.tsx
    (auth)/sign-in · sign-up · callback
    coaches/page.tsx                 # discover
    coaches/[handle]/page.tsx        # profile + menu
    threads/page.tsx · threads/[id]/page.tsx
    purchases/page.tsx
    coach/apply · coach/menu · coach/queue · coach/earnings
    settings/page.tsx · admin/page.tsx
    api/stripe/webhook/route.ts · api/uploads/sign/route.ts
  lib/
    ledger.ts          # spine 1 — post()
    events.ts          # spine 2 — emit()
    lifecycle.ts       # spine 3 — advance() + transition table
    strategies/        # chat.ts · delivery.ts · scheduled.ts
    pricing.ts         # quote()
    payments/          # provider.ts · stripe.ts · mock.ts
    db/ · validators/ · realtime/
  components/ ui · marketplace · thread · coach
  supabase/ migrations/*.sql · seed.sql
  tests/ unit · e2e
```

### 2.8 Environment

```
NEXT_PUBLIC_SUPABASE_URL= / NEXT_PUBLIC_SUPABASE_ANON_KEY= / SUPABASE_SERVICE_ROLE_KEY=
PAYMENTS_MODE=mock|stripe            # default mock — app must be fully demoable in mock
STRIPE_SECRET_KEY= / STRIPE_WEBHOOK_SECRET= / STRIPE_CONNECT_CLIENT_ID=
PLATFORM_FEE_BPS=2000                # 20.00%
AUTO_APPROVE_DAYS=3
SLA_BREACH_MULTIPLIER=3              # auto-refund at 3× the coach's stated turnaround
ATTACHMENT_TTL_DAYS=30
APP_URL=
```

---

## 3. Database schema

Money is **integer cents**. Every table gets `created_at timestamptz default now()`.

```sql
create type specialty as enum ('openers','bio','photos','conversations','dates','custom');
create type fulfilment_mode as enum ('chat','delivery','scheduled');
create type pricing_scheme as enum ('per_unit','flat','pack','subscription','quote');
create type item_status as enum
  ('awaiting_payment','in_progress','delivered','settled','late','cancelled','refunded','disputed');
create type message_kind as enum ('text','attachment','offer','item_update','system');
create type ledger_reason as enum
  ('purchase','escrow_release','platform_fee','processor_fee','refund','payout');
create type coach_status as enum ('none','applied','approved','suspended');

create table profiles (
  id uuid primary key references auth.users(id),
  handle text unique not null,
  display_name text not null,
  avatar_url text, pronouns text,
  is_coach boolean not null default false,
  coach_status coach_status not null default 'none',
  headline text, bio text,
  specialties specialty[] not null default '{}',
  stripe_account_id text,           -- Connect Express (coach)
  stripe_customer_id text,          -- saved cards (client)
  payment_fingerprints text[] not null default '{}',   -- server-only, fraud linking (§12.7)
  created_at timestamptz not null default now()
);
-- NOTE: no rating_avg/rating_count columns. Stats come from coach_stats (§2.5).

create table offerings (
  id uuid primary key default gen_random_uuid(),
  coach_id uuid not null references profiles(id),
  title text not null, description text,
  specialty specialty not null,
  fulfilment fulfilment_mode not null,
  scheme pricing_scheme not null,
  unit_label text default 'answer',
  unit_price_cents int, flat_price_cents int,
  pack_units int, pack_price_cents int,
  session_minutes int,
  required_inputs jsonb not null default '[]',   -- [{key,label,type:'text'|'images',required}]
  first_unit_free boolean not null default false,
  turnaround_mins int not null,                  -- the coach's promise; drives the SLA clock
  active boolean not null default true,
  sort int not null default 0
);

create table threads (
  id uuid primary key default gen_random_uuid(),
  client_id uuid not null references profiles(id),
  coach_id  uuid not null references profiles(id),
  active_engagement_id uuid,        -- FK added after engagements; the chat item "in use"
  last_message_at timestamptz,
  unique (client_id, coach_id)
);

-- ENGAGEMENTS = "items". Terms are FROZEN at purchase (snapshot from pricing.quote()).
create table engagements (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references threads(id),
  offering_id uuid references offerings(id),     -- null for ad-hoc quotes
  client_id uuid not null references profiles(id),
  coach_id  uuid not null references profiles(id),
  status item_status not null default 'awaiting_payment',
  title text not null,                           -- snapshot
  scheme pricing_scheme not null,                -- snapshot
  fulfilment fulfilment_mode not null,           -- snapshot
  unit_label text, unit_price_cents int,         -- snapshot; unit_price is the À LA CARTE rate (§6.6)
  units_total int not null default 1,
  units_used  int not null default 0,
  amount_cents int not null,                     -- what the client paid
  fee_cents int not null,                        -- platform fee, from quote()
  is_free boolean not null default false,        -- free first answers etc. (§12.7)
  payment_intent_id text,
  client_inputs jsonb,
  deliver_by timestamptz,                        -- SLA deadline (§7)
  delivered_at timestamptz,
  auto_approve_at timestamptz,
  settled_at timestamptz
);
alter table threads add constraint threads_active_fk
  foreign key (active_engagement_id) references engagements(id);

create table messages (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references threads(id),
  sender_id uuid not null references profiles(id),
  kind message_kind not null default 'text',
  body text, payload jsonb, attachment_path text,
  engagement_id uuid references engagements(id),  -- set ONLY by the server when this was an answer
  units_consumed int not null default 0,
  created_at timestamptz not null default now()
);
create index on messages (thread_id, created_at);

-- SPINE 1 — money. Accounts: client:<id> | escrow:<engagement_id> | coach:<id>
--   | platform:fees | processor:fees | external:stripe
create table ledger_entries (
  id bigint generated always as identity primary key,
  txn_key text not null,                 -- idempotency: unique per logical transaction
  account text not null,
  delta_cents int not null,
  reason ledger_reason not null,
  ref_id uuid,
  created_at timestamptz not null default now()
);
create unique index on ledger_entries (txn_key, account);
create index on ledger_entries (account);

-- SPINE 2 — behaviour. Audit, analytics, notifications and fraud all READ from here.
create table events (
  id bigint generated always as identity primary key,
  kind text not null,                    -- 'purchase_completed' | 'answer_sent' | ...
  subject_type text not null,            -- 'engagement' | 'thread' | 'profile' | 'review'
  subject_id uuid not null,
  actor_id uuid references profiles(id),
  props jsonb not null default '{}',
  created_at timestamptz not null default now()
);
create index on events (kind, created_at);
create index on events (subject_type, subject_id);

create table reviews (
  id uuid primary key default gen_random_uuid(),
  engagement_id uuid unique not null references engagements(id),
  offering_id uuid references offerings(id),
  client_id uuid not null references profiles(id),
  coach_id uuid not null references profiles(id),
  stars int not null check (stars between 1 and 5),
  outcome_tags text[] not null default '{}',
  body text,
  weight numeric(4,3) not null default 1.0   -- 0 for free items; reduced when flagged (§12.7)
);
create index on reviews (offering_id); create index on reviews (coach_id);

create table help_requests (
  id uuid primary key default gen_random_uuid(),
  engagement_id uuid not null references engagements(id),
  opened_by uuid not null references profiles(id),
  type text not null check (type in ('changes','refund','report')),
  body text,
  status text not null default 'open' check (status in ('open','resolved','escalated')),
  resolved_at timestamptz
);

create table coach_applications (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references profiles(id),
  specialties specialty[] not null, proof text, audition_answer text not null,
  status text not null default 'pending' check (status in ('pending','approved','rejected')),
  reviewed_at timestamptz
);

create table fraud_flags (
  id uuid primary key default gen_random_uuid(),
  coach_id uuid not null references profiles(id),
  rule text not null, evidence jsonb not null default '{}',
  resolved boolean not null default false
);

create table thread_reads (thread_id uuid, user_id uuid, last_read_at timestamptz, primary key (thread_id, user_id));
create table admins (user_id uuid primary key references profiles(id));
```

### Materialized view — the only source of ranking and profile stats

```sql
create materialized view coach_stats as
select p.id as coach_id,
       count(*) filter (where e.status = 'settled')                     as items_completed,
       count(distinct e.client_id)                                      as clients,
       -- defect rate: the primary ranking input (§11)
       (count(*) filter (where e.status in ('refunded','disputed','late'))::numeric
         / nullif(count(*) filter (where e.status <> 'awaiting_payment'),0))  as defect_rate,
       (count(distinct e.client_id) filter (where e.client_id in (
          select client_id from engagements g where g.coach_id = p.id
          group by client_id having count(*) > 1))::numeric
         / nullif(count(distinct e.client_id),0))                       as repeat_rate,
       avg(r.stars) filter (where r.weight > 0)                         as rating_avg,
       count(r.*) filter (where r.weight > 0)                           as rating_count,
       sum(r.stars * r.weight) / nullif(sum(r.weight),0)                as rating_weighted,
       percentile_disc(0.5) within group (order by resp.mins)           as response_time_mins
from profiles p
left join engagements e on e.coach_id = p.id
left join reviews r     on r.coach_id = p.id
left join lateral (select 1 as mins) resp on true   -- replace with real first-response calc
where p.coach_status = 'approved'
group by p.id;
create unique index on coach_stats (coach_id);
-- plus offering_stats: same shape keyed by offering_id, for per-item ratings on menu rows
```

### Row-Level Security (enable on every table)

| Table | Policy |
|---|---|
| `profiles` | Read: approved coaches' public columns + own row. Write: own row, never `coach_status`/`stripe_*`/`payment_fingerprints` |
| `offerings` | Read: active offerings of approved coaches + own. Write: own |
| `threads`, `messages` | Participants only. Client inserts limited to `kind in ('text','attachment')` **with `engagement_id` null and `units_consumed = 0`** — answers are server-written only |
| `engagements`, `ledger_entries`, `events`, `fraud_flags` | Read: own/participant. **No client writes at all** |
| `reviews` | Read: public. Insert: client of a settled engagement with no existing review |
| `help_requests`, `coach_applications` | Own rows; admins read all |

---

## 4. Money

### 4.1 Model

- Every purchase is one Stripe PaymentIntent charged immediately (saved card / Apple Pay / Google Pay).
- Money sits in **escrow per item** until the strategy settles it.
- Coach earnings accrue in `coach:<id>`; weekly payouts transfer to their Connect account.
- **No wallet, no credits.** Charge pattern: separate charges & transfers.

### 4.2 Processor fees are modelled (new in v2)

Stripe's cut (~2.9% + 30¢) is **10.5% on a $4 item** — material to a micro-transaction business, and v1 ignored it. Every purchase records the real fee from the charge object into `processor:fees`, so `platform:fees` reflects true net revenue:

```
purchase($25):  external:stripe +2500 → escrow:<eng> -2500
                escrow:<eng>  ... on settle:
                escrow -2500 → coach +2000, platform:fees +500
                platform:fees -103 → processor:fees +103   (actual Stripe fee)
```

### 4.3 The transaction builders

Thin wrappers over `ledger.post()`; nothing else touches money:

```ts
recordPurchase(eng, paymentIntent)   // txnKey: `purchase:${eng.id}`
settleItem(eng, plan)                // txnKey: `settle:${eng.id}:${plan.seq}`
recordRefund(eng, cents, seq)        // txnKey: `refund:${eng.id}:${seq}`
recordPayout(coachId, cents, ref)    // txnKey: `payout:${ref}`
```

Note the sequence suffixes — v1's `(engagement_id, reason)` key **collided** on repeated consumption and partial refunds.

### 4.4 Purchase → fulfil → settle

1. Client taps an item → `createEngagement` runs `pricing.quote()`, snapshots terms, creates the PaymentIntent with `metadata.engagement_id`.
2. Webhook `payment_intent.succeeded` → verify signature → `recordPurchase` → status `in_progress` → `strategy.onFunded()` → `events.emit('purchase_completed')` → system message in thread.
3. Coach fulfils via `strategy.fulfil()`:
   - **chat** — consumes one unit if this item is the thread's `active_engagement_id`, is `in_progress`, and has units left. Uses `select … for update` to make concurrent sends safe.
   - **delivery** — attaches payload/files, sets `delivered_at` + `auto_approve_at`.
   - **scheduled** — marks the session complete.
4. Client approves (or auto-approve job fires) → `settleItem` → status `settled` → review prompt.
5. Chat items settle automatically when units are exhausted.

### 4.5 Payments provider + mock

```ts
interface PaymentsProvider {
  createPaymentIntent(i): Promise<{id; clientSecret}>
  createCustomer(u): Promise<{id}>
  createConnectAccount(u): Promise<{id; onboardingUrl}>
  transfer(t): Promise<{id}>
  refund(r): Promise<{id}>
  retrieveChargeFees(paymentIntentId): Promise<{feeCents}>   // for processor:fees
}
```

`mock.ts` succeeds instantly and invokes the same webhook handler function directly. **Every demo, test and seed runs in mock mode.**

### 4.6 Abandonment & partial settlement (policy fixed in v2)

Unused answers on an abandoned item are refunded **at the à-la-carte `unit_price_cents`, not the discounted pack rate**. Otherwise abandoning a pack early is cheaper per answer than buying singles, and coaches are penalised for offering bundles.

```
Pack: 10 answers for $2500 (à la carte 400/answer). Client uses 3, goes quiet.
settleAbandoned → coach earns 3 × 400 = 1200 (less fee); client refunded 1300.
```

Chat items settle this way after `AUTO_APPROVE_DAYS` of thread inactivity. Packs may also carry an expiry; on expiry, unused answers are forfeited to the coach only if the offering stated it at purchase.

### 4.7 Multiple items in one thread — the "in use" rule

- **Card-scoped items** (delivery, scheduled, quotes): addressed by their own card; any number coexist; never ambiguous.
- **Chat items**: at most one is `active_engagement_id`. Answers consume that one only.
- **Selection**: a newly funded chat item auto-selects itself. The client switches via `useItem(threadId, engagementId)`. **The coach can never select.** With one chat item the UI shows no control at all (§1 vocabulary; *Flows* §01).
- If nothing is selected or units are exhausted, `sendAnswer` throws and the UI disables the button — free messages and offers remain.
- On settle/refund of the selected item, null `active_engagement_id` in the same transaction.

### 4.8 Per-answer purchases

Sold as small prepaid quantities (1 / 3 / 5) — one charge, `units_total = qty`. One-tap **"Buy 3 more"** re-purchase off-session on the saved card when they run out. Micro-charge batching is out of scope.

---

## 5. Lifecycle

One transition table for all items; illegal transitions throw. Strategy hooks do the type-specific work.

| From | Event | To | Effects |
|---|---|---|---|
| `awaiting_payment` | payment succeeded (webhook) | `in_progress` | `recordPurchase`; `onFunded`; system msg |
| `awaiting_payment` | cancel / expire 24h | `cancelled` | cancel PI |
| `in_progress` | coach answers (chat) | `in_progress` | consume unit; `answer_sent` event |
| `in_progress` | last unit consumed | `settled` | `settleItem` |
| `in_progress` | coach marks delivered | `delivered` | set `auto_approve_at` |
| `in_progress` | **`deliver_by` passed** (job) | **`late`** | notify both; expose client refund button |
| `late` | coach delivers | `delivered` | defect recorded on `coach_stats` |
| `late` | client refunds, or `SLA_BREACH_MULTIPLIER` reached (job) | `refunded` | full `recordRefund` |
| `delivered` | client approves / auto-approve job | `settled` | `settleItem` |
| `delivered` | request changes (max 2) | `in_progress` | reset `deliver_by` |
| any active | client opens refund/report | `disputed` | freeze settlement |
| `disputed` | admin resolves | `settled` \| `refunded` | `settleItem` and/or `recordRefund` |
| `in_progress` (chat) | thread inactive `AUTO_APPROVE_DAYS` | `settled` | `settleAbandoned` (§4.6) |

`subscription` is phase 2 — the enum exists, the UI hides it.

---

## 6. API surface (server actions unless noted)

All validate with zod, check auth and role, and emit events.

**Marketplace** `listCoaches({specialty?,priceMaxCents?,query?,cursor?})` · `getCoachProfile(handle)`
**Menu (coach)** `createOffering` · `updateOffering` · `setOfferingActive` · `reorderOfferings`
**Purchases** `createEngagement({offeringId|offerMessageId, qty?, clientInputs})` · `cancelPending` · `rebuy(engagementId, qty)`
**Thread** `getOrCreateThread(coachId)` · `listThreads` · `listMessages(threadId,cursor)` · `sendMessage` (free) · `sendAnswer(threadId, engagementId, {body,payload?})` · `useItem(threadId, engagementId)` *(client only)* · `sendOffer(threadId, terms)` · `respondToOffer(messageId, accept)`
**Delivery** `markDelivered(engagementId,{payload,attachmentPaths})` · `approveItem` · `requestChanges(engagementId, note)`
**Reviews** `submitReview(engagementId,{stars,outcomeTags,body})`
**Help** `openHelpRequest(engagementId,type,body)` · `resolveHelpRequest(id,resolution)` *(admin)* · `refundLateItem(engagementId)` *(client, only when `late`)*
**Coach** `submitCoachApplication` · `reviewApplication(id,approve)` *(admin)* · `startConnectOnboarding` · `getEarnings` · weekly payout job
**Routes** `POST /api/stripe/webhook` (signature-verified, idempotent by event id) · `POST /api/uploads/sign` (png/jpg/webp/txt/pdf ≤ 10 MB)

---

## 7. Realtime

One Supabase Realtime channel per thread (`thread:<id>`) on `messages` insert and `engagements` update, plus presence for "online now". Inbox subscribes for unread badges via `thread_reads`. SWR polling every 10 s as fallback. **No writes over the socket** — all mutations are server actions.

---

## 8. Frontend

**The UI references are the *Flows & Wireframes* doc (structure, states, every path) and the *UI/UX Pitch* (visual design).** Rules:

- **Mobile-first at 390 px**, then up. Desktop: discover becomes a grid; coaches get a two-pane inbox→thread workspace; max width 1100 px.
- **Money components** (build once in `components/thread/`): `ItemBar` (in-use status + meter; expands to the items sheet with switch control when 2+ chat items), `ItemsSheet`, `AnswerBadge` ("Answer 5 of 10"), `FreeBadge`, `OfferCard` (Accept · $X / Not now), `OrderCard` (Paid → In progress → Delivered → Approved, files, approve/changes, auto-approve note), `PurchaseSheet`, `LateBanner` (refund CTA), `VerdictRow`, `OpenerRow` (copy button), `GroupedAnswer` (multiple bubbles, one answer).
- **Coach composer** = two explicit buttons — "Send free" and "Send answer · N of M" — never a hidden toggle; disabled with reason when nothing is available.
- Dark mode via tokens (§17). Money semantics never colour-only. Respect `prefers-reduced-motion`.
- Copy comes from `strings.ts` (i18n later); use §1 vocabulary verbatim.

### Surface notes

- **Discover**: server component; filters in searchParams; Postgres full-text over name+headline+bio; **ranking is `coach_stats`: defect_rate asc, then repeat_rate desc, then items_completed desc, then rating_weighted desc** (this supersedes v1's rating-first ordering).
- **Thread**: render by message `kind`; `item_update` renders from its snapshot payload so history stays truthful.
- **Uploads**: signed URL → Storage `attachments/` → message with path. Images blurred until tapped. TTL job purges (§12).
- **Reviews**: prompted on settle and from purchases; names the item; `submitReview` sets `weight = 0` when `is_free`; aggregates come from the views, not columns.
- **Coach queue**: one query over the coach's items — answers awaiting reply, deliveries with `deliver_by`, open offers, help requests — ordered by urgency, with `late` first.
- **Earnings**: post-fee figures only, from `coach:<id>` ledger rows + `coach_stats`.

---

## 9. Trust, safety, privacy, ops

1. **Privacy**: attachments auto-expire (`ATTACHMENT_TTL_DAYS`); notifications never contain message content; self-serve deletion purges attachments and anonymises, keeping the ledger for accounting.
2. **The coaching line**: coaches advise and draft; they never operate a client's dating account. ToS + application checkbox; `report` feeds the admin queue.
3. **Rate limits**: messages 30/min, purchases 10/hour, applications 3/day.
4. **Jobs** (Vercel cron or pg_cron; claim rows `for update skip locked` — cron is at-least-once and *will* double-fire; `txn_key` makes money idempotent regardless): auto-approve; SLA breach → `late` → auto-refund; abandonment settlement; expire unpaid items (24 h) and offers (7 d); attachment purge; weekly payouts; refresh `coach_stats`.
5. **Admin**: applications, help requests, fraud flags, reports — plain tables, service-role, allowlist-gated.
6. **Audit**: the ledger plus the event log; no separate audit table.
7. **Review-fraud defences** — reuse-first: card fingerprints and risk scores from **Stripe/Radar**; device fingerprint via **ThumbmarkJS** (MIT — *not* FingerprintJS, whose current OSS release is BSL); text similarity via **`pg_trgm`** (ships with Postgres — no embeddings, no AI dependency); tripwires are ~5 SQL queries on a nightly cron, graduating to **Marble** (OSS rules engine) only if rules multiply. Specifics:
   - Reviews require a **settled, paid** item. **Free items get `weight = 0`** and never count toward totals — otherwise free first answers are a zero-cost review-farming vector.
   - A card fingerprint shared across accounts reviewing the same coach links and voids those reviews together; prepaid/virtual cards start low-weight.
   - **Negatives outrank praise.** Positive ratings inflate toward uselessness; the discriminating signal is the negative tail, which the paid gate defends structurally — attacking a rival costs full price per attempt. Ranking order is defect rate → repeat rate → clean volume → weighted stars, Bayesian-shrunk so 10 clean items don't outrank 2 defects in 1,400. Volume-with-clean-record cannot be purchased.
   - Profiles surface a **"most critical recent"** review alongside the latest; burying negatives is what makes five-star walls read as fake.
   - Tripwires → `fraud_flags`: review bursts vs. baseline; near-zero-consumption items reviewed; review within minutes of purchase; `pg_trgm` similarity clusters; device/IP correlation between coach and reviewers. Flags reduce `weight` immediately; removal is an admin decision.
   - Enforcement: the weekly payout delay is the clawback window — confirmed fraud withholds settlement, reverses escrow, zeroes the reviews and suspends a **KYC-verified** identity.

---

## 10. Seed data

Demo users (password `demo1234`): clients `jordan@`, `alex@`, `riley@demo.wing`; coaches **Maya R.** ($4/answer first-free · $25 pack of 10 · $18 five-opener deliverable · $50 live 30 min), **Dev K.** ($30 bio makeover · $15 prompts), **Sam T.** ($5/photo · $20 lineup), **Elena V.** ($6/answer · free intro call), **Marcus H.** ($45 restart), **Priya N.** (quote-only), **Jo A.** ($5/answer · $22 first-date plan). Seed ~30 reviews linked to their offerings; threads in varied states — pack partly used, bio delivered awaiting approval, accepted quote delivered, one `late` item, one open refund; ledger entries that balance. Runs in mock mode.

---

## 11. Testing

- **Unit**: every `ledger.post` path (balanced legs, idempotency by `txn_key`, guards); `pricing.quote()` for all five schemes; the full transition table (legal and illegal); `settleAbandoned` à-la-carte maths; concurrent `sendAnswer` on one remaining unit → exactly one succeeds.
- **e2e (mock payments)**: (1) buy pack → free clarifier + two answers → meter and badges correct → review; (2) bio: buy with inputs → deliver → approve → coach earnings net of fee; (3) quote: ask → offer → accept → deliver → auto-approve via injected clock; (4) **two chat items in one thread** — answers follow the in-use item, switching re-routes them; (5) **late path** — clock past `deliver_by` → `late` → client refund → escrow reversed.
- CI: typecheck, lint, unit, e2e against local Supabase, single-threaded.

---

## 12. Milestones

**M0 — Foundation & browsable marketplace.** Scaffold, Supabase local, migrations, auth, profiles, seeded coaches, discover + profile with menu (CTA disabled), responsive shell, tokens. ✔ *Sign up and browse with filters at 390 px.*

**M1 — Money rails & instrumentation.** `ledger.post` + `events.emit` + tests, `pricing.quote`, engagements + lifecycle, PurchaseSheet on mock provider, webhook, processor-fee recording, purchases page, **real Stripe Connect Express spike** (one live test account end-to-end — do not defer this), analytics view for first→second purchase. ✔ *Buy a flat item in mock mode; ledger balances; Connect onboarding proven.*

**M2a — Thread.** Threads, messages, attachments, realtime, inbox, free messaging both sides. ✔ *Two users converse in real time.*

**M2b — Metering.** ChatStrategy, `ItemBar`/`ItemsSheet`, in-use selection + switching, `sendAnswer` consumption transaction, per-answer and pack purchases, rebuy. ✔ *e2e #1 and #4.*

**M2c — Delivery & offers.** DeliveryStrategy, input forms, OrderCard, approve/changes, auto-approve job, offers/quotes, ScheduledStrategy (booking + external video link). ✔ *e2e #2 and #3.*

**M3 — Reputation, queue & recourse.** Reviews with weights, `coach_stats`/`offering_stats` views + ranking, coach queue, earnings, weekly payouts, SLA/late path + auto-refund, help requests + admin resolution, coach application + audition + approval. ✔ *e2e #5; refunds reverse escrow correctly.*

**M4 — Polish & launch.** Dark mode, notifications, report/block, rate limits, TTL/privacy jobs, fraud tripwires + flag de-weighting, landing page, empty states, a11y, real-Stripe smoke test. ✔ *Full demo on a phone, both themes, no dead ends.*

---

## 13. Out of scope (do not build)

Wallet/credits · subscriptions (enum only) · micro-charge batching · native apps · in-app video (sessions link out to a coach-provided URL) · intros/matchmaking · **AI features of any kind, including embeddings** · group threads · i18n (strings centralised for later) · email digests.

---

## 14. Design tokens

| Token | Light | Dark | Use |
|---|---|---|---|
| `--bg` | `#FFFBF8` | `#17131A` | app ground |
| `--surface` | `#FFFFFF` | `#201A22` | cards |
| `--ink` | `#2A2230` | `#F1EAEE` | text; client bubbles |
| `--ink-soft` | `#6E6068` | `#C4B8C0` | secondary text |
| `--line` | `#F0E4DD` | `#362E3B` | borders |
| `--accent` | `#D6456A` | `#F2718F` | money + primary CTA **only** |
| `--accent-soft` | `#FBEEF1` | `#3A2530` | answer badges |
| `--positive` | `#2F5D62` | `#7FC0C3` | free/included, success — never CTAs |
| `--positive-soft` | `#E9F2F0` | `#1E2E30` | free badge bg |
| `--gold` | `#E0A03C` | `#E0A64E` | stars only |

Serif display (`Iowan Old Style, Palatino, Georgia, serif`) for voice moments; system sans for UI; **monospace for every dollar amount, meter and status badge**. Radii: cards 14–16, sheets 22 top, pills 999. One signature motion: the answer badge ticking on fulfilment. Everything else instant.

---

*End of specification. Begin with M0.*
