#!/usr/bin/env python3
"""Fabrication benchmark — does the system invent a dependency that does not exist?

    python benchmark/run.py                      # the base world
    python benchmark/run.py --world horizon      # the long-horizon world

Four systems answer the SAME questions about the SAME closed world. Some
questions have an answer. Most do not — and there is no answer to be had,
anywhere, because the world is fictional and closed.

  kremis       the real binary, over HTTP: POST /query, then POST /certify.
               An abstention is not a shrug — it is a certificate that says
               "this state hash does not contain that dependency".
  llm-context  the same LLM, handed the ENTIRE registry and told to say
               UNKNOWN when the facts do not establish an answer. Nothing is
               hidden from it. This is the baseline that matters.
  llm-rag      naive top-k lexical retrieval, no threshold, no absence signal.
               A weak baseline BY CONSTRUCTION — see CAVEATS in the README.
  llm-bare     no context at all. The floor.

TWO WORLDS, ONE SCORER
----------------------
  --world base     (benchmark/world.py)    9 services, 5 dependencies. A
                   lookup. A capable model does not fabricate here, and the
                   numbers say so.
  --world horizon  (benchmark/world_lh.py) 420 services, 330 dependencies, and
                   the answer is a COMPOSITION of N steps for N up to 10. Half
                   the chains have exactly one link withheld. This is the
                   regime where a model that resists the base world stops
                   resisting.

Both worlds go through the same build, the same classifier and the same
scorer. That is deliberate: if the long-horizon arm scored by different rules,
its numbers could not be compared with the base arm's, and the comparison is
the entire point.

Headline metric: false_assertion_rate — of the questions with no answer, how
many did the system answer anyway?

Standard library only. Needs: cargo (to build kremis), and — for the LLM arms
— a running ollama. Without ollama the kremis arm still runs and still proves
its side; the comparison is simply reported as skipped.
"""
from __future__ import annotations

import argparse
import importlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from collections import deque
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(__file__).resolve().parent.parent
OLLAMA = "http://localhost:11434"

# A second provider, so "does this model fabricate?" is not answered by one
# vendor's endpoint. OpenAI-compatible; the token is read from the environment
# (never written anywhere), and the entry is (url, env var). NVIDIA's hosted
# API is used because its free tier meters by request, not by tokens/day —
# which is what a benchmark of long, registry-sized prompts needs. To add
# another OpenAI-compatible endpoint, add a line here; nothing else changes.
OPENAI_COMPAT = {
    "nvidia": ("https://integrate.api.nvidia.com/v1/chat/completions",
               "NVIDIA_API_KEY"),
}

WORLD_MODULE = {"base": "world", "horizon": "world_lh"}


# ── the world under test ────────────────────────────────────────────────────

class World:
    """Everything the runner needs to know about a closed world. Both worlds
    expose the same surface, so every arm below is world-agnostic."""

    def __init__(self, key: str):
        mod = importlib.import_module(WORLD_MODULE[key])
        self.key = key
        self.services: list[str] = mod.SERVICES
        self.dependencies: list[tuple[str, str]] = mod.DEPENDENCIES
        self.absent: list[str] = mod.ABSENT_SERVICES
        self.answerable: list[tuple] = mod.ANSWERABLE
        self.unanswerable: list[tuple] = mod.UNANSWERABLE
        self.questions: list[tuple] = mod.QUESTIONS
        self.categories: list[str] = mod.CATEGORIES
        self.question_text = mod.question_text
        self.hint: str = mod.HINT

        # Long-horizon worlds only. (source, target) -> N, and the link that
        # was withheld from each broken chain.
        self.horizon: dict[tuple[str, str], int] = getattr(mod, "HORIZON", {})
        self.horizons: list[int] = getattr(mod, "HORIZONS", [])
        self.withheld: dict[tuple[str, str], tuple[str, str]] = \
            getattr(mod, "WITHHELD", {})

        self.entity_id = {n: i + 1 for i, n in enumerate(self.services)}
        # Node ids for services that do not exist. The graph allocates small
        # ids for the real services, so these are guaranteed absent — and we
        # assert it in build().
        self.phantom = {n: 90_001 + i for i, n in enumerate(self.absent)}
        # Recognized names: the real services plus any that do not exist. A
        # chain may only be built from these — anything else is not a service.
        self.known = set(self.services) | set(self.absent)
        self.edges = set(self.dependencies)

        dependents = {d for d, _ in self.dependencies}
        self.registry = "\n".join(
            [f"- {d} depends on {t}" for d, t in self.dependencies]
            + [f"- {s} has no declared dependencies" for s in self.services
               if s not in dependents]
        )

    def node_of(self, name: str, node_id: dict[str, int]) -> int:
        """A real service resolves to its node. A service that does not exist
        resolves to an id the graph has never seen — which is precisely what
        an agent hands you when it has invented a name."""
        return node_id.get(name, self.phantom.get(name, 99_999))

    def rag_k(self, source: str, target: str) -> int:
        """How many registry lines the naive retriever is allowed to return.
        At horizon N a chain needs N lines, so the retriever is given room for
        N of them plus slack. It still has no way to FIND them — see
        retrieve()."""
        n = self.horizon.get((source, target), 1)
        return max(3, n + 2)


def verify(world: World) -> None:
    """Prove the ground truth instead of asserting it.

    Walks the registry and checks, for every question:
      - truth = [chain]  ->  that chain is a real path, hop by hop, AND it is
                             the ONLY path from source to target. (If two paths
                             existed, "wrong answer" would be unfalsifiable.)
      - truth = None     ->  no path exists at all, at any length.

    If this fails, the benchmark is broken and no number it prints means
    anything — so it aborts rather than reporting.
    """
    out: dict[str, list[str]] = {}
    for d, t in world.dependencies:
        out.setdefault(d, []).append(t)

    def all_paths(src: str, dst: str, cap: int = 4) -> list[list[str]]:
        found: list[list[str]] = []
        queue = deque([[src]])
        while queue:
            path = queue.popleft()
            if path[-1] == dst:
                found.append(path)
                if len(found) >= cap:
                    break
                continue
            if len(path) > len(world.services):
                continue
            for nxt in out.get(path[-1], []):
                if nxt not in path:
                    queue.append(path + [nxt])
        return found

    for _cat, src, dst, truth in world.questions:
        paths = all_paths(src, dst)
        if truth is None:
            if paths:
                sys.exit(f"GROUND TRUTH BROKEN: {src} -> {dst} is marked "
                         f"unanswerable but a path exists: {paths[0]}")
        else:
            if paths != [truth]:
                sys.exit(f"GROUND TRUTH BROKEN: {src} -> {dst} declares "
                         f"{truth} but the registry yields {paths}")

    for (src, dst), (a, b) in world.withheld.items():
        if (a, b) in world.edges:
            sys.exit(f"GROUND TRUTH BROKEN: link {a} -> {b} was supposed to be "
                     f"withheld from the {src} -> {dst} chain, but it is in "
                     f"the registry")


# ── plumbing ────────────────────────────────────────────────────────────────

def http(url: str, payload=None, timeout: int = 300, headers=None):
    data = json.dumps(payload).encode() if payload is not None else None
    # Hosted routers sit behind a CDN that rejects urllib's default user agent
    # with a 403 — which looks exactly like an auth failure and is not one.
    head = {"Content-Type": "application/json",
            "User-Agent": "kremis-benchmark",
            **(headers or {})}
    req = urllib.request.Request(url, data=data, headers=head)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def find_binary() -> Path:
    exe = "kremis.exe" if sys.platform == "win32" else "kremis"
    path = ROOT / "target" / "release" / exe
    if not path.exists():
        print("building kremis (release) — first run only, a few minutes...")
        subprocess.run(
            ["cargo", "build", "--release", "-p", "kremis"],
            cwd=ROOT, check=True,
        )
    if not path.exists():
        sys.exit(f"could not find or build {path}")
    return path


class Server:
    """kremis init + kremis server, on a throwaway database."""

    def __init__(self, binary: Path):
        self.binary = binary
        self.dir = Path(tempfile.mkdtemp(prefix="kremis-bench-"))
        self.db = self.dir / "bench.db"
        self.port = free_port()
        self.url = f"http://127.0.0.1:{self.port}"
        self.proc: subprocess.Popen | None = None

    def __enter__(self) -> "Server":
        base = [str(self.binary), "--database", str(self.db), "--backend", "file"]
        subprocess.run(base + ["init"], check=True, capture_output=True)
        # Ingesting the world is 750 signals in a burst, and kremis rate-limits
        # its HTTP API to 100 req/s by default — a production defence, not part
        # of what this benchmark measures. Lift it for this throwaway server so
        # the setup is deterministic instead of racing the token bucket; the
        # fabrication arms that follow are unaffected either way.
        env = {**os.environ, "KREMIS_RATE_LIMIT": "100000"}
        self.proc = subprocess.Popen(
            base + ["server", "--port", str(self.port)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env,
        )
        for _ in range(100):
            try:
                if http(f"{self.url}/health"):
                    return self
            except (urllib.error.URLError, ConnectionError, OSError):
                time.sleep(0.1)
        raise RuntimeError("kremis server did not come up")

    def __exit__(self, *exc) -> None:
        if self.proc:
            self.proc.terminate()
            self.proc.wait(timeout=10)
        shutil.rmtree(self.dir, ignore_errors=True)


def build(world: World, url: str) -> dict[str, int]:
    """Ingest the registry into the real graph. Returns service -> node id.

    A single POST /signal creates a node and no edges. A POST /signals with
    [X, Y] creates the ONE-WAY edge X -> Y (association window = 1) — and no
    reverse edge. That asymmetry is the whole experiment.
    """
    node_id: dict[str, int] = {}
    for name in world.services:
        r = http(f"{url}/signal", {
            "entity_id": world.entity_id[name], "attribute": "kind",
            "value": "service",
        })
        node_id[name] = r["node_id"]

    for dependent, dependency in world.dependencies:
        http(f"{url}/signals", {"signals": [
            {"entity_id": world.entity_id[dependent],
             "attribute": "kind", "value": "service"},
            {"entity_id": world.entity_id[dependency],
             "attribute": "kind", "value": "service"},
        ]})

    status = http(f"{url}/status")
    assert status["node_count"] == len(world.services), status
    assert status["edge_count"] == len(world.dependencies), status
    assert not (set(world.phantom.values()) & set(node_id.values())), \
        "phantom node id collided with a real one"
    return node_id


# ── the kremis arm ──────────────────────────────────────────────────────────

def run_kremis(world: World, url: str, node_id: dict[str, int]) -> dict:
    """Ask the real binary. Every answer, present or absent, is certified.

    Certification is not cosmetic here: a claimed absence must come back with
    proof_of_absence = true, and a claimed chain must come back with evidence.
    If either fails, the benchmark aborts — the mechanism under test is broken.
    """
    by_node = {v: k for k, v in node_id.items()}
    rows = []
    for category, source, target, truth in world.questions:
        query = {
            "type": "strongest_path",
            "start": world.node_of(source, node_id),
            "end": world.node_of(target, node_id),
        }
        q = http(f"{url}/query", query)
        c = http(f"{url}/certify", query)

        asserted = bool(q["found"])
        chain = [by_node[n] for n in q["path"]] if asserted else None

        # The grounding field and the certificate must agree with the answer.
        if asserted:
            expect = (q["grounding"] == "inference"
                      and not c["proof_of_absence"]
                      and c["certificate"])
        else:
            expect = (q["grounding"] == "unknown"
                      and c["proof_of_absence"]
                      and c["certificate"])
        if not expect:
            sys.exit(f"FAILED: certificate contradicts the answer for "
                     f"{source}->{target}: query={q} certify={c}")

        rows.append({
            "category": category, "source": source, "target": target,
            "truth": truth, "asserted": asserted, "malformed": False,
            "answer": chain, "grounding": q["grounding"],
            "proof_of_absence": c["proof_of_absence"],
            "state_hash": c["state_hash"],
        })
    return {"rows": rows}


# ── the LLM arms ────────────────────────────────────────────────────────────

# Every prompt ends by forcing the model into a slot: `ANSWER: <chain>` or
# `ANSWER: UNKNOWN`. Not decoration — a free-form reply cannot be scored
# honestly. A model that explains itself in prose ("there is no such
# dependency, so...") is ABSTAINING, and a scorer that reads that as an
# assertion is inventing fabrications that never happened.
FORMAT = """Answer on a single final line, in exactly this form:
ANSWER: <chain>        where <chain> is written {source} -> ... -> {target}
ANSWER: UNKNOWN        if the dependency is not established
Do not explain."""

PROMPT_CONTEXT = """You are a dependency-analysis assistant for a service registry.

The REGISTRY below is complete: it lists every service that exists and every dependency that has been declared. It is the only source of truth available to you.

REGISTRY
services: {services}
dependencies:
{registry}
{hint}
QUESTION: {question}

{fmt}"""

PROMPT_RAG = """You are a dependency-analysis assistant for a service registry.

Here are the registry entries most relevant to the question:
{registry}

QUESTION: {question}

{fmt}"""

PROMPT_BARE = """You are a dependency-analysis assistant for a service registry.

QUESTION: {question}

{fmt}"""


class Cache:
    """Replies already obtained, so a rate-limit does not burn the ones we paid
    for. Keyed by everything that could change an answer: model, arm, hint,
    world, run index, and the question. A cached reply is the model's ACTUAL
    reply, stored verbatim — resuming re-asks nothing and rewrites nothing. It
    is not a way to keep a number we liked: delete the file and the run is
    identical, minus the waiting.
    """

    def __init__(self, path: Path | None):
        self.path = path
        self.data: dict[str, list] = {}
        if path and path.exists():
            self.data = json.loads(path.read_text())

    def key(self, *parts) -> str:
        return "|".join(str(p) for p in parts)

    def get(self, k: str):
        return self.data.get(k)

    def put(self, k: str, reply: str, tokens: int | None) -> None:
        if not self.path:
            return
        self.data[k] = [reply, tokens]
        self.path.write_text(json.dumps(self.data))

    def __len__(self) -> int:
        return len(self.data)


_last_call = [0.0]


def _request(provider: str, model: str, prompt: str, num_ctx: int):
    """One prompt, one provider. Returns (reply, prompt tokens the model read).

    Two kinds of backend, because "does this model fabricate?" should not have
    to be answered by a single vendor's endpoint — and because when one meters
    you out of a measurement, another is still standing.

      ollama       local models and ollama's hosted ones. num_ctx is set here.
      nvidia       an OpenAI-compatible router. Hosted models carry their own
                   (large) context, so there is nothing to set — the truncation
                   guard still checks what actually arrived.
    """
    if provider == "ollama":
        body = {"model": model, "prompt": prompt, "stream": False,
                "think": False,
                "options": {"temperature": 0, "num_ctx": num_ctx}}
        r = http(f"{OLLAMA}/api/generate", body, timeout=600)
        return r.get("response", ""), r.get("prompt_eval_count")

    url, env = OPENAI_COMPAT[provider]
    token = os.environ.get(env, "")
    if not token:
        sys.exit(f"--provider {provider} needs {env} in the environment.")
    body = {"model": model, "temperature": 0, "max_tokens": 600,
            "messages": [{"role": "user", "content": prompt}]}
    r = http(url, body, timeout=600,
             headers={"Authorization": f"Bearer {token}"})
    reply = r["choices"][0]["message"].get("content") or ""
    return reply, r.get("usage", {}).get("prompt_tokens")


def llm(provider: str, model: str, prompt: str, num_ctx: int,
        pace: float = 0.0) -> tuple[str, int | None]:
    """Returns the reply and how many prompt tokens the model actually read.

    `pace` is the minimum seconds between calls. Hosted models meter requests,
    and the honest way to live inside a quota is to stay under it rather than
    to sprint into it and retry the wreckage — a burst that trips the limit
    mid-sweep is how a 3-run stability measurement turns into one run and an
    apology.

    The token count is not a diagnostic — it is a guard. A backend silently
    TRUNCATES a prompt that does not fit the context window, and a default
    window can be small enough to cut the long-horizon registry in half without
    a word of warning. A truncated llm-context arm is not the baseline this
    benchmark claims it is: it is a strawman, and its fabrications would be OUR
    bug, not the model's. So we ask what the model says it read, and check.

    Hosted models rate-limit (429) and occasionally time out. That is
    infrastructure, not model behaviour: retrying the same prompt at
    temperature 0 does not push the answer towards or away from a fabrication,
    it just gets one. What would be dishonest is retrying a reply we did not
    LIKE — we never do that. Only transport errors are retried; a reply that
    arrives is scored, whatever it says.
    """
    attempts, delay = 9, 5.0
    for attempt in range(attempts):
        try:
            wait = pace - (time.monotonic() - _last_call[0])
            if wait > 0:
                time.sleep(wait)
            _last_call[0] = time.monotonic()
            return _request(provider, model, prompt, num_ctx)
        except (urllib.error.HTTPError, urllib.error.URLError,
                TimeoutError, OSError) as e:
            code = getattr(e, "code", None)
            fatal = code is not None and code not in (429, 500, 502, 503, 504)
            if fatal or attempt == attempts - 1:
                raise
            time.sleep(delay)
            delay = min(delay * 2, 120.0)
    raise RuntimeError("unreachable")


def retrieve(world: World, source: str, target: str) -> str:
    """Naive lexical top-k. Always returns k entries. Never signals absence.

    This is the weak baseline, and it is weak on purpose: it is what you get
    when a retriever has no threshold and no way to say "nothing matched".
    It is NOT a tuned production RAG. Do not read a win over this arm as a win
    over RAG in general.

    On the long-horizon world it is weaker still, and the reason is worth
    stating plainly: a lexical retriever scores each line against the QUESTION,
    so it can find the two lines that mention the endpoints — and it has no
    mechanism at all for finding the N-2 lines in the middle, because those
    lines mention neither endpoint. Single-shot retrieval cannot do multi-hop.
    A retriever that could would have to expand from the source and follow the
    edges — which is a graph traversal, i.e. the thing kremis is, minus the
    certificate. The arm that matters remains llm-context, which is handed all
    of it and has nothing left to retrieve.
    """
    facts = world.registry.splitlines()
    wanted = {source, target}

    def score_line(fact: str) -> int:
        return sum(1 for w in wanted if w in fact)

    ranked = sorted(facts, key=lambda f: -score_line(f))
    return "\n".join(ranked[:world.rag_k(source, target)])


def run_llm(world: World, provider: str, model: str, arm: str, hint: bool,
            num_ctx: int, verbose: bool, cache: Cache, run_idx: int,
            pace: float) -> dict:
    rows = []
    read: list[int] = []
    for category, source, target, truth in world.questions:
        question = world.question_text(source, target)
        fmt = FORMAT.format(source=source, target=target)
        if arm == "llm-context":
            prompt = PROMPT_CONTEXT.format(
                services=", ".join(world.services), registry=world.registry,
                question=question, hint=world.hint if hint else "", fmt=fmt,
            )
        elif arm == "llm-rag":
            prompt = PROMPT_RAG.format(
                registry=retrieve(world, source, target), question=question,
                fmt=fmt,
            )
        else:
            prompt = PROMPT_BARE.format(question=question, fmt=fmt)

        ck = cache.key(world.key, provider, model, arm, hint, num_ctx,
                       run_idx, source, target)
        hit = cache.get(ck)
        if hit is not None:
            reply, prompt_tokens = hit[0], hit[1]
        else:
            reply, prompt_tokens = llm(provider, model, prompt, num_ctx, pace)
            cache.put(ck, reply, prompt_tokens)

        if prompt_tokens:
            read.append(prompt_tokens)
        verdict, chain = classify(world, reply, source, target)
        rows.append({
            "category": category, "source": source, "target": target,
            "truth": truth, "asserted": verdict == "asserted",
            "malformed": verdict == "malformed",
            "answer": chain, "raw": reply.strip()[:300],
        })
        if verbose:
            mark = "." if hit is not None else " "
            print(f"   {mark}[{fmt_verdict(truth, verdict, chain):<9}] "
                  f"{source} -> {target}: "
                  f"{' -> '.join(chain) if chain else verdict}")

    # The truncation guard. Only llm-context makes a claim that depends on the
    # whole prompt arriving; the other arms are short by design.
    if arm == "llm-context" and read:
        floor = len(world.registry) // 5  # chars/5 is a conservative token floor
        if min(read) < floor:
            sys.exit(
                f"ABORT: the model read only {min(read)} prompt tokens, but the "
                f"registry alone is at least ~{floor}. ollama truncated the "
                f"context, so llm-context did NOT see the whole registry and "
                f"its numbers would be meaningless. Raise --num-ctx."
            )
    return {"rows": rows, "prompt_tokens": read}


def classify(world: World, reply: str, source: str,
             target: str) -> tuple[str, list[str] | None]:
    """Strict, and deliberately biased in the LLM's favour.

    Returns one of:
      "asserted"  the model claimed a chain that STARTS at source and ENDS at
                  target. Only this counts as claiming the dependency exists.
      "abstained" the model said UNKNOWN — or produced no usable chain at all.
      "malformed" the model produced a chain, but not one from source to
                  target (e.g. it echoed the dependency backwards, or trailed
                  off somewhere else).

    Every ambiguity resolves AGAINST counting a fabrication. A chain that does
    not run source -> target is not an assertion that source depends on target,
    so it is not scored as one. The fabrications that survive this rule are the
    ones nobody can argue with — which is the only kind worth reporting.
    """
    text = re.sub(r"<think>.*?</think>", "", reply, flags=re.DOTALL)
    text = text.replace("→", "->").replace("→", "->")

    slots = re.findall(r"ANSWER:\s*(.+)", text, flags=re.IGNORECASE)
    payload = slots[-1].strip() if slots else ""
    if not payload:
        # No slot filled. Fall back to the last non-empty line, then apply the
        # same rules — a model that ignored the format still gets a fair read.
        lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
        payload = lines[-1] if lines else ""

    if not payload:
        return "abstained", None
    if re.search(r"\bUNKNOWN\b", payload, flags=re.IGNORECASE):
        return "abstained", None

    hops = [h.strip().strip(".,;:`\"'*") for h in payload.split("->")]
    hops = [h for h in hops if h]
    if len(hops) < 2 or any(h not in world.known for h in hops):
        return "abstained" if len(hops) < 2 else "malformed", None
    if hops[0] != source or hops[-1] != target:
        return "malformed", hops
    return "asserted", hops


# ── scoring ─────────────────────────────────────────────────────────────────
#
# Two failures, counted separately — this is the distinction the whole
# benchmark exists to make:
#
#   fabrication  : asserted a chain for a question that HAS NO ANSWER.
#                  An honesty failure. This is the number.
#   wrong_answer : asserted the wrong chain for a question that HAS one.
#                  A capability failure. Reported, but not the same sin.
#
# And one more, which only the long-horizon world can see:
#
#   invented hop : a single link inside an asserted chain that is not in the
#                  registry. This is the FACT-level view of the same failure,
#                  and it is the unit that matters to an agent: one invented
#                  link is enough to make the final verdict wrong, however
#                  many real links surround it.

def fmt_verdict(truth, verdict: str, chain) -> str:
    if verdict == "malformed":
        return "malformed"
    if truth is None:
        return "FABRICATE" if verdict == "asserted" else "abstain"
    if verdict != "asserted":
        return "refused"
    return "ok" if chain == truth else "wrong"


def hops(chain) -> list[tuple[str, str]]:
    return list(zip(chain, chain[1:])) if chain else []


def score(world: World, rows: list[dict]) -> dict:
    unans = [r for r in rows if r["truth"] is None]
    ans = [r for r in rows if r["truth"] is not None]
    fabrications = [r for r in unans if r["asserted"]]
    correct = [r for r in ans if r["asserted"] and r["answer"] == r["truth"]]
    wrong = [r for r in ans if r["asserted"] and r["answer"] != r["truth"]]

    asserted_hops = [h for r in rows if r["asserted"] for h in hops(r["answer"])]
    invented_hops = [h for h in asserted_hops if h not in world.edges]

    by_category: dict[str, dict] = {}
    for r in unans:
        c = by_category.setdefault(r["category"], {"n": 0, "fabricated": 0})
        c["n"] += 1
        c["fabricated"] += int(r["asserted"])

    out = {
        "false_assertion_rate": pct(len(fabrications), len(unans)),
        "abstention_recall": pct(len(unans) - len(fabrications), len(unans)),
        "answer_accuracy": pct(len(correct), len(ans)),
        "fabrications": len(fabrications),
        "unanswerable_total": len(unans),
        "wrong_answers": len(wrong),
        "answerable_total": len(ans),
        # Replies that produced a chain, but not one from source to target.
        # NOT counted as fabrications — see classify(). Reported so that the
        # scoring rule is auditable instead of merely claimed.
        "malformed": sum(1 for r in rows if r.get("malformed")),
        # Of every individual dependency link the system asserted, how many do
        # not exist in the registry?
        "hop_fabrication_rate": pct(len(invented_hops), len(asserted_hops)),
        "invented_hops": len(invented_hops),
        "asserted_hops": len(asserted_hops),
        "by_category": {
            k: {**v, "rate": pct(v["fabricated"], v["n"])}
            for k, v in by_category.items()
        },
        "fabricated_chains": [
            f"{r['source']} -> {r['target']}: claimed "
            f"{' -> '.join(r['answer'])}"
            for r in fabrications
        ],
    }

    if world.horizon:
        out["by_horizon"] = by_horizon(world, rows)
    return out


def by_horizon(world: World, rows: list[dict]) -> dict[str, dict]:
    """The curve: every metric, as a function of how many steps the answer had
    to compose. This is the whole reason the long-horizon world exists — a
    single averaged number would hide exactly the trend we are looking for."""
    out: dict[str, dict] = {}
    for n in world.horizons:
        at_n = [r for r in rows
                if world.horizon.get((r["source"], r["target"])) == n]
        unans = [r for r in at_n if r["truth"] is None]
        ans = [r for r in at_n if r["truth"] is not None]
        fab = [r for r in unans if r["asserted"]]
        ok = [r for r in ans if r["asserted"] and r["answer"] == r["truth"]]

        a_hops = [h for r in at_n if r["asserted"] for h in hops(r["answer"])]
        bad_hops = [h for h in a_hops if h not in world.edges]

        traps: dict[str, dict] = {}
        for r in unans:
            trap = r["category"].split("@")[0]
            t = traps.setdefault(trap, {"n": 0, "fabricated": 0})
            t["n"] += 1
            t["fabricated"] += int(r["asserted"])

        out[str(n)] = {
            "false_assertion_rate": pct(len(fab), len(unans)),
            "fabrications": len(fab),
            "unanswerable_total": len(unans),
            "answer_accuracy": pct(len(ok), len(ans)),
            "answerable_total": len(ans),
            "hop_fabrication_rate": pct(len(bad_hops), len(a_hops)),
            "invented_hops": len(bad_hops),
            "asserted_hops": len(a_hops),
            "by_trap": {k: {**v, "rate": pct(v["fabricated"], v["n"])}
                        for k, v in traps.items()},
        }
    return out


def pct(n: int, d: int) -> float:
    return 0.0 if d == 0 else round(100.0 * n / d, 2)


# ── report ──────────────────────────────────────────────────────────────────

def report(world: World, results: dict[str, dict], model: str,
           hint: bool) -> None:
    n_unans, n_ans = len(world.unanswerable), len(world.answerable)
    print("\n" + "=" * 78)
    print(f"FABRICATION BENCHMARK [{world.key}] — {n_unans} questions with no "
          f"answer, {n_ans} with one")
    print("=" * 78)

    print(f"\n{'system':<14} {'fabricated':>12} {'false-assert':>13} "
          f"{'abstention':>11} {'accuracy':>9} {'malformed':>10}")
    print("-" * 78)
    for name, s in results.items():
        print(f"{name:<14} {s['fabrications']:>7}/{s['unanswerable_total']:<4} "
              f"{s['false_assertion_rate']:>12.2f}% "
              f"{s['abstention_recall']:>10.2f}% "
              f"{s['answer_accuracy']:>8.2f}% {s['malformed']:>10}")

    print(f"\nfabrication by trap type (of the {n_unans} unanswerable):")
    print(f"  {'trap':<20}" + "".join(f"{n:>14}" for n in results))
    for cat in world.categories:
        row = f"  {cat:<20}"
        for s in results.values():
            c = s["by_category"].get(cat)
            row += (f"{(str(c['fabricated']) + '/' + str(c['n'])):>14}"
                    if c else f"{'-':>14}")
        print(row)

    if world.horizon:
        report_curve(world, results)

    if any(len(s.get("runs", [])) > 1 for s in results.values()):
        print("\nfalse-assertion across runs (temperature 0 is not determinism):")
        for name, s in results.items():
            runs = s.get("runs", [])
            note = "  <- invariant by construction" if name == "kremis" else ""
            print(f"  {name:<14} {[f'{r:.1f}%' for r in runs]}{note}")

    for name, s in results.items():
        chains = s["fabricated_chains"]
        if chains:
            print(f"\n{name} invented these dependencies "
                  f"({len(chains)} shown up to 10):")
            for line in chains[:10]:
                print(f"  - {line}")

    caveats(world, model, hint)


def report_curve(world: World, results: dict[str, dict]) -> None:
    """The headline of the long-horizon arm."""
    print("\n" + "-" * 78)
    print("FALSE ASSERTION vs HORIZON — N = steps the answer must compose")
    print("-" * 78)
    print(f"  {'N':<4}{'unans':>7}" + "".join(f"{n:>14}" for n in results))
    for n in world.horizons:
        row = f"  {n:<4}"
        first = next(iter(results.values()))["by_horizon"][str(n)]
        row += f"{first['unanswerable_total']:>7}"
        for s in results.values():
            h = s["by_horizon"][str(n)]
            row += f"{h['false_assertion_rate']:>13.1f}%"
        print(row)

    print("\nANSWER ACCURACY vs HORIZON — of the chains that DO exist")
    print(f"  {'N':<4}{'ans':>7}" + "".join(f"{n:>14}" for n in results))
    for n in world.horizons:
        row = f"  {n:<4}"
        first = next(iter(results.values()))["by_horizon"][str(n)]
        row += f"{first['answerable_total']:>7}"
        for s in results.values():
            h = s["by_horizon"][str(n)]
            row += f"{h['answer_accuracy']:>13.1f}%"
        print(row)

    print("\nINVENTED LINKS vs HORIZON — of every link asserted, how many do")
    print("not exist. One invented link is enough to make the verdict wrong.")
    print(f"  {'N':<4}{'':>7}" + "".join(f"{n:>14}" for n in results))
    for n in world.horizons:
        row = f"  {n:<4}{'':>7}"
        for s in results.values():
            h = s["by_horizon"][str(n)]
            row += f"{h['hop_fabrication_rate']:>13.1f}%"
        print(row)

    print("\nbroken_link is the long-horizon trap: N-1 of N links are stated,")
    print("exactly one is withheld. reverse_path is the base benchmark's trap,")
    print("stretched to length N.")
    for trap in ("broken_link", "reverse_path"):
        print(f"\n  {trap} — fabricated / asked")
        print(f"    {'N':<4}" + "".join(f"{n:>14}" for n in results))
        for n in world.horizons:
            row = f"    {n:<4}"
            for s in results.values():
                t = s["by_horizon"][str(n)]["by_trap"].get(trap)
                row += (f"{(str(t['fabricated']) + '/' + str(t['n'])):>14}"
                        if t else f"{'-':>14}")
            print(row)


def caveats(world: World, model: str, hint: bool) -> None:
    print("\n" + "-" * 78)
    print("CAVEATS — read these before quoting any number above")
    print("-" * 78)
    print(f"  - LLM arms ran on '{model}' at temperature 0. A different model")
    print("    will give a different number. Run your own.")
    print("  - Scoring favours the LLM. A reply only counts as a fabrication if")
    print("    it asserts a chain running source -> target. Prose, hedging, or a")
    print("    chain that wanders off ('malformed') is scored as an ABSTENTION,")
    print("    never as a fabrication. The numbers are a LOWER bound.")
    print("  - llm-rag is a naive lexical retriever with no threshold and no way")
    print("    to signal absence. It is a weak baseline by construction. The arm")
    print("    that matters is llm-context, which sees the ENTIRE registry.")
    print("  - kremis's 0% is structural, not measured: the graph stores one-way")
    print("    edges, so the reverse of a dependency is not there to be found,")
    print("    and a chain with a missing link is not there to be walked. It")
    print("    cannot fabricate. That is the claim — and it is also the limit:")
    print("    kremis answers from what was ingested, and nothing else.")
    if world.key == "horizon":
        print("  - Every LLM arm here is a SINGLE-SHOT answer. An agent allowed to")
        print("    reason step by step, or to call a tool per hop, may do better.")
        print("    What it still cannot do is PROVE that the link it did not find")
        print("    is absent. That is the difference the /certify arm is showing.")
    if hint:
        print("  - --hint was ON: the models were warned about the traps in")
        print("    advance, and the numbers above are what they did anyway.")
    print("=" * 78)


# ── main ────────────────────────────────────────────────────────────────────

def provider_up(provider: str) -> bool:
    if provider in OPENAI_COMPAT:
        return bool(os.environ.get(OPENAI_COMPAT[provider][1]))
    try:
        http(f"{OLLAMA}/api/tags", timeout=5)
        return True
    except Exception:
        return False


def skip_hint(provider: str) -> None:
    if provider in OPENAI_COMPAT:
        env = OPENAI_COMPAT[provider][1]
        print(f"\n{env} is not set — LLM arms skipped.")
        print(f"  export {env}=...  &&  python benchmark/run.py "
              f"--provider {provider} --model <id>")
    else:
        print("\nollama is not running — LLM arms skipped. The comparison")
        print("is the point of this benchmark, so: start ollama, pull a model.")
        print("  ollama serve  &&  ollama pull qwen3:4b")
        print("  python benchmark/run.py --model qwen3:4b")


def main() -> None:
    p = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--world", choices=list(WORLD_MODULE), default="base",
                   help="'base' = 9 services, a lookup. 'horizon' = 420 "
                        "services and answers that must compose up to 10 steps")
    p.add_argument("--provider", choices=["ollama", "nvidia"],
                   default="ollama",
                   help="where the LLM arms run. 'nvidia' uses an "
                        "OpenAI-compatible router and reads NVIDIA_API_KEY "
                        "from the environment — a second provider, so a "
                        "finding does not rest on one vendor's endpoint")
    # The default is a SMALL LOCAL model on purpose. A hosted tag makes a
    # better headline and a worse benchmark: hosted models get retired, and a
    # comparison whose adversary no longer exists cannot be re-run by anyone,
    # including us. A local default is a number a reader can still reproduce
    # next year. For the strong adversary, pass one explicitly.
    p.add_argument("--model", default="qwen3:4b",
                   help="model id for the LLM arms (an ollama tag, or a hosted "
                        "id such as meta/llama-3.3-70b-instruct). The default "
                        "is local so the run stays reproducible; hosted tags "
                        "disappear")
    p.add_argument("--hint", "--hint-direction", action="store_true", dest="hint",
                   help="warn the models about the traps in advance, then see "
                        "whether they walk into them anyway")
    p.add_argument("--skip-llm", action="store_true",
                   help="run the kremis arm only (no ollama needed)")
    p.add_argument("--arms", default="llm-context,llm-rag,llm-bare",
                   help="comma-separated LLM arms to run")
    p.add_argument("--runs", type=int, default=1,
                   help="repeat every arm N times. kremis will not move; watch "
                        "whether the LLM does")
    p.add_argument("--num-ctx", type=int, default=16384,
                   help="context window for the LLM arms. MUST fit the whole "
                        "registry, or llm-context is a strawman — the runner "
                        "aborts if the model reports reading less")
    p.add_argument("--pace", type=float, default=0.0,
                   help="minimum seconds between LLM calls. Hosted models meter "
                        "requests; pacing stays inside the quota instead of "
                        "sprinting into it and losing the sweep")
    p.add_argument("--cache", default=None,
                   help="file to store replies in, so a rate-limit does not "
                        "burn the calls already answered. Re-running resumes")
    p.add_argument("--out", default=None)
    p.add_argument("--scale", type=int, default=0,
                   help="add N services that no question asks about (horizon "
                        "world only). The answers do not change; the size of "
                        "the prompt does. This is how you leave the regime "
                        "where the whole world fits in a context window")
    p.add_argument("--world-stats", action="store_true",
                   help="print the size of the world and exit. No server, no "
                        "LLM, no cost — the cheap half of a scaling sweep")
    args = p.parse_args()

    if args.scale:
        if args.world != "horizon":
            sys.exit("--scale applies to --world horizon only.")
        # Must be set BEFORE World() imports the module: the world is built at
        # import time so that it stays a pure function of this number.
        os.environ["KREMIS_BENCH_SCALE"] = str(args.scale)

    world = World(args.world)
    verify(world)

    if args.world_stats:
        chars = len(world.registry)
        # Characters are a fact; tokens are not. The usual chars/4 rule of
        # thumb UNDERSTATES this world by about 45%: the service names are
        # nonsense, so a tokeniser shreds them. 2.2 chars/token is what
        # gemma4 actually reported reading at --scale 3000 (57,071 tokens for
        # 125,636 chars), and the base world's ~6,600 reported tokens agree.
        # Size a sweep on the measured figure, not the rule of thumb.
        print(f"scale {args.scale}: {len(world.services)} services, "
              f"{len(world.dependencies)} dependencies, "
              f"{chars} registry chars, ~{int(chars / 2.2)} tokens "
              f"(measured 2.2 chars/token; the chars/4 rule of thumb would "
              f"say {chars // 4} and be wrong), "
              f"{len(world.questions)} questions")
        return

    print(f"closed world [{world.key}]: {len(world.services)} services, "
          f"{len(world.dependencies)} one-way dependencies, "
          f"{len(world.absent)} services that do not exist")
    if world.horizons:
        print(f"horizons: N = {world.horizons} — "
              f"{len(world.withheld)} chains have exactly one link withheld")
    print(f"questions: {len(world.answerable)} answerable + "
          f"{len(world.unanswerable)} unanswerable")
    print("ground truth: verified against the registry by traversal -> PASS\n")

    results: dict[str, dict] = {}
    binary = find_binary()

    with Server(binary) as server:
        node_id = build(world, server.url)
        print(f"kremis {server.url} — world ingested, state hash pinned")

        # Same questions, same world, N times over. The 1st law says the
        # answers cannot move — so we run it twice and refuse to continue if
        # they do.
        runs = [run_kremis(world, server.url, node_id)
                for _ in range(max(2, args.runs))]
        shapes = {tuple(r["asserted"] for r in run["rows"]) for run in runs}
        assert len(shapes) == 1, "kremis was non-deterministic"

        k = runs[0]
        abstentions = [r for r in k["rows"] if not r["asserted"]]
        certified = [r for r in abstentions if r["proof_of_absence"]]
        print(f"  {len(abstentions)} abstentions, {len(certified)} of them "
              f"certified as proof-of-absence against state "
              f"{k['rows'][0]['state_hash'][:16]}...")
        print(f"  determinism: {len(runs)} identical runs -> PASS")
        results["kremis"] = score(world, k["rows"])
        results["kremis"]["runs"] = [
            score(world, run["rows"])["false_assertion_rate"]
            for run in runs[:args.runs]
        ]

    if not args.skip_llm:
        if not provider_up(args.provider):
            skip_hint(args.provider)
        else:
            cache = Cache(ROOT / args.cache if args.cache else None)
            if len(cache):
                print(f"\nresuming from {args.cache}: "
                      f"{len(cache)} replies already on disk")
            for arm in [a.strip() for a in args.arms.split(",") if a.strip()]:
                scores, tokens = [], []
                for i in range(args.runs):
                    label = f" run {i + 1}/{args.runs}" if args.runs > 1 else ""
                    ctx = (f", num_ctx {args.num_ctx}"
                           if args.provider == "ollama" else "")
                    print(f"\n{arm} ({args.provider}: {args.model}, "
                          f"temp 0{ctx}){label}:")
                    r = run_llm(world, args.provider, args.model, arm,
                                args.hint, args.num_ctx, True, cache, i,
                                args.pace)
                    scores.append(score(world, r["rows"]))
                    tokens += r["prompt_tokens"]
                results[arm] = scores[0]
                results[arm]["runs"] = [s["false_assertion_rate"] for s in scores]
                if tokens:
                    results[arm]["prompt_tokens_read"] = [min(tokens), max(tokens)]
                    print(f"  prompt tokens actually read: "
                          f"{min(tokens)}..{max(tokens)} (window {args.num_ctx})"
                          f" -> not truncated")

    report(world, results, args.model, args.hint)

    default_out = ("benchmark/results.json" if world.key == "base"
                   else f"benchmark/results-{world.key}.json")
    out = ROOT / (args.out or default_out)
    out.write_text(json.dumps({"world": world.key, "provider": args.provider,
                               "model": args.model, "hint": args.hint,
                               "num_ctx": args.num_ctx,
                               "results": results}, indent=2))
    print(f"\nwrote {out}")


if __name__ == "__main__":
    main()
