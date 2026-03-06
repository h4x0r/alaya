"""MemoryAgentBench runner.

Loads the MemoryAgentBench dataset (Hu et al., ICLR 2026) from
HuggingFace, feeds text chunks through adapters as conversation
messages, and scores answers using LLM-as-Judge.

Four competency categories:
  AR  - Accurate Retrieval
  TTL - Test-Time Learning
  LRU - Long-Range Understanding
  CR  - Conflict Resolution (Selective Forgetting)
"""

import re
import time
from dataclasses import dataclass, field

from tqdm import tqdm

from adapters.base import MemoryAdapter, Message


# Representative sub-dataset per competency (balanced size/coverage).
# max_samples caps how many HF samples to load (each has many questions).
COMPETENCY_CONFIGS = {
    "AR": {
        "split": "Accurate_Retrieval",
        "source": "eventqa_65536",
        "max_samples": 5,
        "label": "Accurate Retrieval",
    },
    "TTL": {
        "split": "Test_Time_Learning",
        "source": "icl_nlu_8296shot_balance",
        "max_samples": 1,
        "label": "Test-Time Learning",
    },
    "LRU": {
        "split": "Long_Range_Understanding",
        "source": "detective_qa",
        "max_samples": 5,
        "label": "Long-Range Understanding",
    },
    "CR": {
        "split": "Conflict_Resolution",
        "source": "factconsolidation_sh_32k",
        "max_samples": 1,
        "label": "Conflict Resolution",
    },
}


@dataclass
class MABResult:
    system: str
    total_questions: int = 0
    correct: int = 0
    scores_by_competency: dict[str, list[float]] = field(default_factory=dict)
    per_question: list[dict] = field(default_factory=list)
    elapsed_seconds: float = 0.0

    @property
    def accuracy(self) -> float:
        return self.correct / self.total_questions if self.total_questions else 0.0


def _load_hf_split(split_name: str, source: str, max_samples: int) -> list[dict]:
    """Load and filter a MemoryAgentBench split from HuggingFace."""
    from datasets import load_dataset

    ds = load_dataset("ai-hyz/MemoryAgentBench", split=split_name, revision="main")

    filtered = ds.filter(
        lambda s: s.get("metadata", {}).get("source", "") == source
    )

    if len(filtered) == 0:
        sources = set(s.get("metadata", {}).get("source", "") for s in ds)
        raise ValueError(
            f"No samples for source '{source}' in '{split_name}'. "
            f"Available: {sorted(sources)}"
        )

    if max_samples and len(filtered) > max_samples:
        filtered = filtered.select(range(max_samples))

    return list(filtered)


def _chunk_text(text: str, chunk_size: int = 4096) -> list[str]:
    """Split text into sentence-boundary-respecting chunks.

    Uses a simple sentence splitter to avoid heavy nltk/tiktoken deps.
    chunk_size is approximate token count (4 chars/token heuristic).
    """
    char_limit = chunk_size * 4
    sentences = re.split(r'(?<=[.!?])\s+', text)

    chunks: list[str] = []
    current: list[str] = []
    current_len = 0

    for sent in sentences:
        if current_len + len(sent) > char_limit and current:
            chunks.append(" ".join(current))
            current = [sent]
            current_len = len(sent)
        else:
            current.append(sent)
            current_len += len(sent)

    if current:
        chunks.append(" ".join(current))

    return chunks


def _chunks_to_messages(chunks: list[str]) -> list[Message]:
    """Convert text chunks into Message objects for adapter ingestion."""
    return [
        Message(
            role="user",
            content=chunk,
            session_id=f"chunk_{i}",
            timestamp=f"2024-01-01T{i:02d}:00:00",
        )
        for i, chunk in enumerate(chunks)
    ]


@dataclass
class _ContextGroup:
    """A context with its associated questions, for ingest-once-query-many."""
    competency: str
    messages: list[Message]
    questions: list[str]
    golds: list[str]


def run_mab(
    adapter: MemoryAdapter,
    judge_fn: callable,
    llm_call: callable,
    competencies: list[str] | None = None,
    limit: int | None = None,
    dry_run: bool = False,
) -> MABResult:
    """Run MemoryAgentBench on a single adapter.

    Optimization: ingests each context once, then queries all questions
    for that context before resetting.  This avoids redundant embedding
    work in RAG adapters.

    Args:
        adapter: The memory system adapter.
        judge_fn: Callable(question, gold, prediction) -> float.
        llm_call: Callable(prompt) -> str for answer generation.
        competencies: Which competency categories to run (default: all four).
        limit: Max total questions to evaluate (None = all).
        dry_run: If True, count questions and estimate cost without running.
    """
    if competencies is None:
        competencies = list(COMPETENCY_CONFIGS.keys())

    result = MABResult(system=adapter.name)

    # Build context groups: one group per (competency, sample).
    groups: list[_ContextGroup] = []
    total_q = 0

    for comp_key in competencies:
        cfg = COMPETENCY_CONFIGS[comp_key]
        try:
            samples = _load_hf_split(cfg["split"], cfg["source"], cfg["max_samples"])
        except Exception as e:
            print(f"[WARN] Could not load {comp_key} ({cfg['source']}): {e}")
            continue

        for sample in samples:
            context = sample["context"]
            chunks = _chunk_text(context)
            messages = _chunks_to_messages(chunks)

            questions = sample.get("questions", [])
            answers = sample.get("answers", [])
            if isinstance(questions, str):
                questions = [questions]
            if isinstance(answers, str):
                answers = [answers]

            # Apply per-question limit
            qa_pairs = list(zip(questions, answers))
            if limit and total_q + len(qa_pairs) > limit:
                qa_pairs = qa_pairs[:limit - total_q]

            if qa_pairs:
                qs, gs = zip(*qa_pairs)
                groups.append(_ContextGroup(
                    competency=comp_key,
                    messages=messages,
                    questions=list(qs),
                    golds=[str(g) if not isinstance(g, str) else g for g in gs],
                ))
                total_q += len(qa_pairs)

            if limit and total_q >= limit:
                break
        if limit and total_q >= limit:
            break

    if dry_run:
        result.total_questions = total_q
        ctx_count = len(groups)
        comp_counts = {}
        for g in groups:
            comp_counts[g.competency] = comp_counts.get(g.competency, 0) + len(g.questions)
        print(f"[DRY RUN] {adapter.name}: {total_q} questions, "
              f"{ctx_count} contexts, {len(competencies)} competencies")
        for k, v in comp_counts.items():
            label = COMPETENCY_CONFIGS[k]["label"]
            print(f"  {k} ({label}): {v} questions")
        return result

    start = time.time()

    for group in tqdm(groups, desc=f"MAB [{adapter.name}]", unit="ctx"):
        # Ingest context once
        adapter.reset()
        adapter.ingest(group.messages)

        # Query all questions for this context
        for question, gold in zip(group.questions, group.golds):
            prediction = adapter.query(question, llm_call)
            score = judge_fn(question, gold, prediction)

            result.total_questions += 1
            result.correct += int(score >= 0.5)
            result.scores_by_competency.setdefault(group.competency, []).append(score)
            result.per_question.append({
                "question": question,
                "gold": gold,
                "prediction": prediction,
                "score": score,
                "competency": group.competency,
                "competency_label": COMPETENCY_CONFIGS[group.competency]["label"],
            })

    result.elapsed_seconds = time.time() - start
    return result
