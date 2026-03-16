# Phonetic Functions (9)

## `caverphone`

**Signature:** `string -> string`

Caverphone code

```
caverphone('Smith') -> \"SMT1111111\"
```
_Common name_

```
caverphone('Thompson') -> \"TMPSN11111\"
```
_Another name_

---

## `caverphone2`

**Signature:** `string -> string`

Caverphone 2 code

```
caverphone2('Smith') -> \"SMT1111111\"
```
_Common name_

```
caverphone2('Thompson') -> \"TMPSN11111\"
```
_Another name_

---

## `double_metaphone`

**Signature:** `string -> object`

Double Metaphone codes

```
double_metaphone('Smith') -> {primary: 'SM0', secondary: 'XMT'}
```
_Common name_

```
double_metaphone('Schmidt') -> {primary: 'XMT', secondary: 'SMT'}
```
_German variant_

---

## `match_rating_codex`

**Signature:** `string -> string`

Match Rating codex

```
match_rating_codex('Smith') -> \"SMTH\"
```
_Common name_

```
match_rating_codex('Johnson') -> \"JHNSN\"
```
_Another name_

---

## `metaphone`

**Signature:** `string -> string`

Metaphone phonetic code

```
metaphone('Smith') -> \"SM0\"
```
_Common name_

```
metaphone('phone') -> \"FN\"
```
_Ph sound_

---

## `nysiis`

**Signature:** `string -> string`

NYSIIS phonetic code

```
nysiis('Smith') -> \"SNAT\"
```
_Common name_

```
nysiis('Johnson') -> \"JANSAN\"
```
_Another name_

---

## `phonetic_match`

**Signature:** `string, string, string -> boolean`

Check phonetic match with algorithm

```
phonetic_match('Smith', 'Smyth', 'soundex') -> true
```
_Soundex match_

```
phonetic_match('Robert', 'Rupert', 'metaphone') -> true
```
_Metaphone match_

---

## `soundex`

**Signature:** `string -> string`

Soundex phonetic code

```
soundex('Robert') -> \"R163\"
```
_Common name_

```
soundex('Rupert') -> \"R163\"
```
_Same code as Robert_

---

## `sounds_like`

**Signature:** `string, string -> boolean`

Check if strings sound similar

```
sounds_like('Robert', 'Rupert') -> true
```
_Similar sounding_

```
sounds_like('Smith', 'Smyth') -> true
```
_Spelling variants_

---

