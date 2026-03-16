# Duration Functions (8)

## `duration_add`

**Signature:** `string, string -> string`

Add two duration strings

```
duration_add('1h', '30m') -> \"1h30m\"
```
_Add 1 hour and 30 minutes_

```
duration_add('1d', '12h') -> \"1d12h\"
```
_Add 1 day and 12 hours_

---

## `duration_days`

**Signature:** `number -> number`

Get days component from seconds

```
duration_days(`86400`) -> 1
```
_1 day_

```
duration_days(`90061`) -> 1
```
_1 day 1 hour 1 min 1 sec_

---

## `duration_hours`

**Signature:** `number -> number`

Convert seconds to hours

```
duration_hours(`7200`) -> 2
```
_2 hours_

```
duration_hours(`3600`) -> 1
```
_1 hour_

---

## `duration_minutes`

**Signature:** `number -> number`

Convert seconds to minutes

```
duration_minutes(`120`) -> 2
```
_2 minutes_

```
duration_minutes(`60`) -> 1
```
_1 minute_

---

## `duration_seconds`

**Signature:** `number -> number`

Get seconds component

```
duration_seconds(`65`) -> 5
```
_65 seconds mod 60_

```
duration_seconds(`120`) -> 0
```
_Exact minutes_

---

## `duration_subtract`

**Signature:** `string, string -> string`

Subtract second duration string from first

```
duration_subtract('2h', '30m') -> \"1h30m\"
```
_Subtract 30 minutes from 2 hours_

```
duration_subtract('1h', '1h') -> \"0s\"
```
_Equal durations_

---

## `format_duration`

**Signature:** `number -> string`

Format seconds as duration string

```
format_duration(`5400`) -> \"1h30m\"
```
_1.5 hours_

```
format_duration(`3661`) -> \"1h1m1s\"
```
_1 hour 1 min 1 sec_

---

## `parse_duration`

**Signature:** `string -> number`

Parse duration string to seconds

```
parse_duration('1h30m') -> 5400
```
_1.5 hours_

```
parse_duration('2h') -> 7200
```
_2 hours_

---

