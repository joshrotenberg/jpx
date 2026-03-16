# Url Functions (6)

## `query_string_build`

**Signature:** `object -> string`

Build a URL query string from an object

```
query_string_build({foo: 'bar', baz: 'qux'}) -> "foo=bar&baz=qux"
```
_Basic query string_

```
query_string_build({q: 'hello world'}) -> "q=hello+world"
```
_With special characters_

---

## `query_string_parse`

**Signature:** `string -> object`

Parse a URL query string into an object

```
query_string_parse('foo=bar&baz=qux') -> {foo: 'bar', baz: 'qux'}
```
_Basic parsing_

```
query_string_parse('greeting=hello%20world') -> {greeting: 'hello world'}
```
_Encoded values_

---

## `url_build`

**Signature:** `object -> string`

Build a URL from component parts

```
url_build({scheme: 'https', host: 'example.com'}) -> "https://example.com/"
```
_Minimal URL_

```
url_build({scheme: 'https', host: 'example.com', port: 8080, path: '/api'}) -> full URL
```
_With port and path_

---

## `url_decode`

**Signature:** `string -> string`

URL decode a string

```
url_decode('hello%20world') -> \"hello world\"
```
_Decode space_

```
url_decode('a%2Bb') -> \"a+b\"
```
_Decode plus sign_

---

## `url_encode`

**Signature:** `string -> string`

URL encode a string

```
url_encode('hello world') -> \"hello%20world\"
```
_Encode space_

```
url_encode('a+b') -> \"a%2Bb\"
```
_Encode plus_

---

## `url_parse`

**Signature:** `string -> object`

Parse URL into components

```
url_parse('https://example.com/path') -> {scheme: 'https', ...}
```
_Parse full URL_

```
url_parse('http://user:pass@host:8080') -> components
```
_With auth and port_

---

