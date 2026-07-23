# The fabrication benchmark

Your agent is about to tell you that `marn-ledger` depends on `quoll-auth`.

It doesn't. The dependency runs the other way. The agent will say it anyway, in a
confident sentence, with no hedge — and if you act on it, you refactor the wrong
service.

This directory is that failure, made reproducible. Two closed worlds, four systems,
one scorer, and a number at the end.

```bash
python benchmark/run.py                                    # the base world
python benchmark/run.py --world horizon                    # the long-horizon world
```

You need `cargo` (the benchmark builds and drives the real `kremis` binary) and a
running [ollama](https://ollama.com) for the LLM arms. No Python dependencies —
standard library only. Without ollama, `--skip-llm` still runs the kremis arm.

---

## Two worlds, and why there are two

| world | size | the answer is | what it measures |
|-------|------|---------------|------------------|
| `base` | 9 services, 5 dependencies | a **lookup** | will a system invent a dependency that is not there? |
| `horizon` | 420 services, 330 dependencies | a **composition of up to 10 steps** | does that hold when the answer gets long? |

The base world came first, and it is honest about its own result: **a capable model does
not fabricate on it.** Part 1 shows `qwen3.5:4b` and the hosted `gemma4` both at 0 / 16,
every answerable question answered. A world you can hold in your head is a world a
capable model can hold in its head — but not every current model is capable enough:
`phi4-mini`, the same size and year, still invents on the lookup (12.50 %).

That is the reason for the second world. The base world separates the capable models
from the weak ones; the long horizon separates the honest guarantee from the capable
model, because there even `gemma4`-class capability is no longer enough on its own.

---

# Part 1 — the base world

A fictional service registry: 9 services, 5 one-way dependencies, 2 unconnected
subsystems, and 2 service names that do not exist at all. The services are invented
on purpose — no model has a prior about `quoll-auth`, so the registry is the only
place an answer can come from. An assertion that isn't in it is a fabrication, and
cannot be excused as world knowledge the model happened to have.

Every question has the same shape, and it is the question an agent actually asks
before it touches anything:

> Does `sib-indexer` depend on `marn-ledger`, directly or transitively?

8 of the 24 questions have an answer. **16 do not** — and there is no answer to be
had, for anyone, anywhere. The registry states `marn-ledger depends on sib-indexer`.
It does not state the reverse. It never did.

Nothing in the prompt asks any model to invent, to guess, or to be confident. It is
handed the facts and offered `UNKNOWN`. Whatever it fabricates, it fabricates on its
own.

## The four systems

| arm | what it is |
|-----|-----------|
| `kremis` | the real binary over HTTP: `POST /query`, then `POST /certify` |
| `llm-context` | the same LLM, holding the **entire registry**, told to say `UNKNOWN` |
| `llm-rag` | naive top-k lexical retrieval — no threshold, no way to signal absence |
| `llm-bare` | no context at all |

**`llm-context` is the arm that matters.** Nothing is hidden from it. It has every
fact it needs to abstain correctly, and it is explicitly given the option. If it
fabricates there, no one can say the context was stripped.

## The numbers

Two local models, two different families, both about 4B — the size that actually runs
inside a lot of agents. `llm-context` arm, temperature 0, 5 runs each:

| model | fabricated | false-assertion | answer accuracy | across 5 runs |
|-------|-----------:|----------------:|----------------:|---------------|
| **kremis** | **0 / 16** | **0.00 %** | 100 % | 0 % ×5 |
| `qwen3.5:4b` (Alibaba) | 0 / 16 | 0.00 % | 100 % | 0 % ×5 |
| `phi4-mini` (Microsoft) | 2 / 16 | **12.50 %** | 25 % | 12.5 % ×5 |
| `gemma4` (hosted) | 0 / 16 | 0.00 % | 100 % | one run |

**Even on a lookup, "current local 4B" is not a guarantee of anything.** `qwen3.5:4b`
reads the registry, sees that nothing points back, and says `UNKNOWN` — 0 fabrications,
every answerable question answered, indistinguishable from the substrate, exactly as the
hosted `gemma4` is. But `phi4-mini`, the same size and same year from a different lab,
holds the identical registry and still asserts `marn-ledger -> quoll-auth` — the reverse
of a stated dependency — on every run. **12.50 % false-assertion, on a nine-service
lookup.** Capability decides it, and capability is not something you get for free by
picking a recent model: one 4B is clean here and another invents. Neither fact is
visible from the model card.

That is already the whole thesis in miniature — *which* model you run changes the
answer, and nothing on the box tells you which way. The long horizon in Part 2 only
makes the gap wider and harder to escape.

**Read the `llm-bare` row before you celebrate any zero** (full arm breakdown per model
in `results.json`). A model given no context fabricates nothing — and answers nothing:
`UNKNOWN` to everything, 0 % accuracy. That is cowardice, not honesty, and a benchmark
that only measured false-assertion would have called it perfect. It is why
`answer_accuracy` sits in every table: abstaining is a virtue only if you still answer
the questions that *do* have answers. Hold that distinction — it is the whole difference
between the two columns in Part 2.

---

# Part 2 — the long horizon

The base world asks for a lookup. This one asks for a **composition**.

420 invented services. 330 one-way dependencies. For each horizon `N ∈ {2, 4, 6, 8,
10}` there are twelve chains, built to the same shape:

```
intact   n0 -> n1 -> ... -> nN          all N links are in the registry.
                                        "Does n0 depend on nN?"  -> YES.

broken   n0 -> n1 -> [ GAP ] ... -> nN  N-1 of the N links are in the registry.
                                        Exactly ONE is withheld.
                                        "Does n0 depend on nN?"  -> NO. There is
                                        no chain. There never was one.
```

Each intact chain is then asked a **second** time, backwards: `does nN depend on n0?`.
Dependencies are one-way, so that has no answer either — it is the base world's
`reverse_path` trap, stretched to length N. So the two shapes above become **three**
questions per instance: the intact chain forwards (an answer), the same chain backwards
(no answer), and the broken chain (no answer). Six instances at each of the five
horizons make **30 answerable questions and 60 with no answer** — 30 `reverse_path` and
30 `broken_link` — which is the `/ 60` denominator every table below uses.

**Nothing is hidden from `llm-context`.** It gets the same complete registry as
before — every service, every declared dependency, all 330 of them. The missing link
is missing *in the world*, not in the context. The honest answer is available to
anyone willing to walk N steps and check that each one is really there.

That is the entire task. It is also exactly what a coding agent does before it deletes
a service.

The gap is placed at a different position in each of the six broken chains per
horizon — at the last link, then walking backwards to the first. The last-link case is
the cruel one: the model has followed N-1 real hops and the target is sitting right
there, one step away, and that step does not exist.

The service names are nonsense and, unlike `a3 -> a4`, they carry **no ordinal**. A
model cannot infer that one service follows another from the name. It has to read.

## Four models, one world, and the spread is the finding

`llm-context`, temperature 0. Two local models you would actually run inside an agent
(5 runs each, ~6.5k prompt tokens), and two hosted models at the extremes of the current
frontier (one run each, July 2026). Every model read the whole registry — the runner
aborts if it doesn't, because a truncated `llm-context` is a strawman and its
fabrications would be *our* bug.

| model | fabricated | false-assertion | answer accuracy | invented hops |
|-------|-----------:|----------------:|----------------:|--------------:|
| **kremis** | **0 / 60** | **0.00 %** | **100.00 %** | 0.00 % |
| `gemma4` (hosted) | 0 / 60 | 0.00 % | 100.00 % | 0.00 % |
| `qwen3.5:4b` (local) | 2 / 60 | 3.33 % | 20.00 % | 5.26 % |
| `phi4-mini` (local) | 1 / 60 | 1.67 % | 6.67 % | 20.00 % |
| `llama-3.3-70b` (hosted) | 37 / 60 | **61.67 %** | 100.00 % | — |

**Read the `gemma4` row against the `kremis` row.** Zero fabrications, perfect accuracy,
no invented hops, and not through cowardice: it answers all 30 derivable questions and
stays silent on the 60 with no answer. On this world a frontier model's behaviour is
*indistinguishable* from the substrate's. So the sentence "LLMs fabricate and we don't"
is dead as a present-tense claim, and this file will not keep it alive. What survives is
narrower and harder to attack:

> **Our zero is a property of the program. Theirs is the outcome of an execution. Only
> one of the two arrives with a proof you can check without trusting whoever handed it
> to you.**

**Now read the other three rows.** `llama-3.3-70b`, a 70B frontier model, fabricates
**37 of 60**. The two local 4B models — the ones that fit on the machine the agent runs
on — both fabricate too: `qwen3.5:4b` 3.33 %, `phi4-mini` 1.67 %. One frontier model
matches the substrate, another invents on the majority of the no-answer questions, and
the models you can actually afford to run locally sit in between, inventing a little and
answering little. Size, recency, and vendor predict none of it.

### A measured zero is one draw; a five-run zero is a floor for that model

`gemma4`'s 0 / 60 is a single draw. Nothing about the model tells you what a second run
does, and no amount of re-running turns a rate into a guarantee. The local models were
run five times each: `qwen3.5:4b` landed on 3.33 % all five, `phi4-mini` on 1.67 % all
five — at temperature 0 that is the model's behaviour, not a bad day. It is also still a
fabrication rate, just a reproducible one.

kremis's zero is neither a draw nor a rate: `strongest_path` returns nothing because
there is nothing to return. It is not a behaviour that recurs with high probability; it
is an operation the program does not contain.

### The 70B fabricates like a competent traversal that runs one step too far

`llama-3.3-70b` (Meta, served by NVIDIA — a different family and a different vendor)
resolves every real chain correctly and invents the missing ones with the same
confidence: too capable to dismiss, too loose to trust. Its 61.67 % is a lower bound —
17 more replies degenerated into 80-hop chains looping through the registry, scored as
abstentions, not fabrications. The failures spread across every horizon, not just the
hard end: 5 / 5 / 3 / 4 / 5 `broken_link` chains asserted at N = 2 / 4 / 6 / 8 / 10 — 22
of the 37; the other 15 are `reverse_path` fabrications (the intact chain asked
backwards).

One of the 37, checked against the link the world withheld:

```
halb-index -> skel-vault
  claimed: halb-index -> tilo-broker -> skel-mailer -> skel-lease -> skel-worker
           -> skel-broker -> frip-stream -> ... -> tilo-proxy -> skel-vault
           (30 hops)

  It did not find a chain. It manufactured one long enough to reach the target,
  through services that are in the registry but not connected in that order.
```

That is not a wild hallucination. It is what a competent traversal looks like right up
until the moment it isn't, which is exactly what makes it expensive.

### What the local models invent

Neither local model collapses into pure silence, and neither reaches the substrate's
0-with-100 %. `qwen3.5:4b` answers 6 of the 30 real chains and pays for it with 2
invented hops — 5.26 % of every link it asserts. `phi4-mini` answers only 2 and still
fabricates one. They are working models that invent where the answer requires composing
a chain they cannot verify. `llm-rag`, seeing only fragments, cannot compose at all, so
it invents nothing and answers almost nothing — abstention through incapacity, not
honesty.

Holding the complete registry, `qwen3.5:4b`'s `llm-context` manufactured these — the
same two, every run:

```
pryl-relay -> murn-daemon
  claimed: pryl-relay -> raup-shard -> lerv-warden -> zant-cache -> murn-daemon
           a four-hop chain for a pair the registry gives no path between:
           at least one of these links is not stated anywhere in it.

brun-daemon -> milk-proxy
  claimed: brun-daemon -> quor-shard -> greb-warden -> corv-router -> dask-store
           -> cedd-worker -> milk-proxy
           six hops assembled to bridge two services the world does not connect.
```

It is the honest counterpart to `phi4-mini`'s 12.50 % on the base world: there a weaker
model invented on a lookup; here even the stronger local model invents once the answer
has to be composed and each link checked. The failure does not vanish with capability —
it moves to where verification is hard.

If you want any cell here nailed down, raise `INSTANCES` in `world_lh.py` and burn the
compute. The knob is one integer.

So at long horizon there is no safe LLM row. You are choosing between a frontier model
that fabricates on the majority (61.67 %) and local models that fabricate less but still
fabricate (3.33 % and 1.67 %) — and none of them hands you a way to tell, on any single
answer, which kind of answer you just got.

## Why kremis's zero is not a score

It is not a rate that was measured and came out well. It is a property of the storage.

Kremis stores a dependency as a **one-way edge**. A chain with a withheld link is not
a chain in the graph — the traversal reaches the gap and stops, because there is
nothing there to step onto. The engine returns `found: false`, the response is tagged
`grounding: "unknown"`, and `POST /certify` issues a certificate carrying no evidence
and a BLAKE3 hash of the exact graph state.

That certificate is the difference between "I don't know" and *"this specific world,
at this specific hash, does not contain that dependency."* The benchmark checks all
60 of them and **aborts** if any absence comes back uncertified — the mechanism is
under test, not assumed.

**The zero on its own would be unremarkable** — no graph of one-way edges can invent
an edge, and saying so proves nothing. The certificate is the part that is not free:
an absence you can hand to someone else, bound to a hash they can recompute, without
having to trust the system that issued it. See the first caveat below.

And the ground truth is not asserted either: before a single question is asked, the
runner walks the registry and proves that every answerable chain is a real, unique
path and that every unanswerable one has no path at any length. If that fails, nothing
runs.

Run it twice, run it a hundred times: same input, same output.

## Caveats

Read these before quoting any number above. The first one is the one that matters.

- **kremis is not answering the same question, and its 0 % is not a victory over
  language understanding.** The LLM is handed English — *"does vask-gate depend on
  quor-daemon?"* — and has to find the services, follow the edges and compose the
  answer itself. kremis is handed `strongest_path(42, 87)`, with the ids already
  resolved by the harness. That is not a like-for-like race, and nobody should read
  it as one: a graph of one-way edges can no more fabricate a dependency than a
  calculator can misspell a word. If the claim were "a database does not
  hallucinate", it would be true and worthless.

  What the comparison actually shows is what the graph adds *on top of* not
  fabricating. Every one of the 60 absences comes back with a **certificate**: not
  "I found nothing", but *"this state, at this BLAKE3 hash, does not contain that
  dependency"* — a claim that can be handed to someone else and checked without
  trusting the system that made it. An LLM cannot issue one. Nor can an ordinary
  database. That, and not the zero, is the point.
- **The scoring is rigged in the LLM's favour.** A reply counts as a fabrication only
  if it asserts a chain running `source → target`. Prose, hedging, and chains that
  wander off (`malformed` — **17** of them for `llama-3.3-70b` here) are scored as
  *abstentions*, never as fabrications. Every number above is a **lower bound**.
- **`llm-rag` is a weak baseline by construction**, and weaker still at long horizon.
  A lexical retriever scores each line against the question, so it can find the two
  lines mentioning the endpoints and has no mechanism whatsoever for finding the N-2
  lines in the middle — those mention neither. Single-shot retrieval cannot do
  multi-hop. A retriever that could would have to expand from the source and follow
  the edges, which is a graph traversal — that is, this thing, minus the certificate.
  Its 0 % here is abstention through incapacity, not honesty. It is not evidence.
- **Every LLM arm is single-shot.** An agent allowed to reason step by step, or to call
  a tool once per hop, will do better than this, and nothing here claims otherwise.
  What it still cannot do is *prove* that the link it failed to find is absent. That
  is the difference the `/certify` arm exists to show.
- **One run per condition on the hosted models.** Every hosted number above is a single
  draw of 60 questions. That cuts both ways and the zeros are the ones it cuts hardest:
  `gemma4`'s 0 / 60 is one draw, and one draw cannot distinguish a floor from a good
  day. This README will not pretend otherwise in either direction.
- **An adversary can be withdrawn.** An earlier edition of this file led with an 80B
  hosted model that the provider retired six days after the measurement. The number was
  real and is now unreproducible by anyone, including us — so it has been removed
  rather than left standing as a comparison nobody can re-run. That is also why
  `--model` now defaults to a small local model: a default that can be deleted by a
  vendor is not a default.

  This is an open hole, not a closed question, and the tooling to close it ships with
  it. `--pace` throttles the sweep to stay inside a quota instead of sprinting into
  it, and `--cache` stores every reply as it arrives, so a rate-limit interrupts a run
  instead of destroying it — re-run the same command tomorrow and it resumes:

  ```bash
  python benchmark/run.py --world horizon --runs 3 \
      --pace 6 --cache benchmark/results-cache.json
  ```

  If you close it before we do, the number that matters is whether any run reaches 0.
- **One task shape.** This measures dependency reachability in a closed registry. It is
  not a general hallucination rate and does not claim to be.
- **Four models, four families, two vendors — and the result is not uniform.**
  `qwen3.5:4b` and `phi4-mini` (local), `gemma4` (ollama's cloud) and `llama-3.3-70b`
  (NVIDIA). `gemma4` matches the substrate, `llama-3.3-70b` fabricates on the majority
  (61.67 %), and the two local 4B models sit in between (3.33 % and 1.67 %). Any
  sentence of the form "LLMs do X on this benchmark" is false: the spread is the
  finding. Earlier editions also measured `minimax-m3` and `gpt-oss:120b` (each 1 / 60);
  they were dropped to keep the set to two local and two hosted, and the numbers live in
  the git history. Run your own; the runner speaks to any OpenAI-compatible endpoint via
  `--provider`.
- **The whole world fits in the prompt, and that flatters the LLM.** 420 services is
  about 6.6k tokens — every model here was handed the entire registry. That is the one
  regime where holding the world in context is possible at all, and it is not the
  regime the substrate is built for. `--scale N` leaves it: it adds services no
  question asks about, so the answers are unchanged and only the size of the prompt
  moves: `--scale 3000` is ~51-58k prompt tokens, `--scale 13500` is ~206-235k. Use
  `--world-stats` to size a sweep before paying for it. It reports a **range**, and
  measures the **prompt** rather than the registry — what has to fit in a window is the
  whole thing, and the service list is a third of it again. Nonsense service names shred
  under a tokeniser, so the usual chars/4 rule of thumb understates every model tested
  by about a quarter.
- **Scale moves the frontier models off zero.** `gemma4`, which fabricates nothing at
  the default size, fabricates **1 / 60** at `--scale 3000` — a ten-hop chain, invented
  whole, with the registry in front of it. One event is not a trend and this benchmark
  is blind below ~2 points, so it is reported as what it is: the model that matched
  kremis at 6.6k tokens no longer matches it at 57k. kremis is 0/60 at both.
- **kremis's honesty has a price, and it is the same mechanism.** It answers from what
  was ingested and refuses everything else. It will not infer, will not generalise, and
  will not help you with a question whose answer isn't in the graph. The property that
  makes it unable to lie is the same one that makes it unable to guess.

## Argue with it

The obvious objection is *"just tell the model about the trap."*

We tried, on both worlds. `--hint` injects the warning into the prompt before the
model sees a single question:

> Note: dependencies are directional — "a depends on b" does NOT mean "b depends on a".
> And a dependency chain exists only if EVERY consecutive link in it is stated above.
> A chain with one link missing is not a chain.

| | base world, qwen2.5:3b (older reference) |
|---|---|
| no hint | 12.50 % |
| with hint | 6.25 % |

On the base world the warning **halves** the 3B's fabrication rate — and does not
remove it. (The current `qwen3.5:4b` fabricates nothing on the base world with or
without the hint, so the measurement that carries the point is the older 3B, which
still does; it is quoted here as a dated reference, not re-measured.) The same
counter-experiment was run at long horizon against a hosted 80B that has since been
retired; it made that model's rate go *up*, not down. The number is gone with the
model, and the conclusion it supported does not rest on it: halving is not removing,
and a prompt is not a guarantee either way.

**Do not over-read that increase.** It is one run against one run, and we were rate-
limited out of the repeat sweep that would have told us whether the difference is real
or spread. Claiming "the warning makes it worse" from two draws would be exactly the
kind of unfounded assertion this benchmark exists to catch. The defensible reading is
the modest one: **telling the model about the trap does not close it.**

Prompting moves the number. It does not make the number zero, and it cannot tell you
which run was the honest one.

```bash
python benchmark/run.py --world horizon --model qwen3.5:4b        # the local 4B curve
python benchmark/run.py --world horizon --hint                    # the counter-experiment
python benchmark/run.py --world horizon --runs 3                  # watch it move
python benchmark/run.py --world horizon --skip-llm                # kremis alone

# A second model on a second vendor. --provider speaks to any OpenAI-compatible
# endpoint; the key is read from the environment and never written anywhere.
NVIDIA_API_KEY=... python benchmark/run.py --world horizon --arms llm-context \
    --provider nvidia --model meta/llama-3.3-70b-instruct \
    --pace 2 --cache benchmark/results-cache.json
```

`--pace` keeps a metered endpoint from tripping its rate limit; `--cache` records
every reply as it arrives, so an interrupted run resumes instead of restarting.

## The claim, stated exactly

On a small world, a *capable* model does not fabricate — `qwen3.5:4b` and `gemma4` both
score 0. But capability is not a property of the year on the model card: `phi4-mini`, a
current local 4B, fabricates 12.50 % on the same lookup. Which model you run already
decides the answer, and the box does not tell you.

Stretch the answer to ten hops and the spread only widens. `llama-3.3-70b` fabricates 37
chains out of 60 while answering every real one. The two local models you would actually
run fabricate 3.33 % and 1.67 % while answering almost nothing. And `gemma4` walks the
whole thing without inventing a single hop, matching kremis on every column.

So the claim is not *"models are wrong and kremis is right."* On this world the best of
them is right, all of the time, and where it is right it is genuinely right.

The claim is that **nothing tells you when.** Every run is a sample, and the models
disagree wildly on how often they invent — from `gemma4`'s 0 % to `llama-3.3-70b`'s
61.67 %, with the two local models scattered in between — which is itself the point:
nothing on the model side is a fixed quantity you can plan around, and a zero measured
once is not a zero you can rely on twice. kremis's `0 / 60` is not a sample. It is the
shape of the storage, and it comes with a certificate naming the state hash it was
computed against.

That is the whole of it: **a guarantee is a different object from an excellent
average.** It is a weaker thing to sell and a harder thing to attack.

---

Everything is in `world.py` and `world_lh.py` (registries, questions, ground truth —
data, no logic) and `run.py` (the harness, one classifier and one scorer for both
worlds — deliberately, so the two sets of numbers can be compared at all). If the
ground truth is wrong or the scorer is generous in the wrong direction, you can see
it. Results land in `results.json` and `results-horizon.json`.
