//! Text analysis functions.

use std::collections::BTreeMap;
use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// Average reading speed in words per minute
const WORDS_PER_MINUTE: f64 = 200.0;

// =============================================================================
// word_count(s) -> number
// =============================================================================

defn!(WordCountFn, vec![arg!(string)], None);

impl Function for WordCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let count = s.split_whitespace().count();
        Ok(Value::Number(Number::from(count)))
    }
}

// =============================================================================
// char_count(s) -> number
// =============================================================================

defn!(CharCountFn, vec![arg!(string)], None);

impl Function for CharCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let count = s.chars().filter(|c| !c.is_whitespace()).count();
        Ok(Value::Number(Number::from(count)))
    }
}

// =============================================================================
// sentence_count(s) -> number
// =============================================================================

defn!(SentenceCountFn, vec![arg!(string)], None);

impl Function for SentenceCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        if s.trim().is_empty() {
            return Ok(Value::Number(Number::from(0)));
        }

        // Count sentence-ending punctuation
        let count = s
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count();

        // If no sentence-ending punctuation but has content, count as 1
        let count = if count == 0 && !s.trim().is_empty() {
            1
        } else {
            count
        };

        Ok(Value::Number(Number::from(count)))
    }
}

// =============================================================================
// paragraph_count(s) -> number
// =============================================================================

defn!(ParagraphCountFn, vec![arg!(string)], None);

impl Function for ParagraphCountFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        // Split by double newlines (paragraph separator)
        let count = s.split("\n\n").filter(|p| !p.trim().is_empty()).count();

        Ok(Value::Number(Number::from(count)))
    }
}

// =============================================================================
// reading_time(s) -> number (minutes)
// =============================================================================

defn!(ReadingTimeFn, vec![arg!(string)], None);

impl Function for ReadingTimeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let word_count = s.split_whitespace().count() as f64;
        let minutes = (word_count / WORDS_PER_MINUTE).ceil();
        Ok(number_value(minutes))
    }
}

// =============================================================================
// reading_time_seconds(s) -> number (seconds)
// =============================================================================

defn!(ReadingTimeSecondsFn, vec![arg!(string)], None);

impl Function for ReadingTimeSecondsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let word_count = s.split_whitespace().count() as f64;
        let seconds = (word_count / WORDS_PER_MINUTE) * 60.0;
        Ok(number_value(seconds.ceil()))
    }
}

// =============================================================================
// char_frequencies(s) -> object
// =============================================================================

defn!(CharFrequenciesFn, vec![arg!(string)], None);

impl Function for CharFrequenciesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let mut freq: BTreeMap<char, usize> = BTreeMap::new();
        for c in s.chars() {
            if !c.is_whitespace() {
                *freq.entry(c).or_insert(0) += 1;
            }
        }

        let obj: serde_json::Map<String, Value> = freq
            .into_iter()
            .map(|(k, v)| (k.to_string(), Value::Number(Number::from(v))))
            .collect();

        Ok(Value::Object(obj))
    }
}

// =============================================================================
// word_frequencies(s) -> object
// =============================================================================

defn!(WordFrequenciesFn, vec![arg!(string)], None);

impl Function for WordFrequenciesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let mut freq: BTreeMap<String, usize> = BTreeMap::new();
        for word in s.split_whitespace() {
            // Normalize: lowercase and remove punctuation
            let normalized: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            if !normalized.is_empty() {
                *freq.entry(normalized).or_insert(0) += 1;
            }
        }

        let obj: serde_json::Map<String, Value> = freq
            .into_iter()
            .map(|(k, v)| (k, Value::Number(Number::from(v))))
            .collect();

        Ok(Value::Object(obj))
    }
}

// =============================================================================
// ngrams(s, n, type?) -> array
// Generate n-grams from text. Type can be "word" (default) or "char".
// =============================================================================

defn!(
    NgramsFn,
    vec![arg!(string), arg!(number)],
    Some(arg!(string))
);

impl Function for NgramsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let n = args[1].as_f64().unwrap() as usize;

        // Default to "word" if not specified
        let ngram_type = if args.len() > 2 {
            args[2].as_str().unwrap_or("word")
        } else {
            "word"
        };

        if n == 0 {
            return Ok(Value::Array(vec![]));
        }

        let result = match ngram_type {
            "char" => {
                // Character n-grams
                let chars: Vec<char> = s.chars().collect();
                if chars.len() < n {
                    vec![]
                } else {
                    chars
                        .windows(n)
                        .map(|w| Value::String(w.iter().collect()))
                        .collect()
                }
            }
            _ => {
                // Word n-grams (default)
                let words: Vec<&str> = s.split_whitespace().collect();
                if words.len() < n {
                    vec![]
                } else {
                    words
                        .windows(n)
                        .map(|w| {
                            let arr: Vec<Value> = w
                                .iter()
                                .map(|word| Value::String(word.to_string()))
                                .collect();
                            Value::Array(arr)
                        })
                        .collect()
                }
            }
        };

        Ok(Value::Array(result))
    }
}

// =============================================================================
// bigrams(s) -> array
// Convenience function for word bigrams (2-grams).
// =============================================================================

defn!(BigramsFn, vec![arg!(string)], None);

impl Function for BigramsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() < 2 {
            return Ok(Value::Array(vec![]));
        }

        let result: Vec<Value> = words
            .windows(2)
            .map(|w| {
                let arr: Vec<Value> = w
                    .iter()
                    .map(|word| Value::String(word.to_string()))
                    .collect();
                Value::Array(arr)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// trigrams(s) -> array
// Convenience function for word trigrams (3-grams).
// =============================================================================

defn!(TrigramsFn, vec![arg!(string)], None);

impl Function for TrigramsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() < 3 {
            return Ok(Value::Array(vec![]));
        }

        let result: Vec<Value> = words
            .windows(3)
            .map(|w| {
                let arr: Vec<Value> = w
                    .iter()
                    .map(|word| Value::String(word.to_string()))
                    .collect();
                Value::Array(arr)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// tokens(s) -> array
// Simple word tokenization: lowercase, strip punctuation
// =============================================================================

defn!(TokensFn, vec![arg!(string)], None);

impl Function for TokensFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let tokens: Vec<Value> = s
            .split_whitespace()
            .filter_map(|word| {
                let normalized: String = word
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase();

                if normalized.is_empty() {
                    None
                } else {
                    Some(Value::String(normalized))
                }
            })
            .collect();

        Ok(Value::Array(tokens))
    }
}

// =============================================================================
// tokenize(s, options?) -> array
// Configurable tokenization with options:
//   - case: "lower" (default), "upper", "preserve"
//   - punctuation: "strip" (default), "keep"
// =============================================================================

defn!(TokenizeFn, vec![arg!(string)], Some(arg!(object)));

impl Function for TokenizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        // Parse options
        let (case_mode, strip_punctuation) = if args.len() > 1 {
            if let Some(opts) = args[1].as_object() {
                let case_mode = opts.get("case").and_then(|v| v.as_str()).unwrap_or("lower");

                let punctuation = opts
                    .get("punctuation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("strip");

                (case_mode.to_string(), punctuation != "keep")
            } else {
                ("lower".to_string(), true)
            }
        } else {
            ("lower".to_string(), true)
        };

        let tokens: Vec<Value> = s
            .split_whitespace()
            .filter_map(|word| {
                let processed: String = if strip_punctuation {
                    word.chars().filter(|c| c.is_alphanumeric()).collect()
                } else {
                    word.to_string()
                };

                if processed.is_empty() {
                    return None;
                }

                let final_token = match case_mode.as_str() {
                    "upper" => processed.to_uppercase(),
                    "preserve" => processed,
                    _ => processed.to_lowercase(), // "lower" or default
                };

                Some(Value::String(final_token))
            })
            .collect();

        Ok(Value::Array(tokens))
    }
}

// =============================================================================
// stem(word, lang?) -> string
// Stem a single word using Porter/Snowball stemmer
// =============================================================================

defn!(StemFn, vec![arg!(string)], Some(arg!(string)));

impl Function for StemFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rust_stemmers::{Algorithm, Stemmer};

        self.signature.validate(args, ctx)?;
        let word = args[0].as_str().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let algorithm = match lang.as_deref() {
            Some("ar" | "arabic") => Algorithm::Arabic,
            Some("da" | "danish") => Algorithm::Danish,
            Some("nl" | "dutch") => Algorithm::Dutch,
            Some("fi" | "finnish") => Algorithm::Finnish,
            Some("fr" | "french") => Algorithm::French,
            Some("de" | "german") => Algorithm::German,
            Some("el" | "greek") => Algorithm::Greek,
            Some("hu" | "hungarian") => Algorithm::Hungarian,
            Some("it" | "italian") => Algorithm::Italian,
            Some("no" | "norwegian") => Algorithm::Norwegian,
            Some("pt" | "portuguese") => Algorithm::Portuguese,
            Some("ro" | "romanian") => Algorithm::Romanian,
            Some("ru" | "russian") => Algorithm::Russian,
            Some("es" | "spanish") => Algorithm::Spanish,
            Some("sv" | "swedish") => Algorithm::Swedish,
            Some("ta" | "tamil") => Algorithm::Tamil,
            Some("tr" | "turkish") => Algorithm::Turkish,
            _ => Algorithm::English, // Default to English
        };

        let stemmer = Stemmer::create(algorithm);
        let stemmed = stemmer.stem(word).to_string();

        Ok(Value::String(stemmed))
    }
}

// =============================================================================
// stems(tokens, lang?) -> array
// Stem an array of tokens
// =============================================================================

defn!(StemsFn, vec![arg!(array)], Some(arg!(string)));

impl Function for StemsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rust_stemmers::{Algorithm, Stemmer};

        self.signature.validate(args, ctx)?;
        let tokens = args[0].as_array().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let algorithm = match lang.as_deref() {
            Some("ar" | "arabic") => Algorithm::Arabic,
            Some("da" | "danish") => Algorithm::Danish,
            Some("nl" | "dutch") => Algorithm::Dutch,
            Some("fi" | "finnish") => Algorithm::Finnish,
            Some("fr" | "french") => Algorithm::French,
            Some("de" | "german") => Algorithm::German,
            Some("el" | "greek") => Algorithm::Greek,
            Some("hu" | "hungarian") => Algorithm::Hungarian,
            Some("it" | "italian") => Algorithm::Italian,
            Some("no" | "norwegian") => Algorithm::Norwegian,
            Some("pt" | "portuguese") => Algorithm::Portuguese,
            Some("ro" | "romanian") => Algorithm::Romanian,
            Some("ru" | "russian") => Algorithm::Russian,
            Some("es" | "spanish") => Algorithm::Spanish,
            Some("sv" | "swedish") => Algorithm::Swedish,
            Some("ta" | "tamil") => Algorithm::Tamil,
            Some("tr" | "turkish") => Algorithm::Turkish,
            _ => Algorithm::English,
        };

        let stemmer = Stemmer::create(algorithm);

        let result: Vec<Value> = tokens
            .iter()
            .filter_map(|t| {
                t.as_str()
                    .map(|s| Value::String(stemmer.stem(s).to_string()))
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// stopwords(lang?) -> array
// Get stopwords list for a language
// =============================================================================

defn!(StopwordsFn, vec![], Some(arg!(string)));

impl Function for StopwordsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;

        let lang = if !args.is_empty() {
            args[0].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let words = get(language);
        let result: Vec<Value> = words.iter().map(|w| Value::String(w.to_string())).collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// remove_stopwords(tokens, lang?) -> array
// Remove stopwords from token array
// =============================================================================

defn!(RemoveStopwordsFn, vec![arg!(array)], Some(arg!(string)));

impl Function for RemoveStopwordsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;
        let tokens = args[0].as_array().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let stopwords = get(language);
        let stopwords_set: HashSet<String> = stopwords.iter().map(|s| s.to_string()).collect();

        let result: Vec<Value> = tokens
            .iter()
            .filter_map(|t| {
                t.as_str().and_then(|s| {
                    if stopwords_set.contains(&s.to_lowercase()) {
                        None
                    } else {
                        Some(Value::String(s.to_string()))
                    }
                })
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// is_stopword(word, lang?) -> boolean
// Check if word is a stopword
// =============================================================================

defn!(IsStopwordFn, vec![arg!(string)], Some(arg!(string)));

impl Function for IsStopwordFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;
        let word = args[0].as_str().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let stopwords = get(language);
        let is_stop = stopwords.iter().any(|sw| sw.eq_ignore_ascii_case(word));

        Ok(Value::Bool(is_stop))
    }
}

// =============================================================================
// normalize_unicode(s, form?) -> string
// Unicode normalization (NFC, NFD, NFKC, NFKD)
// =============================================================================

defn!(NormalizeUnicodeFn, vec![arg!(string)], Some(arg!(string)));

impl Function for NormalizeUnicodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use unicode_normalization::UnicodeNormalization;

        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let form = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_uppercase())
        } else {
            None
        };

        let normalized = match form.as_deref() {
            Some("NFD") => s.nfd().collect::<String>(),
            Some("NFKC") => s.nfkc().collect::<String>(),
            Some("NFKD") => s.nfkd().collect::<String>(),
            _ => s.nfc().collect::<String>(), // Default to NFC
        };

        Ok(Value::String(normalized))
    }
}

// =============================================================================
// remove_accents(s) -> string
// Strip diacritics/accents from text
// =============================================================================

defn!(RemoveAccentsFn, vec![arg!(string)], None);

impl Function for RemoveAccentsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use unicode_normalization::UnicodeNormalization;

        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        // NFD normalize then filter out combining marks (diacritics)
        let result: String = s
            .nfd()
            .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
            .collect();

        Ok(Value::String(result))
    }
}

// =============================================================================
// collapse_whitespace(s) -> string
// Normalize whitespace (multiple spaces -> single, trim)
// =============================================================================

defn!(CollapseWhitespaceFn, vec![arg!(string)], None);

impl Function for CollapseWhitespaceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        let result: String = s.split_whitespace().collect::<Vec<_>>().join(" ");

        Ok(Value::String(result))
    }
}

/// Register text functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "word_count", enabled, Box::new(WordCountFn::new()));
    register_if_enabled(runtime, "char_count", enabled, Box::new(CharCountFn::new()));
    register_if_enabled(
        runtime,
        "sentence_count",
        enabled,
        Box::new(SentenceCountFn::new()),
    );
    register_if_enabled(
        runtime,
        "paragraph_count",
        enabled,
        Box::new(ParagraphCountFn::new()),
    );
    register_if_enabled(
        runtime,
        "reading_time",
        enabled,
        Box::new(ReadingTimeFn::new()),
    );
    register_if_enabled(
        runtime,
        "reading_time_seconds",
        enabled,
        Box::new(ReadingTimeSecondsFn::new()),
    );
    register_if_enabled(
        runtime,
        "char_frequencies",
        enabled,
        Box::new(CharFrequenciesFn::new()),
    );
    register_if_enabled(
        runtime,
        "word_frequencies",
        enabled,
        Box::new(WordFrequenciesFn::new()),
    );
    register_if_enabled(runtime, "ngrams", enabled, Box::new(NgramsFn::new()));
    register_if_enabled(runtime, "bigrams", enabled, Box::new(BigramsFn::new()));
    register_if_enabled(runtime, "trigrams", enabled, Box::new(TrigramsFn::new()));
    register_if_enabled(runtime, "tokens", enabled, Box::new(TokensFn::new()));
    register_if_enabled(runtime, "tokenize", enabled, Box::new(TokenizeFn::new()));
    register_if_enabled(runtime, "stem", enabled, Box::new(StemFn::new()));
    register_if_enabled(runtime, "stems", enabled, Box::new(StemsFn::new()));
    register_if_enabled(runtime, "stopwords", enabled, Box::new(StopwordsFn::new()));
    register_if_enabled(
        runtime,
        "remove_stopwords",
        enabled,
        Box::new(RemoveStopwordsFn::new()),
    );
    register_if_enabled(
        runtime,
        "is_stopword",
        enabled,
        Box::new(IsStopwordFn::new()),
    );
    register_if_enabled(
        runtime,
        "normalize_unicode",
        enabled,
        Box::new(NormalizeUnicodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "remove_accents",
        enabled,
        Box::new(RemoveAccentsFn::new()),
    );
    register_if_enabled(
        runtime,
        "collapse_whitespace",
        enabled,
        Box::new(CollapseWhitespaceFn::new()),
    );
}
