//! Translation module using dictionary-based approach
//!
//! Uses phrase and word dictionaries for fast ES→EN translation.
//! This is faster and more reliable than using the model for translation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    Other,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Spanish",
            Language::Other => "Other",
        }
    }
}

/// Spanish to English phrase dictionary (sorted by length, longest first)
static ES_EN_PHRASES: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    let mut phrases = vec![
        // Questions - longest first to avoid partial matches
        ("cuál es la capital de", "what is the capital of"),
        ("cuál es el planeta más grande", "what is the largest planet"),
        ("quién escribió", "who wrote"),
        ("quién pintó", "who painted"),
        ("cuántos continentes hay", "how many continents are there"),
        ("cuántos continentes", "how many continents"),
        ("cuál es la", "what is the"),
        ("cuál es el", "what is the"),
        ("cuál es", "what is"),
        ("qué es", "what is"),
        ("quién es", "who is"),
        ("cómo se llama", "what is the name of"),
        ("dónde está", "where is"),
        ("dónde queda", "where is"),
        ("en qué año", "in what year"),
        ("por qué", "why"),
        // Common proper nouns
        ("don quijote", "Don Quixote"),
        ("mona lisa", "Mona Lisa"),
        // Countries
        ("estados unidos", "United States"),
        ("reino unido", "United Kingdom"),
    ];
    // Sort by length descending to match longer phrases first
    phrases.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    phrases
});

/// Spanish to English word dictionary
static ES_EN_DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Question words
    m.insert("qué", "what");
    m.insert("cuál", "which");
    m.insert("quién", "who");
    m.insert("cómo", "how");
    m.insert("dónde", "where");
    m.insert("cuándo", "when");
    m.insert("cuánto", "how much");
    m.insert("cuántos", "how many");
    m.insert("cuántas", "how many");
    m.insert("por", "for");
    // Verbs
    m.insert("es", "is");
    m.insert("son", "are");
    m.insert("está", "is");
    m.insert("están", "are");
    m.insert("hay", "are there");
    m.insert("tiene", "has");
    m.insert("tienen", "have");
    m.insert("fue", "was");
    m.insert("fueron", "were");
    m.insert("escribió", "wrote");
    m.insert("pintó", "painted");
    m.insert("descubrió", "discovered");
    m.insert("inventó", "invented");
    m.insert("fundó", "founded");
    m.insert("nació", "was born");
    m.insert("murió", "died");
    m.insert("ganó", "won");
    // Articles
    m.insert("el", "the");
    m.insert("la", "the");
    m.insert("los", "the");
    m.insert("las", "the");
    m.insert("un", "a");
    m.insert("una", "a");
    m.insert("unos", "some");
    m.insert("unas", "some");
    m.insert("del", "of the");
    m.insert("al", "to the");
    // Prepositions
    m.insert("de", "of");
    m.insert("en", "in");
    m.insert("con", "with");
    m.insert("para", "for");
    m.insert("sobre", "about");
    m.insert("entre", "between");
    m.insert("hacia", "towards");
    m.insert("desde", "from");
    m.insert("hasta", "until");
    // Adjectives
    m.insert("más", "most");
    m.insert("grande", "large");
    m.insert("pequeño", "small");
    m.insert("primer", "first");
    m.insert("primero", "first");
    m.insert("primera", "first");
    m.insert("segundo", "second");
    m.insert("última", "last");
    m.insert("último", "last");
    // Nouns
    m.insert("capital", "capital");
    m.insert("país", "country");
    m.insert("países", "countries");
    m.insert("planeta", "planet");
    m.insert("planetas", "planets");
    m.insert("continente", "continent");
    m.insert("continentes", "continents");
    m.insert("mundo", "world");
    m.insert("año", "year");
    m.insert("años", "years");
    m.insert("persona", "person");
    m.insert("personas", "people");
    m.insert("libro", "book");
    m.insert("obra", "work");
    m.insert("pintura", "painting");
    m.insert("autor", "author");
    m.insert("escritor", "writer");
    m.insert("presidente", "president");
    m.insert("rey", "king");
    m.insert("reina", "queen");
    // Countries
    m.insert("francia", "France");
    m.insert("españa", "Spain");
    m.insert("alemania", "Germany");
    m.insert("italia", "Italy");
    m.insert("japón", "Japan");
    m.insert("china", "China");
    m.insert("brasil", "Brazil");
    m.insert("méxico", "Mexico");
    m.insert("argentina", "Argentina");
    m.insert("chile", "Chile");
    m.insert("perú", "Peru");
    m.insert("colombia", "Colombia");
    m.insert("rusia", "Russia");
    m.insert("india", "India");
    m
});

/// Simple language detection based on common patterns
pub fn detect_language(text: &str) -> Language {
    let lower = text.to_lowercase();
    
    // Spanish indicators
    let spanish_markers = ["¿", "¡", "ñ", "á", "é", "í", "ó", "ú"];
    let spanish_words = ["qué", "cuál", "cómo", "dónde", "quién", "cuánto", 
                         "que", "cual", "como", "donde", "quien", "cuanto",
                         "es", "son", "está", "están", "hay", "tiene",
                         "del", "las", "los", "una", "uno"];
    
    // Check markers first
    for marker in spanish_markers {
        if lower.contains(marker) {
            return Language::Spanish;
        }
    }
    
    // Check common words
    let words: Vec<&str> = lower.split_whitespace().collect();
    let spanish_count = words.iter()
        .filter(|w| spanish_words.contains(&w.trim_matches(|c: char| !c.is_alphanumeric())))
        .count();
    
    if spanish_count >= 2 || (words.len() <= 5 && spanish_count >= 1) {
        return Language::Spanish;
    }
    
    Language::English
}

/// Translate Spanish text to English using dictionary
pub fn translate_to_english(text: &str) -> String {
    // Remove Spanish punctuation marks
    let clean = text.replace("¿", "").replace("¡", "");
    let mut result = clean.to_lowercase();
    
    // First, apply phrase translations (longest first)
    for (es, en) in ES_EN_PHRASES.iter() {
        result = result.replace(es, en);
    }
    
    // Then, translate remaining words
    let words: Vec<&str> = result.split_whitespace().collect();
    let translated_words: Vec<String> = words.iter().map(|word| {
        // Remove punctuation for lookup but preserve for output
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        let suffix = word.chars().skip(clean_word.len()).collect::<String>();
        
        if let Some(translation) = ES_EN_DICT.get(clean_word) {
            format!("{}{}", translation, suffix)
        } else {
            // Keep original (might be proper noun or already English)
            word.to_string()
        }
    }).collect();
    
    let translated = translated_words.join(" ");
    
    // Capitalize first letter and add question mark if needed
    let mut chars: Vec<char> = translated.chars().collect();
    if !chars.is_empty() {
        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
    }
    let mut final_text: String = chars.into_iter().collect();
    
    // Add question mark if original had one
    if text.contains("?") && !final_text.ends_with("?") {
        final_text.push('?');
    }
    
    final_text
}

/// Build a translation - now uses dictionary instead of model
pub fn build_translation_prompt(text: &str) -> String {
    // For backward compatibility, but we now translate directly
    translate_to_english(text)
}

/// Build a prompt that asks for response in specific language  
pub fn build_multilingual_prompt(question: &str, response_language: Language) -> String {
    match response_language {
        Language::Spanish => format!("{}\nResponde brevemente en español.", question),
        Language::English => question.to_string(),
        Language::Other => question.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_spanish() {
        assert_eq!(detect_language("¿Cuál es la capital de Francia?"), Language::Spanish);
        assert_eq!(detect_language("¿Cuántos continentes hay?"), Language::Spanish);
        assert_eq!(detect_language("Quién pintó la Mona Lisa"), Language::Spanish);
    }

    #[test]
    fn test_detect_english() {
        assert_eq!(detect_language("What is the capital of France?"), Language::English);
        assert_eq!(detect_language("How many continents are there?"), Language::English);
    }

    #[test]
    fn test_translation() {
        assert_eq!(
            translate_to_english("¿Cuál es la capital de Francia?"),
            "What is the capital of France?"
        );
        assert_eq!(
            translate_to_english("¿Quién escribió Don Quijote?"),
            "Who wrote Don Quixote?"
        );
        assert_eq!(
            translate_to_english("¿Quién pintó la Mona Lisa?"),
            "Who painted the Mona Lisa?"
        );
    }
}
