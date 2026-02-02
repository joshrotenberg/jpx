# Language Functions

Natural language processing functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`detect_language`](#detect-language) | `string -> string \| null` | Detect the language of text, returning the native language name |
| [`detect_language_confidence`](#detect-language-confidence) | `string -> number \| null` | Detect language and return confidence score (0.0-1.0) |
| [`detect_language_info`](#detect-language-info) | `string -> object \| null` | Detect language and return full detection info object |
| [`detect_language_iso`](#detect-language-iso) | `string -> string \| null` | Detect the language of text, returning the ISO 639-3 code |
| [`detect_script`](#detect-script) | `string -> string \| null` | Detect the script (writing system) of text |

## Functions

### detect_language

Detect the language of text, returning the native language name

**Signature:** `string -> string | null`

**Examples:**

```text
# Detect English
detect_language('This is English text.') -> \"English\"
# Detect Spanish (native name)
detect_language('Esto es texto en español.') -> \"Español\"
# Detect French (native name)
detect_language('Ceci est du texte français.') -> \"Français\"
# Empty or undetectable returns null
detect_language('') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'detect_language(`"This is English text."`)'
```

### detect_language_confidence

Detect language and return confidence score (0.0-1.0)

**Signature:** `string -> number | null`

**Examples:**

```text
# High confidence
detect_language_confidence('This is definitely English text.') -> 0.95
# Low confidence for short text
detect_language_confidence('Hi') -> 0.3
# Empty returns null
detect_language_confidence('') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'detect_language_confidence(`"This is definitely English text."`)'
```

### detect_language_info

Detect language and return full detection info object

**Signature:** `string -> object | null`

**Examples:**

```text
# Full detection info
detect_language_info('This is a test.') -> {language: 'English', code: 'eng', script: 'Latin', confidence: 0.9, reliable: true}
# Access language field
detect_language_info('Bonjour').language -> 'French'
# Check reliability
detect_language_info('Hola').reliable -> true/false
# Empty returns null
detect_language_info('') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'detect_language_info(`"This is a test."`)'
```

### detect_language_iso

Detect the language of text, returning the ISO 639-3 code

**Signature:** `string -> string | null`

**Examples:**

```text
# English ISO code
detect_language_iso('This is English text.') -> \"eng\"
# Spanish ISO code
detect_language_iso('Esto es texto en español.') -> \"spa\"
# German ISO code
detect_language_iso('Dies ist deutscher Text.') -> \"deu\"
# Empty or undetectable returns null
detect_language_iso('') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'detect_language_iso(`"This is English text."`)'
```

### detect_script

Detect the script (writing system) of text

**Signature:** `string -> string | null`

**Examples:**

```text
# Latin script
detect_script('Hello world') -> \"Latin\"
# Cyrillic script
detect_script('Привет мир') -> \"Cyrillic\"
# Arabic script
detect_script('مرحبا بالعالم') -> \"Arabic\"
# Japanese Hiragana
detect_script('こんにちは') -> \"Hiragana\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'detect_script(`"Hello world"`)'
```

