use crate::error::Result;
use crate::types::*;

/// The agent provides this trait to support intelligent consolidation.
/// Alaya never calls an LLM directly — the agent owns the LLM connection.
pub trait ConsolidationProvider {
    /// Extract semantic knowledge from a batch of episodes.
    fn extract_knowledge(&self, episodes: &[Episode]) -> Result<Vec<NewSemanticNode>>;

    /// Extract behavioral impressions from an interaction.
    fn extract_impressions(&self, interaction: &Interaction) -> Result<Vec<NewImpression>>;

    /// Detect whether two semantic nodes contradict each other.
    fn detect_contradiction(&self, a: &SemanticNode, b: &SemanticNode) -> Result<bool>;
}

/// A no-op provider for when no LLM is available.
/// Consolidation and perfuming simply skip the LLM-dependent steps.
pub struct NoOpProvider;

impl ConsolidationProvider for NoOpProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(vec![])
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(vec![])
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}

/// Trait for automatic embedding generation.
///
/// Implement this to auto-embed episodes, semantic nodes, and queries.
/// When no provider is set, embeddings must be provided manually.
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding vector for the given text.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts. Default implementation
    /// calls `embed()` for each text sequentially.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

/// Mock embedding provider for tests. Returns deterministic embeddings
/// based on a hash of the input text.
pub struct MockEmbeddingProvider {
    dim: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl EmbeddingProvider for MockEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Simple hash-based deterministic embedding
        let hash = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let emb: Vec<f32> = (0..self.dim)
            .map(|i| {
                let val =
                    ((hash.wrapping_mul((i as u64).wrapping_add(1))) % 1000) as f32 / 1000.0;
                val * 2.0 - 1.0 // normalize to [-1, 1]
            })
            .collect();
        Ok(emb)
    }
}

#[cfg(test)]
pub struct MockProvider {
    pub knowledge: Vec<NewSemanticNode>,
    pub impressions: Vec<NewImpression>,
}

#[cfg(test)]
impl MockProvider {
    pub fn empty() -> Self {
        Self {
            knowledge: vec![],
            impressions: vec![],
        }
    }

    pub fn with_knowledge(knowledge: Vec<NewSemanticNode>) -> Self {
        Self {
            knowledge,
            impressions: vec![],
        }
    }

    pub fn with_impressions(impressions: Vec<NewImpression>) -> Self {
        Self {
            knowledge: vec![],
            impressions,
        }
    }
}

#[cfg(test)]
impl ConsolidationProvider for MockProvider {
    fn extract_knowledge(&self, _episodes: &[Episode]) -> Result<Vec<NewSemanticNode>> {
        Ok(self.knowledge.clone())
    }

    fn extract_impressions(&self, _interaction: &Interaction) -> Result<Vec<NewImpression>> {
        Ok(self.impressions.clone())
    }

    fn detect_contradiction(&self, _a: &SemanticNode, _b: &SemanticNode) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedding_provider_embed() {
        let provider = MockEmbeddingProvider::new(3);
        let emb = provider.embed("hello world").unwrap();
        assert_eq!(emb.len(), 3);
    }

    #[test]
    fn test_mock_embedding_provider_batch() {
        let provider = MockEmbeddingProvider::new(4);
        let results = provider.embed_batch(&["hello", "world"]).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 4);
        assert_eq!(results[1].len(), 4);
    }

    #[test]
    fn test_mock_embedding_provider_deterministic() {
        let provider = MockEmbeddingProvider::new(4);
        let emb1 = provider.embed("same text").unwrap();
        let emb2 = provider.embed("same text").unwrap();
        assert_eq!(emb1, emb2, "same input should produce same embedding");
    }

    #[test]
    fn test_mock_embedding_provider_different_inputs() {
        let provider = MockEmbeddingProvider::new(4);
        let emb1 = provider.embed("hello").unwrap();
        let emb2 = provider.embed("world").unwrap();
        assert_ne!(emb1, emb2, "different inputs should produce different embeddings");
    }
}
