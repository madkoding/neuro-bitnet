//! Regex patterns for query classification
//!
//! This module defines weighted patterns for classifying user queries.
//! Each pattern has an associated weight that determines its importance
//! in the classification scoring.

use once_cell::sync::Lazy;
use regex::Regex;

mod patterns_es;

/// A pattern with an associated weight for scoring
#[derive(Debug, Clone)]
pub struct WeightedPattern {
    pub pattern: &'static str,
    pub weight: f32,
}

impl WeightedPattern {
    /// Create a new weighted pattern
    pub const fn new(pattern: &'static str, weight: f32) -> Self {
        Self { pattern, weight }
    }
}

/// Compiled weighted pattern for efficient matching
#[derive(Debug)]
pub struct CompiledPattern {
    pub regex: Regex,
    pub weight: f32,
}

impl CompiledPattern {
    /// Create a new compiled pattern
    pub fn new(pattern: &WeightedPattern) -> Option<Self> {
        Regex::new(pattern.pattern).ok().map(|regex| Self {
            regex,
            weight: pattern.weight,
        })
    }
    
    /// Check if the pattern matches the text and return the weight
    pub fn score(&self, text: &str) -> f32 {
        if self.regex.is_match(text) {
            self.weight
        } else {
            0.0
        }
    }
}

/// Pre-compiled regex patterns for each query category
pub struct QueryPatterns {
    pub math: Vec<CompiledPattern>,
    pub code: Vec<CompiledPattern>,
    pub reasoning: Vec<CompiledPattern>,
    pub tools: Vec<CompiledPattern>,
    pub greeting: Vec<CompiledPattern>,
    pub factual: Vec<CompiledPattern>,
}

impl QueryPatterns {
    /// Create a new set of query patterns (English + Spanish)
    pub fn new() -> Self {
        Self {
            math: compile_patterns(&build_math_patterns()),
            code: compile_patterns(&build_code_patterns()),
            reasoning: compile_patterns(&build_reasoning_patterns()),
            tools: compile_patterns(&build_tools_patterns()),
            greeting: compile_patterns(&build_greeting_patterns()),
            factual: compile_patterns(&build_factual_patterns()),
        }
    }
    
    /// Calculate total weighted score for a category
    pub fn score_category(patterns: &[CompiledPattern], text: &str) -> f32 {
        patterns.iter().map(|p| p.score(text)).sum()
    }
}

impl Default for QueryPatterns {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile a list of weighted patterns into regex patterns
fn compile_patterns(patterns: &[WeightedPattern]) -> Vec<CompiledPattern> {
    patterns
        .iter()
        .filter_map(CompiledPattern::new)
        .collect()
}

/// Global singleton for patterns (compiled once)
pub static PATTERNS: Lazy<QueryPatterns> = Lazy::new(QueryPatterns::new);

// ============================================================================
// MATH PATTERNS
// ============================================================================

fn build_math_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // Mathematical operations - high priority
        WeightedPattern::new(r"(?i)\b\d+\s*[\+\-\*\/\^]\s*\d+", 1.5),
        WeightedPattern::new(r"(?i)\bcalcul(a|e|ate)", 1.0),
        WeightedPattern::new(r"(?i)\bsolve\b", 1.0),
        WeightedPattern::new(r"(?i)\bequation\b", 1.0),
        WeightedPattern::new(r"(?i)\bmath(ematic)?s?\b", 1.0),
        WeightedPattern::new(r"(?i)\bformula\b", 1.0),
        WeightedPattern::new(r"(?i)\balgebra(ic)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bgeometry\b", 1.0),
        WeightedPattern::new(r"(?i)\btrigonometry\b", 1.0),
        WeightedPattern::new(r"(?i)\bcalculus\b", 1.0),
        WeightedPattern::new(r"(?i)\bderivative\b", 1.0),
        WeightedPattern::new(r"(?i)\bintegral\b", 1.0),
        WeightedPattern::new(r"(?i)\bstatistics?\b", 1.0),
        WeightedPattern::new(r"(?i)\bprobability\b", 1.0),
        WeightedPattern::new(r"(?i)\bpercentage\b", 1.0),
        WeightedPattern::new(r"(?i)\bfraction\b", 1.0),
        // Increased weight for specific math terms to beat "What is" factual pattern
        WeightedPattern::new(r"(?i)\bsquare root\b", 1.8),
        WeightedPattern::new(r"(?i)\blogarithm\b", 1.5),
        WeightedPattern::new(r"(?i)\bexponent\b", 1.5),
        WeightedPattern::new(r"(?i)\bprime number\b", 1.5),
        WeightedPattern::new(r"(?i)\bfactorial\b", 1.5),
        WeightedPattern::new(r"(?i)\bsum of\b", 1.0),
        WeightedPattern::new(r"(?i)\bproduct of\b", 1.0),
        WeightedPattern::new(r"(?i)\baverage\b", 1.0),
        WeightedPattern::new(r"(?i)\bmean\b", 0.8),
        WeightedPattern::new(r"(?i)\bmedian\b", 1.0),
        WeightedPattern::new(r"(?i)\bstandard deviation\b", 1.0),
        WeightedPattern::new(r"(?i)\bwhat is \d+", 1.2),
        WeightedPattern::new(r"(?i)\bhow much is\b", 1.2),
        WeightedPattern::new(r"(?i)\bconvert\s+\d+", 1.0),
        // Percentage with number - very high priority
        WeightedPattern::new(r"(?i)\b\d+\s*%\s*(of|de)\b", 2.0),
        WeightedPattern::new(r"(?i)\bwhat\s+is\s+\d+\s*%", 2.0),
        
        // NEW: Word problems patterns
        WeightedPattern::new(r"(?i)\bif\s+(i|you|we|they)\s+(have|had)\s+\d+", 1.5),
        WeightedPattern::new(r"(?i)\bif\s+there\s+(are|is|were|was)\s+\d+", 1.5),
        WeightedPattern::new(r"(?i)\bhow\s+many\s+.*\bleft\b", 1.2),
        WeightedPattern::new(r"(?i)\bhow\s+many\s+.*\bin\s+total\b", 1.2),
        WeightedPattern::new(r"(?i)\btotal\s+(cost|price|amount|number)\b", 1.2),
        WeightedPattern::new(r"(?i)\b(faster|slower|more|less)\s+than\b", 1.0),
        WeightedPattern::new(r"(?i)\bspeed\s+of\b", 0.8),
        WeightedPattern::new(r"(?i)\bdistance\s+(from|to|between)\b", 1.0),
        WeightedPattern::new(r"(?i)\btime\s+to\s+(travel|reach|complete)\b", 1.0),
        WeightedPattern::new(r"(?i)\barea\s+of\s+(a|an|the)?\s*(circle|square|rectangle|triangle)\b", 1.5),
        WeightedPattern::new(r"(?i)\bperimeter\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bvolume\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bradius\s+\d+", 1.2),
        WeightedPattern::new(r"(?i)\bsimplify\b", 1.0),
        WeightedPattern::new(r"(?i)\b\d+x\s*[\+\-]\s*\d+", 1.5), // Algebraic expressions like 3x + 2
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_math_patterns_es());
    patterns
}

// ============================================================================
// CODE PATTERNS
// ============================================================================

fn build_code_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // Programming keywords
        WeightedPattern::new(r"(?i)\bcode\b", 1.0),
        WeightedPattern::new(r"(?i)\bprogram(ming)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bfunction\b", 1.0),
        WeightedPattern::new(r"(?i)\bclass\b", 0.8),
        WeightedPattern::new(r"(?i)\bmethod\b", 1.0),
        WeightedPattern::new(r"(?i)\bvariable\b", 1.0),
        WeightedPattern::new(r"(?i)\bloop\b", 1.0),
        WeightedPattern::new(r"(?i)\barray\b", 1.0),
        WeightedPattern::new(r"(?i)\blist\b", 0.6),
        WeightedPattern::new(r"(?i)\bdict(ionary)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bstring\b", 0.8),
        WeightedPattern::new(r"(?i)\binteger\b", 0.8),
        WeightedPattern::new(r"(?i)\bfloat\b", 0.8),
        WeightedPattern::new(r"(?i)\bboolean\b", 1.0),
        WeightedPattern::new(r"(?i)\bnull\b", 0.8),
        WeightedPattern::new(r"(?i)\bundefined\b", 1.0),
        WeightedPattern::new(r"(?i)\breturn\b", 0.8),
        WeightedPattern::new(r"(?i)\bif\s+else\b", 1.0),
        WeightedPattern::new(r"(?i)\bfor\s+loop\b", 1.0),
        WeightedPattern::new(r"(?i)\bwhile\s+loop\b", 1.0),
        
        // Languages
        WeightedPattern::new(r"(?i)\bpython\b", 1.0),
        WeightedPattern::new(r"(?i)\bjavascript\b", 1.0),
        WeightedPattern::new(r"(?i)\btypescript\b", 1.0),
        WeightedPattern::new(r"(?i)\brust\b", 1.0),
        WeightedPattern::new(r"(?i)\bjava\b", 1.0),
        WeightedPattern::new(r"(?i)\bc\+\+\b", 1.0),
        WeightedPattern::new(r"(?i)\bgo(lang)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bruby\b", 1.0),
        WeightedPattern::new(r"(?i)\bphp\b", 1.0),
        WeightedPattern::new(r"(?i)\bswift\b", 1.0),
        WeightedPattern::new(r"(?i)\bkotlin\b", 1.0),
        
        // NEW: SQL and database
        WeightedPattern::new(r"(?i)\bsql\b", 1.2),
        WeightedPattern::new(r"(?i)\bquery\b", 1.0),
        WeightedPattern::new(r"(?i)\bdatabase\b", 1.0),
        WeightedPattern::new(r"(?i)\bselect\s+.*\s+from\b", 1.5),
        WeightedPattern::new(r"(?i)\binsert\s+into\b", 1.5),
        WeightedPattern::new(r"(?i)\bupdate\s+.*\s+set\b", 1.5),
        WeightedPattern::new(r"(?i)\bdelete\s+from\b", 1.5),
        WeightedPattern::new(r"(?i)\bjoin\s+(on|table)\b", 1.2),
        WeightedPattern::new(r"(?i)\bwhere\s+clause\b", 1.2),
        
        // NEW: Regex
        WeightedPattern::new(r"(?i)\bregex\b", 1.2),
        WeightedPattern::new(r"(?i)\bregexp?\b", 1.2),
        WeightedPattern::new(r"(?i)\bregular\s+expression\b", 1.5),
        WeightedPattern::new(r"(?i)\bpattern\s+match(ing)?\b", 1.2),
        
        // Actions
        WeightedPattern::new(r"(?i)\bdebug\b", 1.0),
        WeightedPattern::new(r"(?i)\bcompile\b", 1.0),
        WeightedPattern::new(r"(?i)\bexecut(e|ion)\b", 0.8),
        WeightedPattern::new(r"(?i)\bimplement\b", 1.0),
        WeightedPattern::new(r"(?i)\brefactor\b", 1.0),
        WeightedPattern::new(r"(?i)\boptimize\b", 0.8),
        WeightedPattern::new(r"(?i)\bfix\s+(the\s+)?(bug|error|issue)\b", 1.2),
        WeightedPattern::new(r"(?i)\bwrite\s+(a\s+)?(code|function|program|script)\b", 1.2),
        WeightedPattern::new(r"(?i)\bhow\s+to\s+(code|program|implement)\b", 1.0),
        
        // Code blocks and syntax
        WeightedPattern::new(r"```", 1.5),
        WeightedPattern::new(r"(?i)\bsyntax\b", 1.0),
        WeightedPattern::new(r"(?i)\bapi\b", 1.0),
        WeightedPattern::new(r"(?i)\bsdk\b", 1.0),
        WeightedPattern::new(r"(?i)\blibrary\b", 0.8),
        WeightedPattern::new(r"(?i)\bframework\b", 1.0),
        WeightedPattern::new(r"(?i)\bpackage\b", 0.8),
        WeightedPattern::new(r"(?i)\bmodule\b", 0.8),
        WeightedPattern::new(r"(?i)\bimport\b", 0.8),
        WeightedPattern::new(r"(?i)\bexport\b", 0.6),
        WeightedPattern::new(r"(?i)\balgorithm\b", 1.0),
        WeightedPattern::new(r"(?i)\bdata\s+structure\b", 1.2),
        
        // NEW: Inline code detection
        WeightedPattern::new(r"(?i)\bdef\s+\w+\s*\(", 1.5), // Python function def
        WeightedPattern::new(r"(?i)\bfn\s+\w+\s*\(", 1.5), // Rust function
        WeightedPattern::new(r"(?i)\bfunction\s+\w+\s*\(", 1.5), // JS function
        WeightedPattern::new(r"(?i)\bclass\s+\w+\s*[:\{]", 1.5), // Class definition
        WeightedPattern::new(r"(?i)=>\s*\{", 1.2), // Arrow function
        WeightedPattern::new(r"(?i)\breturn\s+\w+", 1.0), // Return statement
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_code_patterns_es());
    patterns
}

// ============================================================================
// REASONING PATTERNS
// ============================================================================

fn build_reasoning_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // Analysis - standard priority
        WeightedPattern::new(r"(?i)\banalyze\b", 1.0),
        WeightedPattern::new(r"(?i)\banalysis\b", 1.0),
        WeightedPattern::new(r"(?i)\bcompare\b", 1.0),
        WeightedPattern::new(r"(?i)\bcomparison\b", 1.0),
        WeightedPattern::new(r"(?i)\bcontrast\b", 1.0),
        WeightedPattern::new(r"(?i)\bevaluate\b", 1.0),
        WeightedPattern::new(r"(?i)\bevaluation\b", 1.0),
        WeightedPattern::new(r"(?i)\bcritique\b", 1.0),
        WeightedPattern::new(r"(?i)\bcritical\b", 0.8),
        
        // Compare X and Y pattern - high priority to beat code when comparing languages
        WeightedPattern::new(r"(?i)^compare\s+\w+\s+and\s+\w+", 2.5),
        WeightedPattern::new(r"(?i)\bcompare\s+\w+\s+(and|vs\.?|versus|with)\s+\w+", 2.0),
        
        // Pros and cons - high priority
        WeightedPattern::new(r"(?i)\bpros\s+and\s+cons\b", 2.0),
        // What are the advantages/disadvantages/tradeoffs - very high to beat factual
        WeightedPattern::new(r"(?i)\bwhat\s+are\s+the\s+(advantages?|disadvantages?|tradeoffs?|benefits?)\b", 2.5),
        WeightedPattern::new(r"(?i)\badvantages?\b", 1.5),
        WeightedPattern::new(r"(?i)\bdisadvantages?\b", 1.5),
        WeightedPattern::new(r"(?i)\bbenefits?\b", 1.2),
        WeightedPattern::new(r"(?i)\bdrawbacks?\b", 1.5),
        WeightedPattern::new(r"(?i)\btradeoffs?\b", 1.8),
        
        // Why questions - high priority
        WeightedPattern::new(r"(?i)^why\s+(is|are|do|does|would|should|did|was|were)\b", 2.0),
        WeightedPattern::new(r"(?i)\bwhy\s+(is|are|do|does|would|should)\b", 1.5),
        WeightedPattern::new(r"(?i)\bexplain\s+why\b", 1.5),
        WeightedPattern::new(r"(?i)\breason(ing|s)?\b", 1.0),
        WeightedPattern::new(r"(?i)\blogic(al)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bargument\b", 1.0),
        WeightedPattern::new(r"(?i)\bhypothesis\b", 1.0),
        WeightedPattern::new(r"(?i)\bconclusion\b", 1.0),
        WeightedPattern::new(r"(?i)\binfer(ence)?\b", 1.0),
        WeightedPattern::new(r"(?i)\bdeduc(e|tion)\b", 1.0),
        WeightedPattern::new(r"(?i)\binduc(e|tion)\b", 1.0),
        
        // Thinking and deciding
        WeightedPattern::new(r"(?i)\bthink\s+(about|through)\b", 1.0),
        WeightedPattern::new(r"(?i)\bconsider\b", 1.0),
        WeightedPattern::new(r"(?i)\bweigh\b", 1.0),
        WeightedPattern::new(r"(?i)\bassess\b", 1.0),
        WeightedPattern::new(r"(?i)\bjudge\b", 0.8),
        WeightedPattern::new(r"(?i)\bdecide\b", 1.0),
        WeightedPattern::new(r"(?i)\bdecision\b", 1.0),
        
        // NEW: Hypothetical questions - high priority
        WeightedPattern::new(r"(?i)\bwhat\s+if\b", 2.0),
        WeightedPattern::new(r"(?i)\bwhat\s+would\s+happen\b", 2.0),
        WeightedPattern::new(r"(?i)\bimagine\s+(if|that)\b", 1.5),
        WeightedPattern::new(r"(?i)\bsuppose\b", 1.5),
        WeightedPattern::new(r"(?i)\bhypothetically\b", 1.5),
        WeightedPattern::new(r"(?i)\bin\s+theory\b", 1.2),
        WeightedPattern::new(r"(?i)\bcould\s+.*\bpossibly\b", 1.2),
        WeightedPattern::new(r"(?i)\bwould\s+it\s+be\s+possible\b", 1.2),
        WeightedPattern::new(r"(?i)\bif\s+.*\bdidn'?t\s+exist\b", 1.5),
        
        // NEW: Should I questions - high priority
        WeightedPattern::new(r"(?i)^should\s+i\b", 2.0),
        WeightedPattern::new(r"(?i)\bshould\s+i\s+(learn|use|choose|pick|start)\b", 2.0),
        
        // NEW: Understanding/learning patterns - high priority for reasoning over factual
        WeightedPattern::new(r"(?i)\bunderstanding\s+how\b", 1.8),
        WeightedPattern::new(r"(?i)\bunderstand\s+how\b", 1.8),
        WeightedPattern::new(r"(?i)\bhow\s+.*\s+work(s)?\s+and\b", 1.8),
        WeightedPattern::new(r"(?i)\bhow\s+.*\s+can\s+be\s+(applied|used)\b", 1.8),
        WeightedPattern::new(r"(?i)\bapplied\s+to\s+solve\b", 1.5),
        WeightedPattern::new(r"(?i)\breal-?world\s+problems?\b", 1.5),
        WeightedPattern::new(r"(?i)\bwhich\s+(is|one\s+is)\s+better\b", 1.5),
        WeightedPattern::new(r"(?i)\bbetter\s+to\s+(use|learn|choose)\b", 1.5),
        WeightedPattern::new(r"(?i)\b(python|javascript|rust)\s+(or|vs\.?)\s+(python|javascript|rust)\b", 1.5),
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_reasoning_patterns_es());
    patterns
}

// ============================================================================
// TOOLS PATTERNS
// ============================================================================

fn build_tools_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // Search - high priority
        WeightedPattern::new(r"(?i)\bsearch\s+(for|the\s+web)\b", 1.2),
        WeightedPattern::new(r"(?i)\blook\s+up\b", 1.0),
        WeightedPattern::new(r"(?i)\bfind\s+(information|data|results)\b", 1.0),
        WeightedPattern::new(r"(?i)\bweb\s+search\b", 1.2),
        WeightedPattern::new(r"(?i)\bgoogle\b", 1.0),
        WeightedPattern::new(r"(?i)\bbrowse\b", 0.8),
        WeightedPattern::new(r"(?i)\bopen\s+(a\s+)?(file|url|link|website)\b", 1.0),
        
        // Files
        WeightedPattern::new(r"(?i)\bdownload\b", 1.0),
        WeightedPattern::new(r"(?i)\bupload\b", 0.8),
        WeightedPattern::new(r"(?i)\bsave\s+(to|as)\b", 1.0),
        WeightedPattern::new(r"(?i)\bexport\s+to\b", 1.0),
        WeightedPattern::new(r"(?i)\bconvert\s+to\b", 1.0),
        
        // NEW: Image generation - high priority
        WeightedPattern::new(r"(?i)\bgenerate\s+(an?\s+)?(image|picture|diagram|chart|photo|illustration)\b", 1.5),
        WeightedPattern::new(r"(?i)\bcreate\s+(an?\s+)?(image|picture|photo|illustration|diagram)\b", 1.5),
        WeightedPattern::new(r"(?i)\bdraw\s+(a|an|me)?\b", 1.5),
        WeightedPattern::new(r"(?i)\bmake\s+(an?\s+)?(image|picture|photo)\b", 1.5),
        
        // Documents
        WeightedPattern::new(r"(?i)\bcreate\s+(a\s+)?(file|document|report)\b", 1.0),
        WeightedPattern::new(r"(?i)\bsend\s+(an?\s+)?(email|message)\b", 1.0),
        WeightedPattern::new(r"(?i)\bschedule\b", 1.0),
        WeightedPattern::new(r"(?i)\breminder\b", 1.0),
        WeightedPattern::new(r"(?i)\balarm\b", 1.0),
        WeightedPattern::new(r"(?i)\btimer\b", 1.0),
        WeightedPattern::new(r"(?i)\bcalendar\b", 1.0),
        
        // Real-time info
        WeightedPattern::new(r"(?i)\bweather\b", 1.0),
        WeightedPattern::new(r"(?i)\bnews\b", 1.0),
        // Stock price - very high priority to beat factual "What is"
        WeightedPattern::new(r"(?i)\bstock\s+price\b", 2.0),
        WeightedPattern::new(r"(?i)\bstock\s+price\s+of\b", 2.5),
        WeightedPattern::new(r"(?i)\bprice\s+of\s+.*\b(stock|share)s?\b", 2.0),
        WeightedPattern::new(r"(?i)\btranslate\b", 1.0),
        WeightedPattern::new(r"(?i)\btranslation\b", 1.0),
        
        // Latest/current info
        WeightedPattern::new(r"(?i)\blatest\s+(news|updates?)\b", 1.2),
        WeightedPattern::new(r"(?i)\bcurrent\s+(price|weather|time)\b", 1.2),
        WeightedPattern::new(r"(?i)\btoday'?s?\s+(weather|news|date)\b", 1.2),
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_tools_patterns_es());
    patterns
}

// ============================================================================
// GREETING PATTERNS
// ============================================================================

fn build_greeting_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // Direct greetings - very high priority
        WeightedPattern::new(r"(?i)^(hi|hello|hey)\b", 2.0),
        WeightedPattern::new(r"(?i)^good\s+(morning|afternoon|evening|night)\b", 2.0),
        WeightedPattern::new(r"(?i)^(what'?s?\s+up|sup|yo)\b", 1.5),
        WeightedPattern::new(r"(?i)^(how\s+are\s+you|how'?s?\s+it\s+going)\b", 2.0),
        WeightedPattern::new(r"(?i)^(nice|pleased)\s+to\s+meet\s+you\b", 1.5),
        WeightedPattern::new(r"(?i)^greetings?\b", 2.0),
        
        // Farewells
        WeightedPattern::new(r"(?i)\bbye\b", 1.0),
        WeightedPattern::new(r"(?i)\bgoodbye\b", 1.0),
        WeightedPattern::new(r"(?i)\bsee\s+you\b", 1.0),
        WeightedPattern::new(r"(?i)\btake\s+care\b", 1.0),
        WeightedPattern::new(r"(?i)\bhave\s+a\s+(nice|good|great)\s+(day|night|one)\b", 1.0),
        
        // Courtesy
        WeightedPattern::new(r"(?i)^thanks?\b", 1.0),
        WeightedPattern::new(r"(?i)^thank\s+you\b", 1.0),
        WeightedPattern::new(r"(?i)^(please|pls)\b", 0.8),
        WeightedPattern::new(r"(?i)^sorry\b", 1.0),
        WeightedPattern::new(r"(?i)^excuse\s+me\b", 1.0),
        
        // About the assistant - high priority
        WeightedPattern::new(r"(?i)\bwho\s+are\s+you\b", 1.5),
        WeightedPattern::new(r"(?i)\bwhat\s+is\s+your\s+name\b", 1.5),
        WeightedPattern::new(r"(?i)\bwhat\s+can\s+you\s+do\b", 1.5),
        WeightedPattern::new(r"(?i)\btell\s+me\s+about\s+yourself\b", 1.5),
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_greeting_patterns_es());
    patterns
}

// ============================================================================
// FACTUAL PATTERNS
// ============================================================================

fn build_factual_patterns() -> Vec<WeightedPattern> {
    let mut patterns = vec![
        // What/Who/When/Where questions - high priority
        WeightedPattern::new(r"(?i)^what\s+is\b", 1.5),
        WeightedPattern::new(r"(?i)^what\s+are\b", 1.5),
        WeightedPattern::new(r"(?i)^what\s+was\b", 1.5),
        WeightedPattern::new(r"(?i)^what\s+were\b", 1.5),
        WeightedPattern::new(r"(?i)^who\s+is\b", 1.5),
        WeightedPattern::new(r"(?i)^who\s+are\b", 1.5),
        WeightedPattern::new(r"(?i)^who\s+was\b", 1.5),
        WeightedPattern::new(r"(?i)^who\s+were\b", 1.5),
        WeightedPattern::new(r"(?i)^when\s+(is|was|did|does|will)\b", 1.5),
        WeightedPattern::new(r"(?i)^where\s+(is|are|was|were|do|does)\b", 1.5),
        WeightedPattern::new(r"(?i)^which\s+(is|are|was|were)\b", 1.2),
        WeightedPattern::new(r"(?i)^how\s+(many|much|old|long|far|tall|big|small)\b", 1.2),
        
        // Definitions
        WeightedPattern::new(r"(?i)\bdefine\b", 1.0),
        WeightedPattern::new(r"(?i)\bdefinition\b", 1.0),
        WeightedPattern::new(r"(?i)\bmeaning\s+of\b", 1.0),
        
        // History and origin
        WeightedPattern::new(r"(?i)\bhistory\s+of\b", 1.0),
        WeightedPattern::new(r"(?i)\borigin\s+of\b", 1.0),
        WeightedPattern::new(r"(?i)\bfact(s)?\s+about\b", 1.0),
        WeightedPattern::new(r"(?i)\binformation\s+(about|on)\b", 1.0),
        WeightedPattern::new(r"(?i)\btell\s+me\s+about\b", 1.0),
        WeightedPattern::new(r"(?i)\bexplain\s+(what|how|the)\b", 1.0),
        WeightedPattern::new(r"(?i)\bdescribe\b", 1.0),
        WeightedPattern::new(r"(?i)\bwhat\s+does\b.*\bmean\b", 1.0),
        
        // Specific data
        WeightedPattern::new(r"(?i)\bcapital\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bpopulation\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bpresident\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bceo\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bfounder\s+of\b", 1.2),
        WeightedPattern::new(r"(?i)\bauthor\s+of\b", 1.0),
        WeightedPattern::new(r"(?i)\bdirector\s+of\b", 1.0),
        
        // NEW: Invented/Discovered - high priority
        WeightedPattern::new(r"(?i)\binventor\s+of\b", 1.5),
        WeightedPattern::new(r"(?i)\bwho\s+invented\b", 2.0),
        WeightedPattern::new(r"(?i)\bwho\s+discovered\b", 2.0),
        WeightedPattern::new(r"(?i)\bwho\s+created\b", 2.0),
        WeightedPattern::new(r"(?i)\binvented\b", 1.5),
        WeightedPattern::new(r"(?i)\bdiscovered\b", 1.5),
        WeightedPattern::new(r"(?i)\bdiscoverer\s+of\b", 1.5),
        WeightedPattern::new(r"(?i)\bcreator\s+of\b", 1.5),
        WeightedPattern::new(r"(?i)\bwhen\s+was\s+.*\binvented\b", 2.0),
        WeightedPattern::new(r"(?i)\bwhen\s+was\s+.*\bdiscovered\b", 2.0),
    ];
    
    // Add Spanish patterns
    patterns.extend(patterns_es::build_factual_patterns_es());
    patterns
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_score(patterns: &[CompiledPattern], text: &str) -> f32 {
        QueryPatterns::score_category(patterns, text)
    }

    #[test]
    fn test_math_patterns() {
        let patterns = compile_patterns(&build_math_patterns());
        assert!(test_score(&patterns, "what is 2 + 2?") > 0.0);
        assert!(test_score(&patterns, "Calculate the sum") > 0.0);
        assert!(test_score(&patterns, "solve this equation") > 0.0);
        assert!(test_score(&patterns, "what is the derivative of x^2") > 0.0);
        assert!(test_score(&patterns, "cuánto es 5 + 3") > 0.0); // Spanish
        assert!(test_score(&patterns, "hello world") == 0.0);
    }

    #[test]
    fn test_math_word_problems() {
        let patterns = compile_patterns(&build_math_patterns());
        assert!(test_score(&patterns, "If I have 5 apples and give away 2") > 0.0);
        assert!(test_score(&patterns, "What is the area of a circle with radius 5") > 0.0);
    }

    #[test]
    fn test_code_patterns() {
        let patterns = compile_patterns(&build_code_patterns());
        assert!(test_score(&patterns, "write a function in Python") > 0.0);
        assert!(test_score(&patterns, "how to implement a class") > 0.0);
        assert!(test_score(&patterns, "fix the bug") > 0.0);
        assert!(test_score(&patterns, "```python\nprint('hello')```") > 0.0);
        // NEW: SQL
        assert!(test_score(&patterns, "Write a SQL query to select all users") > 0.0);
        // NEW: Regex
        assert!(test_score(&patterns, "Create a regex to match email addresses") > 0.0);
        // Spanish
        assert!(test_score(&patterns, "escribe una función en Python") > 0.0);
        assert!(test_score(&patterns, "what is the weather?") == 0.0);
    }

    #[test]
    fn test_greeting_patterns() {
        let patterns = compile_patterns(&build_greeting_patterns());
        assert!(test_score(&patterns, "hello") > 0.0);
        assert!(test_score(&patterns, "Hi there!") > 0.0);
        assert!(test_score(&patterns, "Good morning") > 0.0);
        assert!(test_score(&patterns, "what is your name") > 0.0);
        assert!(test_score(&patterns, "hola") > 0.0); // Spanish
        assert!(test_score(&patterns, "buenos días") > 0.0); // Spanish
        assert!(test_score(&patterns, "what is the capital of France") == 0.0);
    }

    #[test]
    fn test_factual_patterns() {
        let patterns = compile_patterns(&build_factual_patterns());
        assert!(test_score(&patterns, "What is the capital of France?") > 0.0);
        assert!(test_score(&patterns, "Who was Albert Einstein?") > 0.0);
        assert!(test_score(&patterns, "When was World War 2?") > 0.0);
        assert!(test_score(&patterns, "define photosynthesis") > 0.0);
        // NEW: invented/discovered
        assert!(test_score(&patterns, "Who invented the telephone?") > 0.0);
        assert!(test_score(&patterns, "Who discovered penicillin?") > 0.0);
        // Spanish
        assert!(test_score(&patterns, "quién inventó el teléfono") > 0.0);
        assert!(test_score(&patterns, "hello") == 0.0);
    }

    #[test]
    fn test_tools_patterns() {
        let patterns = compile_patterns(&build_tools_patterns());
        assert!(test_score(&patterns, "search the web for") > 0.0);
        assert!(test_score(&patterns, "generate an image of") > 0.0);
        assert!(test_score(&patterns, "translate to Spanish") > 0.0);
        assert!(test_score(&patterns, "what's the weather") > 0.0);
        // NEW: Image generation variants
        assert!(test_score(&patterns, "Create an image of a sunset") > 0.0);
        assert!(test_score(&patterns, "Draw me a cat") > 0.0);
        // Spanish
        assert!(test_score(&patterns, "generar una imagen") > 0.0);
        assert!(test_score(&patterns, "hello world") == 0.0);
    }

    #[test]
    fn test_reasoning_patterns() {
        let patterns = compile_patterns(&build_reasoning_patterns());
        assert!(test_score(&patterns, "analyze the pros and cons") > 0.0);
        assert!(test_score(&patterns, "why is the sky blue") > 0.0);
        // NEW: Hypothetical
        assert!(test_score(&patterns, "What would happen if gravity didn't exist?") > 0.0);
        assert!(test_score(&patterns, "What if we could time travel?") > 0.0);
        // NEW: Should I
        assert!(test_score(&patterns, "Should I learn Python or JavaScript first?") > 0.0);
        // Spanish
        assert!(test_score(&patterns, "qué pasaría si") > 0.0);
        assert!(test_score(&patterns, "ventajas y desventajas") > 0.0);
        assert!(test_score(&patterns, "hello") == 0.0);
    }

    #[test]
    fn test_weighted_scoring() {
        let patterns = compile_patterns(&build_reasoning_patterns());
        // "pros and cons" has weight 2.0, should score higher
        let score1 = test_score(&patterns, "pros and cons");
        let score2 = test_score(&patterns, "analyze");
        assert!(score1 > score2, "Weighted pattern should score higher");
    }
}
