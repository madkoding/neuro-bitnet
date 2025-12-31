//! Cosine similarity calculations using ndarray for SIMD optimization

use ndarray::{Array1, ArrayView1};

/// Calculate cosine similarity between two vectors
///
/// Returns a value between -1 and 1, where 1 means identical direction,
/// 0 means orthogonal, and -1 means opposite direction.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector (must have same length as `a`)
///
/// # Panics
/// Panics if vectors have different lengths
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");

    let a = ArrayView1::from(a);
    let b = ArrayView1::from(b);

    let dot = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Calculate cosine similarity between a query vector and multiple document vectors
///
/// # Arguments
/// * `query` - Query embedding vector
/// * `documents` - Slice of document embedding vectors
///
/// # Returns
/// Vector of similarity scores in the same order as documents
pub fn batch_cosine_similarity(query: &[f32], documents: &[Vec<f32>]) -> Vec<f32> {
    if documents.is_empty() {
        return Vec::new();
    }

    let query = Array1::from_vec(query.to_vec());
    let query_norm = query.dot(&query).sqrt();

    if query_norm == 0.0 {
        return vec![0.0; documents.len()];
    }

    documents
        .iter()
        .map(|doc| {
            let doc = ArrayView1::from(doc.as_slice());
            let doc_norm = doc.dot(&doc).sqrt();

            if doc_norm == 0.0 {
                0.0
            } else {
                query.dot(&doc) / (query_norm * doc_norm)
            }
        })
        .collect()
}

/// Find top-k most similar documents
///
/// # Arguments
/// * `query` - Query embedding vector
/// * `documents` - Slice of document embedding vectors
/// * `k` - Number of top results to return
///
/// # Returns
/// Vector of (index, similarity) tuples, sorted by similarity descending
pub fn top_k_similar(query: &[f32], documents: &[Vec<f32>], k: usize) -> Vec<(usize, f32)> {
    let similarities = batch_cosine_similarity(query, documents);

    let mut indexed: Vec<(usize, f32)> = similarities.into_iter().enumerate().collect();

    // Partial sort for efficiency when k << n
    if k < indexed.len() {
        indexed.select_nth_unstable_by(k, |a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        indexed.truncate(k);
    }

    // Final sort of top-k
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    indexed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_scaled() {
        // Cosine similarity should be scale-invariant
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![2.0, 4.0, 6.0]; // Same direction, different magnitude
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_batch_cosine_similarity() {
        let query = vec![1.0, 0.0, 0.0];
        let documents = vec![
            vec![1.0, 0.0, 0.0],  // identical
            vec![0.0, 1.0, 0.0],  // orthogonal
            vec![-1.0, 0.0, 0.0], // opposite
        ];

        let sims = batch_cosine_similarity(&query, &documents);
        assert_eq!(sims.len(), 3);
        assert!((sims[0] - 1.0).abs() < 1e-6);
        assert!(sims[1].abs() < 1e-6);
        assert!((sims[2] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_top_k_similar() {
        let query = vec![1.0, 0.0, 0.0];
        let documents = vec![
            vec![0.5, 0.5, 0.0],  // ~0.707
            vec![1.0, 0.0, 0.0],  // 1.0
            vec![0.0, 1.0, 0.0],  // 0.0
            vec![0.9, 0.1, 0.0],  // ~0.994
        ];

        let top = top_k_similar(&query, &documents, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 1); // Index 1 is most similar
        assert_eq!(top[1].0, 3); // Index 3 is second most similar
    }

    #[test]
    fn test_top_k_similar_empty() {
        let query = vec![1.0, 0.0];
        let documents: Vec<Vec<f32>> = vec![];
        let top = top_k_similar(&query, &documents, 5);
        assert!(top.is_empty());
    }
}
