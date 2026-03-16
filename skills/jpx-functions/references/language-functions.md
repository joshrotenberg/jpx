# Language Functions (5)

## `detect_language`

**Signature:** `string -> string | null`

Detect the language of text, returning the native language name

```
detect_language('This is English text.') -> \"English\"
```
_Detect English_

```
detect_language('Esto es texto en español.') -> \"Español\"
```
_Detect Spanish (native name)_

---

## `detect_language_confidence`

**Signature:** `string -> number | null`

Detect language and return confidence score (0.0-1.0)

```
detect_language_confidence('This is definitely English text.') -> 0.95
```
_High confidence_

```
detect_language_confidence('Hi') -> 0.3
```
_Low confidence for short text_

---

## `detect_language_info`

**Signature:** `string -> object | null`

Detect language and return full detection info object

```
detect_language_info('This is a test.') -> {language: 'English', code: 'eng', script: 'Latin', confidence: 0.9, reliable: true}
```
_Full detection info_

```
detect_language_info('Bonjour').language -> 'French'
```
_Access language field_

---

## `detect_language_iso`

**Signature:** `string -> string | null`

Detect the language of text, returning the ISO 639-3 code

```
detect_language_iso('This is English text.') -> \"eng\"
```
_English ISO code_

```
detect_language_iso('Esto es texto en español.') -> \"spa\"
```
_Spanish ISO code_

---

## `detect_script`

**Signature:** `string -> string | null`

Detect the script (writing system) of text

```
detect_script('Hello world') -> \"Latin\"
```
_Latin script_

```
detect_script('Привет мир') -> \"Cyrillic\"
```
_Cyrillic script_

---

