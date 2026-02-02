# Hacker News

The [Hacker News Algolia API](https://hn.algolia.com/api) provides searchable access to HN stories, comments, and discussions. This dataset is excellent for NLP analysis because it has:

- Rich technical text (story titles, Ask HN posts, comments)
- Metadata for filtering (points, comments, timestamps)
- Real developer discussions with domain-specific vocabulary
- Multilingual content (occasional non-English posts)

## Getting the Data

```bash
# Front page stories
curl -s "https://hn.algolia.com/api/v1/search?tags=front_page&hitsPerPage=50" > hn_front.json

# Ask HN posts (rich text content)
curl -s "https://hn.algolia.com/api/v1/search?tags=ask_hn&hitsPerPage=50" > hn_ask.json

# Show HN posts
curl -s "https://hn.algolia.com/api/v1/search?tags=show_hn&hitsPerPage=50" > hn_show.json

# Search for specific topics
curl -s "https://hn.algolia.com/api/v1/search?query=rust&tags=story&hitsPerPage=50" > hn_rust.json

# Comments on a story
curl -s "https://hn.algolia.com/api/v1/search?tags=comment,story_12345&hitsPerPage=100" > comments.json
```

## Data Structure

The search API returns:

```json
{
  "hits": [
    {
      "objectID": "37392676",
      "title": "Ask HN: I'm an FCC Commissioner proposing regulation of IoT security",
      "url": null,
      "author": "SimingtonFCC",
      "points": 2847,
      "story_text": "Hi everyone, I'm FCC Commissioner Nathan Simington...",
      "num_comments": 475,
      "created_at": "2023-09-05T15:00:00.000Z",
      "created_at_i": 1693926000,
      "_tags": ["story", "author_SimingtonFCC", "story_37392676", "ask_hn"]
    }
  ],
  "nbHits": 12345,
  "page": 0,
  "nbPages": 50,
  "hitsPerPage": 50
}
```

Key fields:
- `title` - Story headline (always present)
- `story_text` - Full text for Ask HN / Show HN posts (HTML)
- `url` - External link (null for Ask HN)
- `points` - Upvotes
- `num_comments` - Discussion size
- `_tags` - Includes `ask_hn`, `show_hn`, `front_page`, etc.

---

## Basic Queries

### List Story Titles

```bash
jpx 'hits[*].title' hn_front.json
```

### Get Top Stories with Metadata

```bash
jpx 'hits[*].{
  title: title,
  points: points,
  comments: num_comments,
  author: author
}' hn_front.json
```

### Filter by Points

```bash
jpx 'hits[?points > `500`].title' hn_front.json
```

### Sort by Discussion Size

```bash
jpx 'sort_by(hits, &num_comments) | reverse(@) | [:10].{
  title: title,
  comments: num_comments
}' hn_front.json
```

---

## NLP Analysis on Titles

### Tokenize and Analyze Titles

```bash
# Get word frequencies across all titles
jpx 'hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | frequencies(@)' hn_front.json
```

### Extract Keywords from Top Stories

```bash
jpx 'hits[?points > `200`].title 
  | join(` `, @) 
  | tokens(@) 
  | remove_stopwords(@) 
  | stems(@) 
  | frequencies(@)' hn_front.json
```

### Find Common Technical Terms

```bash
# Stem and count to normalize variations (Rust/rust, APIs/API)
jpx 'hits[*].title 
  | join(` `, @) 
  | tokens(@) 
  | stems(@) 
  | frequencies(@) 
  | items(@) 
  | sort_by(@, &[1]) 
  | reverse(@) 
  | [:20]' hn_front.json
```

### Bigram Analysis on Headlines

```bash
jpx 'hits[*].title | join(` `, @) | bigrams(@) | frequencies(@)' hn_front.json
```

---

## Analyzing Ask HN Posts

Ask HN posts have rich `story_text` content - perfect for NLP.

### Extract and Clean Story Text

```bash
# Remove HTML tags and normalize
jpx 'hits[0].story_text 
  | regex_replace(@, `<[^>]+>`, ` `) 
  | collapse_whitespace(@)' hn_ask.json
```

### Full Text Analysis Pipeline

```bash
jpx 'hits[0].story_text 
  | regex_replace(@, `<[^>]+>`, ` `)
  | tokens(@) 
  | remove_stopwords(@) 
  | stems(@) 
  | frequencies(@)
  | items(@)
  | sort_by(@, &[1])
  | reverse(@)
  | [:15]' hn_ask.json
```

### Compare Vocabulary Across Posts

```bash
# Extract top keywords per Ask HN post
jpx 'hits[:5] | [*].{
  title: title,
  keywords: story_text 
    | regex_replace(@, `<[^>]+>`, ` `) 
    | tokens(@) 
    | remove_stopwords(@) 
    | stems(@) 
    | frequencies(@) 
    | items(@) 
    | sort_by(@, &[1]) 
    | reverse(@) 
    | [:5][*][0]
}' hn_ask.json
```

### Reading Time Estimates

```bash
jpx 'hits[:10] | [*].{
  title: title,
  reading_time: story_text | regex_replace(@, `<[^>]+>`, ` `) | reading_time(@),
  word_count: story_text | regex_replace(@, `<[^>]+>`, ` `) | word_count(@)
}' hn_ask.json
```

---

## Topic Detection

### Categorize by Keywords

```bash
# Find AI/ML related posts
jpx 'hits[?contains(lower(title), `ai`) || 
        contains(lower(title), `llm`) || 
        contains(lower(title), `gpt`) ||
        contains(lower(title), `machine learning`)].{
  title: title,
  points: points
}' hn_front.json
```

### Programming Language Mentions

```bash
jpx 'hits[*].{
  title: title,
  mentions_rust: contains(lower(title), `rust`),
  mentions_go: regex_match(lower(title), `\\bgo\\b`),
  mentions_python: contains(lower(title), `python`)
} | [?mentions_rust || mentions_go || mentions_python]' hn_front.json
```

### N-gram Topic Extraction

```bash
# Find common 2-word phrases (potential topics)
jpx 'hits[*].title 
  | [*] | join(` `, @) 
  | lower(@)
  | ngrams(@, `2`, `word`)
  | frequencies(@)
  | items(@)
  | sort_by(@, &[1])
  | reverse(@)
  | [:10]' hn_front.json
```

---

## Sentiment and Engagement Analysis

### High Engagement Posts

```bash
# Posts with high comment-to-point ratio (controversial?)
jpx 'hits[?points > `100`] | [*].{
  title: title,
  points: points,
  comments: num_comments,
  ratio: divide(num_comments, points)
} | sort_by(@, &ratio) | reverse(@) | [:10]' hn_front.json
```

### Question Posts (Seeking Help)

```bash
jpx 'hits[?ends_with(title, `?`)].{
  title: title,
  comments: num_comments
}' hn_front.json
```

---

## Time-Based Analysis

### Posts by Hour

```bash
jpx 'hits[*].{
  title: title,
  hour: split(created_at, `T`)[1] | split(@, `:`)[0]
} | group_by(@, &hour)' hn_front.json
```

### Recent vs Older Content

```bash
jpx 'hits | {
  recent: [?created_at_i > `1700000000`] | length(@),
  older: [?created_at_i <= `1700000000`] | length(@)
}' hn_front.json
```

---

## Author Analysis

### Most Active Authors

```bash
jpx 'hits[*].author | frequencies(@) | items(@) | sort_by(@, &[1]) | reverse(@) | [:10]' hn_front.json
```

### Author Vocabulary Fingerprint

```bash
# What words does a specific author use most?
jpx 'hits[?author == `dang`].title 
  | join(` `, @) 
  | tokens(@) 
  | remove_stopwords(@) 
  | frequencies(@)' hn_front.json
```

---

## Cross-Dataset Comparisons

### Compare Ask HN vs Show HN Vocabulary

```bash
# Run separately and compare results
jpx 'hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)' hn_ask.json > ask_vocab.json
jpx 'hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)' hn_show.json > show_vocab.json
```

### Search Query Analysis

```bash
# Fetch multiple topics and compare
curl -s "https://hn.algolia.com/api/v1/search?query=kubernetes&hitsPerPage=30" > hn_k8s.json
curl -s "https://hn.algolia.com/api/v1/search?query=docker&hitsPerPage=30" > hn_docker.json

# Compare title vocabulary
jpx 'hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | frequencies(@)' hn_k8s.json
jpx 'hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | frequencies(@)' hn_docker.json
```

---

## Building a Search Index

Use the NLP functions to prepare content for search:

```bash
# Create searchable document representations
jpx 'hits[:20] | [*].{
  id: objectID,
  title: title,
  normalized_title: title | lower(@) | remove_accents(@),
  title_tokens: title | tokens(@) | remove_stopwords(@) | stems(@),
  has_text: story_text != null,
  text_keywords: story_text 
    | default(@, `""`)
    | regex_replace(@, `<[^>]+>`, ` `)
    | tokens(@)
    | remove_stopwords(@)
    | stems(@)
    | unique(@)
    | slice(@, `0`, `20`)
}' hn_ask.json
```

---

## Complete Analysis Pipeline

Here's a comprehensive analysis combining multiple NLP techniques:

```bash
jpx '{
  meta: {
    total_stories: length(hits),
    total_points: sum(hits[*].points),
    avg_comments: avg(hits[*].num_comments)
  },
  top_keywords: hits[*].title 
    | join(` `, @) 
    | tokens(@) 
    | remove_stopwords(@) 
    | stems(@) 
    | frequencies(@)
    | items(@)
    | sort_by(@, &[1])
    | reverse(@)
    | [:10][*][0],
  top_bigrams: hits[*].title
    | join(` `, @)
    | lower(@)
    | bigrams(@)
    | [*] | join(` `, @)
    | frequencies(@)
    | items(@)
    | sort_by(@, &[1])
    | reverse(@)
    | [:5][*][0],
  question_posts: length(hits[?ends_with(title, `?`)]),
  avg_title_words: avg(hits[*].title | [*] | word_count(@))
}' hn_front.json
```

---

## Using Query Libraries

Instead of typing these queries repeatedly, save them in a `.jpx` query library. See [examples/hacker-news.jpx](https://github.com/joshrotenberg/jmespath-extensions/blob/main/examples/hacker-news.jpx) for a ready-to-use library:

```bash
# List available queries
jpx -Q examples/hacker-news.jpx --list-queries

# Run common analyses
jpx -Q examples/hacker-news.jpx:title-keywords hn_front.json
jpx -Q examples/hacker-news.jpx:top-stories hn_front.json
jpx -Q examples/hacker-news.jpx:summary hn_front.json

# Output as table
jpx -Q examples/hacker-news.jpx:most-discussed -t hn_front.json
```

See [Query Files](../../guide/query-files.md) for more on creating and using query libraries.

---

## Tips for HN Data

1. **HTML in story_text**: Always strip HTML tags before NLP processing
   ```bash
   regex_replace(story_text, `<[^>]+>`, ` `)
   ```

2. **Rate limiting**: The Algolia API is generous but cache results locally for iteration

3. **Pagination**: Use `page` parameter for more results
   ```bash
   curl "https://hn.algolia.com/api/v1/search?tags=ask_hn&page=2&hitsPerPage=50"
   ```

4. **Date filtering**: Use `numericFilters` for time ranges
   ```bash
   curl "https://hn.algolia.com/api/v1/search?numericFilters=created_at_i>1700000000"
   ```

5. **Combining with fuzzy search**: Use jpx's `fuzzy_search` on processed results
   ```bash
   jpx 'fuzzy_search(hits, `title`, `database`)' hn_front.json
   ```

---

## Related Functions

- [Text Functions](../functions/text.md) - `tokens`, `stems`, `remove_stopwords`, `word_frequencies`
- [NLP Pipelines](./nlp-text-processing.md) - Complete NLP pipeline examples
- [Regex Functions](../functions/regex.md) - `regex_replace`, `regex_match` for HTML cleaning
- [Fuzzy Functions](../functions/fuzzy.md) - `fuzzy_search` for content discovery
