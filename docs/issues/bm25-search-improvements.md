# BM25 Discovery Search Improvements

## Summary

Testing jpx's tool discovery against a real MCP server (redisctl with 65 tools) revealed several areas for improvement in the BM25 search implementation. The search works well for basic keyword matching but has issues with stop words, stemming, and semantic search.

**Overall Score: B+ (Good, with clear improvement paths)**

| Category | Score | Notes |
|----------|-------|-------|
| Basic keyword search | A | Excellent precision and recall |
| Multi-word phrase search | A | Good scoring, finds relevant matches |
| Domain-specific terms | A | ACL, CRDB, BDB work perfectly |
| Semantic/conceptual queries | C | Limited - no synonym expansion |
| Similar tools discovery | B- | Works but noisy due to stop words |

## Issues to Address

### 1. Stop Words Not Filtered (High Priority)

**Problem:** Common English words are indexed and affect relevance scoring significantly.

**Evidence from `inspect_discovery_index` top terms:**
| Term | Document Frequency |
|------|-------------------|
| redis | 52 |
| enterprise | 42 |
| **a** | **39** |
| get | 25 |
| cluster | 23 |
| **the** | **23** |
| **in** | **18** |
| **and** | **11** |

**Impact:**
- `similar_tools("redisctl:enterprise_database_backup")` returns matches including "a", "of" as significant terms
- Unrelated tools get inflated scores from common words
- Search precision is reduced

**Recommended Fix:**

Add stop word filtering to `Bm25Index::build()` in `jpx/src/mcp/bm25.rs`:

```rust
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", 
    "of", "with", "by", "from", "as", "is", "was", "are", "were", "been", 
    "be", "have", "has", "had", "do", "does", "did", "will", "would", 
    "could", "should", "may", "might", "must", "shall", "can", "this", 
    "that", "these", "those", "it", "its"
];

fn is_stop_word(term: &str) -> bool {
    STOP_WORDS.contains(&term.to_lowercase().as_str())
}
```

Filter during tokenization in `tokenize()` function.

**Alternative:** Weight stop words very low (0.1x) rather than removing entirely.

---

### 2. Plural Forms Not Matching Singular (Medium Priority)

**Problem:** "ACL" doesn't match "ACLs", "shard" doesn't match "shards".

**Evidence:**
- Query `ACL` found 3 tools but missed `enterprise_acls_list`
- Query `shard` found 1 tool but missed `enterprise_shards_list`
- Query `CRDB` correctly found `enterprise_crdbs_list` (inconsistent)

**Recommended Fix:**

Implement basic stemming during tokenization. Options:

1. **Simple plural stripping** (quick fix):
```rust
fn stem_simple(term: &str) -> String {
    let t = term.to_lowercase();
    if t.ends_with("ies") {
        format!("{}y", &t[..t.len()-3])
    } else if t.ends_with("es") && t.len() > 3 {
        t[..t.len()-2].to_string()
    } else if t.ends_with('s') && t.len() > 2 {
        t[..t.len()-1].to_string()
    } else {
        t
    }
}
```

2. **Porter Stemmer** (better quality):
Add `rust-stemmers` crate and use Porter algorithm.

---

### 3. Tags Support in Discovery Schema (Medium Priority)

**Problem:** No way to add semantic metadata to tools for conceptual search.

**Evidence:**
- `security access control permissions` returned empty despite ACL tools existing
- `data protection disaster recovery` missed backup/restore tools
- Terms in descriptions are too literal

**Recommended Fix:**

The schema already supports tags - ensure they're indexed:

```json
{
  "name": "enterprise_acl_create",
  "description": "Create a new Redis ACL",
  "tags": ["security", "access-control", "permissions", "auth", "rbac"]
}
```

Verify `ToolSpec.tags` are being included in the BM25 document:

```rust
// In discovery.rs rebuild_index()
let docs: Vec<Value> = self.tools.iter().map(|(id, (server, tool))| {
    serde_json::json!({
        "id": id,
        "server": server,
        "name": tool.name,
        "tags": tool.tags.join(" "),  // <-- Ensure this is indexed
        // ...
    })
}).collect();
```

---

### 4. Parameter Names Not Indexed (Low Priority)

**Problem:** Cannot search by parameter names.

**Evidence:** Searching for `database_id` or `subscription_id` doesn't find relevant tools.

**Recommended Fix:**

Add parameter names to indexed content:

```rust
"params": tool.params.iter()
    .map(|p| format!("{} {}", p.name, p.description.as_deref().unwrap_or("")))
    .collect::<Vec<_>>()
    .join(" "),
```

---

## Enhancement Ideas (Future)

### Weighted Field Search
Different weights for name vs description vs tags:
```rust
IndexOptions {
    field_weights: vec![
        ("name", 3.0),
        ("tags", 2.0),
        ("description", 1.0),
        ("params", 0.5),
    ],
    // ...
}
```

### Fuzzy Matching
Add optional typo tolerance:
- `databse` → matches `database`
- `bakcup` → matches `backup`

Could use Levenshtein distance or n-gram matching.

### Prefix/Wildcard Search
Support glob patterns:
- `enterprise_*` → all enterprise tools
- `*_list` → all list tools

### Negative Terms
Support excluding terms:
- `database -delete` → database tools except delete

### Filter by Server
When multiple servers registered:
```json
{
  "query": "backup",
  "server": "redisctl",
  "top_k": 5
}
```

---

## Test Cases to Add

```rust
#[test]
fn test_stop_words_filtered() {
    let mut registry = DiscoveryRegistry::new();
    registry.register(spec_with_description("Tool a]for the database"), false);
    
    let stats = registry.index_stats().unwrap();
    let top_terms: Vec<_> = stats.top_terms.iter().map(|(t, _)| t.as_str()).collect();
    
    assert!(!top_terms.contains(&"a"));
    assert!(!top_terms.contains(&"the"));
    assert!(!top_terms.contains(&"for"));
    assert!(top_terms.contains(&"database"));
}

#[test]
fn test_plural_stemming() {
    let mut registry = DiscoveryRegistry::new();
    registry.register(spec_with_tools(vec![
        ("list_databases", "List all databases"),
        ("get_database", "Get a database"),
    ]), false);
    
    // Singular should match plural
    let results = registry.query("database", 10);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_tags_indexed() {
    let mut registry = DiscoveryRegistry::new();
    registry.register(spec_with_tags("acl_create", vec!["security", "permissions"]), false);
    
    // Should find by tag even if not in description
    let results = registry.query("security", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].tool.name, "acl_create");
}
```

---

## Priority Order

1. **Stop word filtering** - Quick win, biggest impact on search quality
2. **Basic plural stemming** - Fixes real gaps in search results
3. **Verify tags are indexed** - Enables semantic search without NLP
4. **Index parameter names** - Nice to have for detailed searches

---

## References

- Full test report: `redisctl/jpx-tool-discovery-report.md`
- BM25 implementation: `jpx/src/mcp/bm25.rs`
- Discovery registry: `jpx/src/mcp/discovery.rs`
- Integration tests: `jpx/tests/mock_server_integration.rs`
