//! Latent Retrieval Pipeline (`katSVG Retrieval`).
//!
//! Model-less, deterministic retrieval of layout candidates for a prompt,
//! behind a single trait so the backend can be upgraded without touching the
//! router (canonical plan §4.3):
//!
//! - [`TagRetriever`] — exact lexical overlap against corpus tags (baseline).
//! - [`EmbeddingRetriever`] — feature-hashed (word + char-trigram) embedding with
//!   cosine relevance: a latent-space analog of katGPT's O(log N) retrieval,
//!   robust to token order and near-miss phrasing.
//!
//! Both are pure functions of (query, corpus): same inputs → identical ranks.

use crate::router::LayoutDef;

/// A layout candidate ranked by a retrieval pipeline.
#[derive(Debug, Clone, Copy)]
pub struct RetrievedLayout {
    /// Index into the candidate corpus.
    pub index: usize,
    /// Relevance in [0,1]; higher = better fit.
    pub relevance: f32,
}

/// Backend-agnostic retrieval contract.
pub trait RetrievalPipeline: Send + Sync {
    fn name(&self) -> &str;

    /// Ranks corpus candidates for `query`, best first. Deterministic.
    fn retrieve(&self, query: &str, candidates: &[LayoutDef]) -> Vec<RetrievedLayout>;
}

// ── Tokenization helpers ─────────────────────────────────────────────────────

/// English stop words excluded from retrieval features (avoid false matches
/// from function words common to every description).
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "of", "to", "in", "on", "for", "with", "by",
    "at", "from", "as", "into", "via", "per", "each", "is", "are", "be", "been",
    "it", "its", "this", "that", "these", "those", "their", "they",
];

/// Lowercased alphanumeric word tokens, excluding stop words.
fn word_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            cur.push(c.to_ascii_lowercase());
        } else if !cur.is_empty() {
            if !STOP_WORDS.contains(&cur.as_str()) {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if !cur.is_empty() && !STOP_WORDS.contains(&cur.as_str()) {
        out.push(cur);
    }
    out
}

fn query_features(text: &str) -> Vec<String> {
    word_tokens(text)
}

fn candidate_features(l: &LayoutDef) -> Vec<String> {
    let mut feats = Vec::new();
    // tags + layout type + description (all part of the closed corpus)
    for t in &l.tags {
        feats.extend(word_tokens(t));
    }
    feats.push(format!("{:?}", l.layout_type).to_lowercase());
    if let Some(d) = &l.description {
        feats.extend(word_tokens(d));
    }
    feats
}

// ── Tag baseline ─────────────────────────────────────────────────────────────

/// Exact tag-overlap baseline: relevance = |query_tokens ∩ tags| / |tags|.
pub struct TagRetriever;

impl RetrievalPipeline for TagRetriever {
    fn name(&self) -> &str {
        "tag"
    }

    fn retrieve(&self, query: &str, candidates: &[LayoutDef]) -> Vec<RetrievedLayout> {
        let qtokens: std::collections::HashSet<String> = word_tokens(query).into_iter().collect();
        let mut ranked: Vec<RetrievedLayout> = candidates
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let tags: std::collections::HashSet<&String> = l.tags.iter().collect();
                let hits = tags
                    .iter()
                    .filter(|t| qtokens.contains(t.as_str()))
                    .count();
                let denom = tags.len().max(1) as f32;
                RetrievedLayout { index: i, relevance: hits as f32 / denom }
            })
            .collect();
        ranked.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }
}

// ── Vocabulary-based sparse embedding ────────────────────────────────────────

/// Sparse TF embedding retriever over a closed corpus vocabulary.
///
/// Features (words from tags + layout type + description) form a vocabulary;
/// query and candidate become count vectors in that space; relevance is cosine
/// similarity. Unlike feature-hashing, non-matching words contribute exactly
/// zero, so there is no similarity noise floor. Deterministic: vocabulary order
/// follows corpus order; no RNG.
pub struct EmbeddingRetriever;

impl EmbeddingRetriever {
    pub fn new() -> Self {
        Self
    }

    fn embed(&self, feats: &[String], vocab: &std::collections::HashMap<String, usize>) -> Vec<f32> {
        let mut v = vec![0.0f32; vocab.len()];
        for f in feats {
            if let Some(&i) = vocab.get(f) {
                v[i] += 1.0;
            }
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

impl Default for EmbeddingRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl RetrievalPipeline for EmbeddingRetriever {
    fn name(&self) -> &str {
        "embedding"
    }

    fn retrieve(&self, query: &str, candidates: &[LayoutDef]) -> Vec<RetrievedLayout> {
        // Closed vocabulary from the candidate corpus (deterministic order).
        let mut vocab: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for l in candidates {
            for f in candidate_features(l) {
                if !vocab.contains_key(&f) {
                    vocab.insert(f, vocab.len());
                }
            }
        }

        let qvec = self.embed(&query_features(query), &vocab);
        let mut ranked: Vec<RetrievedLayout> = candidates
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let cvec = self.embed(&candidate_features(l), &vocab);
                RetrievedLayout { index: i, relevance: self.cosine(&qvec, &cvec) }
            })
            .collect();
        // Stable sort (vocabulary order is fixed by corpus order).
        ranked.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }
}

/// Default retrieval pipeline: vocabulary-based embedding (S6).
pub fn default_retriever() -> Box<dyn RetrievalPipeline> {
    Box::new(EmbeddingRetriever::new())
}
