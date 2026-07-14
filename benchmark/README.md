# The fabrication benchmark

Your agent is about to tell you that `marn-ledger` depends on `quoll-auth`.

It doesn't. The dependency runs the other way. The agent will say it anyway, in a
confident sentence, with no hedge — and if you act on it, you refactor the wrong
service.

This directory is that failure, made reproducible. One command, four systems, the
same 24 questions, and a number at the end.

```bash
python benchmark/run.py --model qwen2.5:3b --runs 3
```

You need `cargo` (the benchmark builds and drives the real `kremis` binary) and a
running [ollama](https://ollama.com) for the LLM arms. No Python dependencies —
standard library only. Without ollama, `--skip-llm` still runs the kremis arm.

---

## What is being asked

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

Holding the complete registry, the model invented these — every run, identically:

```
marn-ledger -> quoll-auth
    the reverse of a stated dependency. It assumed the relationship is symmetric.

drenn-router -> ovic-cache -> pell-queue -> quoll-auth -> veyra-gateway -> sib-indexer
    three edges that do not exist, invented to bridge two unconnected subsystems.
```

**Read the `llm-bare` row before you celebrate.** It fabricates nothing — and answers
nothing. It says `UNKNOWN` to all 24 questions and scores 0 % accuracy. That is
cowardice, not honesty, and a benchmark that only measured false-assertion would
have called it perfect. This is why `answer_accuracy` is in the table: abstaining is
only a virtue if you still answer the questions that *do* have answers. kremis
abstains 16 times and still answers all 8 correctly.

## Why kremis's zero is not a score

It is not a rate that was measured and came out well. It is a property of the
storage.

Kremis stores a dependency as a **one-way edge**. `marn-ledger → sib-indexer` puts
nothing in the graph pointing back. When the query `strongest_path(sib-indexer,
marn-ledger)` runs, there is no path to find — not because a threshold rejected it,
but because it was never there. The engine returns `found: false`, the response is
tagged `grounding: "unknown"`, and `POST /certify` issues a certificate carrying no
evidence and a BLAKE3 hash of the exact graph state.

That certificate is the difference between "I don't know" and *"this specific world,
at this specific hash, does not contain that dependency."* The benchmark checks all
16 of them and **aborts** if any absence comes back uncertified — the mechanism is
under test, not assumed.

Run it twice, run it a hundred times: same input, same output.

## Caveats

Read these before quoting any number above.

- **A bigger model resists this.** `qwen3-coder-next` (80B, cloud) holding the full
  registry scored **0 % across 3 runs** — it abstained correctly on all 16. In a
  separate run it fabricated 2. That jitter at temperature 0 is itself the finding:
  its zero is a sample, not a guarantee. kremis's zero is a guarantee. Both
  statements are in this README because both are true.
- **The scoring is rigged in the LLM's favour.** A reply counts as a fabrication only
  if it asserts a chain running `source → target`. Prose, hedging, and chains that
  wander off are scored as *abstentions*, never as fabrications. The numbers are a
  lower bound. Run with `--runs 3` and read the raw replies in `results.json`.
- **`llm-rag` is a weak baseline by construction** — a lexical retriever with no
  threshold and no absence signal. Beating it proves little. It is in the table for
  completeness, not as evidence.
- **One partition, one task shape.** This measures dependency reachability in a closed
  registry. It is not a general hallucination rate and does not claim to be.
- **kremis's honesty has a price, and it is the same mechanism.** It answers from what
  was ingested and refuses everything else. It will not infer, will not generalise,
  and will not help you with a question whose answer isn't in the graph. The property
  that makes it unable to lie is the same one that makes it unable to guess.

## Argue with it

The obvious objection is *"just tell the model that dependencies are one-way."*

We tried. `--hint-direction` injects that warning into the prompt, before the model
sees a single question:

> Note: dependencies are directional. "a depends on b" does NOT mean "b depends on a".

On `qwen2.5:3b` it **halves** the fabrication rate — 12.50 % → 6.25 % — and does not
remove it. Warned about the exact trap it was walking into, the model walked into it
anyway. Prompting moves the number. It does not make the number zero, and it cannot
tell you which run was the honest one.

Run it yourself:

```bash
python benchmark/run.py --model qwen2.5:3b --runs 3 --hint-direction
```

Other things worth doing:

```bash
python benchmark/run.py --model <your model>   # the number for the model you ship
python benchmark/run.py --skip-llm             # kremis alone, no ollama needed
```

Everything is in `world.py` (the registry, the questions, the ground truth — data,
no logic) and `run.py` (the harness and the scoring rule). If the ground truth is
wrong or the scorer is generous in the wrong direction, it is 200 lines and you can
see it. Results land in `results.json`.
