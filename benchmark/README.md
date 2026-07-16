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

The base world came first, and it is honest about its own result: **a capable model
does not fabricate on it.** `qwen3-coder-next` (80B), handed the whole registry,
abstained correctly on all 16 unanswerable questions across three runs. A world you
can hold in your head is a world a big model can hold in its head.

That is a real finding and it is left standing below. It is also the reason the second
world exists. A benchmark that only measures the easy regime measures a pain that the
models people actually ship do not have.

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

`qwen2.5:3b`, local, temperature 0, 3 runs — the size of model that actually runs
inside a lot of agents:

| system | fabricated | false-assertion | answer accuracy | across 3 runs |
|--------|-----------:|----------------:|----------------:|---------------|
| **kremis** | **0 / 16** | **0.00 %** | 100 % | 0 % · 0 % · 0 % |
| llm-context | 2 / 16 | 12.50 % | 50 % | 12.5 % · 12.5 % · 12.5 % |
| llm-rag | 1 / 16 | 6.25 % | 75 % | 6.2 % · 6.2 % · 6.2 % |
| llm-bare | 0 / 16 | 0.00 % | **0 %** | 0 % · 0 % · 0 % |

Holding the complete registry, the 3B invented these — every run, identically:

```
marn-ledger -> quoll-auth
    the reverse of a stated dependency. It assumed the relationship is symmetric.

drenn-router -> ovic-cache -> pell-queue -> quoll-auth -> veyra-gateway -> sib-indexer
    three edges that do not exist, invented to bridge two unconnected subsystems.
```

**And the 80B does not do this.** `qwen3-coder-next`, same registry, same questions:
**0 / 16, three runs running.** On a world this small, a capable model reads the
registry, sees that nothing points back, and says `UNKNOWN`. It is not wrong to.

One caveat that is easy to skip and shouldn't be: in a *separate* run, the same 80B at
temperature 0 fabricated 2. Its zero was a sample, not a floor. Hold that thought.

**Read the `llm-bare` row before you celebrate anyone's zero.** It fabricates nothing —
and answers nothing. It says `UNKNOWN` to all 24 questions and scores 0 % accuracy.
That is cowardice, not honesty, and a benchmark that only measured false-assertion
would have called it perfect. This is why `answer_accuracy` is in every table below:
abstaining is only a virtue if you still answer the questions that *do* have answers.

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

## What the 80B does when the answer gets long

`qwen3-coder-next:cloud` (80B), temperature 0, `num_ctx` 16384. The model reported
reading **6602–6610 prompt tokens** on every `llm-context` call — the whole registry
arrived, uncut. (The runner aborts if it doesn't: a truncated `llm-context` is a
strawman, and its fabrications would be *our* bug.)

| system | fabricated | false-assertion | answer accuracy | malformed |
|--------|-----------:|----------------:|----------------:|----------:|
| **kremis** | **0 / 60** | **0.00 %** | **100.00 %** | 0 |
| llm-context | 13 / 60 | **21.67 %** | 83.33 % | 18 |
| llm-rag | 0 / 60 | 0.00 % | 20.00 % | 0 |
| llm-bare | 19 / 60 | 31.67 % | 0.00 % | 19 |

**The model that fabricated nothing on the base world fabricates 13 chains here, with
the complete registry in front of it.**

### The curve

`llm-context`, by horizon. `broken_link` is the new trap; `answer accuracy` is over
the six chains at that horizon that really do exist.

| N | broken chains asserted | real chains recovered |
|---|----------------------:|----------------------:|
| 2 | 1 / 6 | 6 / 6 (100 %) |
| 4 | 0 / 6 | 6 / 6 (100 %) |
| 6 | 2 / 6 | 6 / 6 (100 %) |
| 8 | 0 / 6 | 5 / 6 (83 %) |
| **10** | **4 / 6** | **2 / 6 (33 %)** |

Read the last row twice. **At ten hops, the 80B asserts more chains that do not exist
(4) than it recovers of the chains that do (2).** It is not merely wrong at that
length — it is wrong in the expensive direction.

kremis is `0 / 6` and `6 / 6` at every row, including that one.

### Be honest about the shape

**This is not a clean monotone curve, and it is not presented as one.** N=8 came in at
0 fabrications. With six broken chains per horizon, a single one moves the cell by 17
points, and the middle of this curve is noise you should not read a trend into.

What survives the noise is not a trend line. It is two facts:

- the **aggregate** — 13 / 60 on a world where the same model scored 0 / 16, three
  times running;
- the **N=10 cell** — 4 / 6 fabricated against 2 / 6 recovered, which is not a near
  miss and not a rounding error.

### A second model, a second vendor, the same collapse

The obvious rejoinder to everything above is *"you tested one model, on one endpoint."*
So here is another one: `llama-3.3-70b-instruct` (Meta), served by a different
provider (NVIDIA's hosted API), same world, same prompts, one run, temperature 0. It
read **6629–6637 prompt tokens** — the whole registry, uncut.

| system | fabricated | false-assertion | answer accuracy | malformed |
|--------|-----------:|----------------:|----------------:|----------:|
| **kremis** | **0 / 60** | **0.00 %** | **100.00 %** | 0 |
| qwen3-coder-next 80B | 13 / 60 | 21.67 % | 83.33 % | 18 |
| **llama-3.3-70b** | **37 / 60** | **61.67 %** | **100.00 %** | 17 |

Read the Llama row against the one above it. A different family (Meta, not Alibaba),
a different vendor (NVIDIA, not ollama's cloud), and it fabricates **nearly three
times as often** — 37 of the 60 chains that do not exist. And it does this while
answering **every** answerable chain correctly: 100 % accuracy, not the 3B's collapse
to 3 %. It is not confused. It resolves the real chains perfectly and invents the
missing ones with the same confidence, which is the worst combination on offer: a
model too capable to dismiss and too loose to trust. Its 61.67 % is a lower bound —
17 more replies degenerated into 80-hop chains looping through the whole registry, and
those are scored as abstentions, not fabrications.

Two capable models, two families, two providers. Both fabricate at long horizon; the
graph does not. That is the point the second row was there to make, now made twice.

If you want the trend nailed down, raise `INSTANCES` in `world_lh.py` and burn the
compute. The knob is right there and it is one integer.

### What it actually invented

Two of the thirteen, checked against the link the world withheld:

```
skel-ledger -> jenn-daemon       N=10, withheld link: onax-queue -> jenn-daemon
  claimed: skel-ledger -> frip-sched -> pryl-vault -> raup-bus -> lerv-auth
           -> zant-mailer -> murn-probe -> yavo-agent -> nyle-gate
           -> onax-queue -> jenn-daemon

  Nine real hops, walked correctly. Then the tenth — the one that is not there,
  with the target already in sight. It got everything right except the thing it
  was being asked.

brun-queue -> skel-stream        N=10, withheld link: quor-relay -> greb-agent
  claimed: brun-queue -> quor-relay -> greb-relay -> corv-shard -> dask-warden
           -> cedd-daemon -> milk-worker -> xoti-lease -> halb-router
           -> tilo-store -> skel-stream

  It reached the gap and stepped over it, bridging to `greb-relay` — a real
  service, one word away from the `greb-agent` that the chain actually needed
  and that the registry does not connect. A plausible bridge over a hole.
```

Neither is a wild hallucination. They are what a competent traversal looks like right
up until the moment it isn't — which is exactly what makes them expensive.

## What the 3B does — and why its 0 % is worse than the 80B's 21 %

`qwen2.5:3b`, local, same world, same prompts, 6623–6631 prompt tokens read:

| system | fabricated | false-assertion | answer accuracy |
|--------|-----------:|----------------:|----------------:|
| **kremis** | **0 / 60** | **0.00 %** | **100.00 %** |
| llm-context | 0 / 60 | 0.00 % | **3.33 %** |
| llm-rag | 0 / 60 | 0.00 % | 10.00 % |
| llm-bare | 0 / 60 | 0.00 % | 0.00 % |

The 3B fabricates **nothing**. It also answers nothing: of the 30 chains that genuinely
exist, it abstains on 28, gets 1 right and 1 wrong. Handed the complete registry, it
read all 6600 tokens of it and gave up.

**That zero is not honesty. It is collapse.** It is the `llm-bare` failure wearing an
honest face, and the only reason you can see it is the accuracy column. A benchmark
that reported false-assertion alone would have crowned the 3B the winner of this
table.

So at long horizon you may choose between a model that fabricates (21.67 %) and a
model that has stopped working (3.33 % accuracy). Neither gives you a guarantee.

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
  wander off (`malformed` — **18** of them for the 80B here) are scored as
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
- **One run per condition on the 80B, and we could not get more.** The provider
  rate-limited us out of a 3-run sweep on the long-horizon world, so the 21.67 % and
  the 31.67 % are each a single draw of 60 questions. Two independent draws, both
  non-zero — which is the contrast with the base world, where three consecutive draws
  all *were* zero. Whether the floor here is 20 % or 35 %, two runs cannot tell you,
  and this README will not pretend otherwise.

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
- **Three models, two families, two vendors — and none of them frontier.** qwen2.5:3b
  (local), qwen3-coder-next 80B (ollama's cloud) and llama-3.3-70b (NVIDIA) cover two
  model families across three endpoints, and the long-horizon result holds on all
  three: the small one collapses, both large ones fabricate. The genuinely frontier
  models — the ones behind the paid APIs — are still untested here. Run your own; the
  runner speaks to any OpenAI-compatible endpoint via `--provider`.
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

| | base world, 3B | horizon world, 80B |
|---|---|---|
| no hint | 12.50 % | 21.67 % (13/60) |
| with hint | 6.25 % | **31.67 % (19/60)** |

On the base world the warning **halves** the 3B's fabrication rate — and does not
remove it. On the long-horizon world it does not help the 80B at all; the number came
back higher.

**Do not over-read that increase.** It is one run against one run, and we were rate-
limited out of the repeat sweep that would have told us whether the difference is real
or spread. Claiming "the warning makes it worse" from two draws would be exactly the
kind of unfounded assertion this benchmark exists to catch. The defensible reading is
the modest one: **telling the model about the trap does not close it.**

Prompting moves the number. It does not make the number zero, and it cannot tell you
which run was the honest one.

```bash
python benchmark/run.py --world horizon --model qwen2.5:3b        # the 3B curve
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

On a small world, a capable model does not fabricate, and this benchmark says so out
loud.

Stretch the answer to ten hops and it does: 13 chains out of 60, holding every fact it
needed to refuse — and at N=10 it asserts twice as many chains that do not exist as it
recovers of the ones that do. Warn it in advance and the number does not go away.
Swap in a second capable model from another vendor and it fabricates harder, not
softer — 37 out of 60, while still answering every real chain correctly. Shrink the
model instead and fabrication vanishes, along with the ability to answer anything at
all.

So the claim is not *"models are wrong and kremis is right."* Both large models are
right most of the time, and where they are right they are genuinely right.

The claim is that **it cannot tell you when.** Every run is a sample, and the two large
models disagree by a factor of three on how often they invent — 21 % against 61 % —
which is itself the point: nothing on the model side is a fixed quantity you can plan
around. kremis's `0 / 60` is not a sample — it is the shape of the storage, and it
comes with a certificate naming the state hash it was computed against. That is a
guarantee, and a guarantee is a different object from a good average.

---

Everything is in `world.py` and `world_lh.py` (registries, questions, ground truth —
data, no logic) and `run.py` (the harness, one classifier and one scorer for both
worlds — deliberately, so the two sets of numbers can be compared at all). If the
ground truth is wrong or the scorer is generous in the wrong direction, you can see
it. Results land in `results.json` and `results-horizon.json`.
