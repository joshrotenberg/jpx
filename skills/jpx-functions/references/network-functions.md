# Network Functions (7)

## `cidr_broadcast`

**Signature:** `string -> string`

Get broadcast address from CIDR

```
cidr_broadcast('192.168.1.0/24') -> \"192.168.1.255\"
```
_/24 network_

```
cidr_broadcast('10.0.0.0/8') -> \"10.255.255.255\"
```
_/8 network_

---

## `cidr_contains`

**Signature:** `string, string -> boolean`

Check if IP is in CIDR range

```
cidr_contains('192.168.0.0/16', '192.168.1.1') -> true
```
_IP in range_

```
cidr_contains('10.0.0.0/8', '192.168.1.1') -> false
```
_IP not in range_

---

## `cidr_network`

**Signature:** `string -> string`

Get network address from CIDR

```
cidr_network('192.168.1.0/24') -> \"192.168.1.0\"
```
_/24 network_

```
cidr_network('10.0.0.0/8') -> \"10.0.0.0\"
```
_/8 network_

---

## `cidr_prefix`

**Signature:** `string -> number`

Get prefix length from CIDR

```
cidr_prefix('192.168.1.0/24') -> 24
```
_/24 prefix_

```
cidr_prefix('10.0.0.0/8') -> 8
```
_/8 prefix_

---

## `int_to_ip`

**Signature:** `number -> string`

Convert integer to IP address

```
int_to_ip(`3232235777`) -> \"192.168.1.1\"
```
_Private IP_

```
int_to_ip(`0`) -> \"0.0.0.0\"
```
_Zero_

---

## `ip_to_int`

**Signature:** `string -> number`

Convert IP address to integer

```
ip_to_int('192.168.1.1') -> 3232235777
```
_Private IP_

```
ip_to_int('0.0.0.0') -> 0
```
_Zero_

---

## `is_private_ip`

**Signature:** `string -> boolean`

Check if IP is in private range

```
is_private_ip('192.168.1.1') -> true
```
_Class C private_

```
is_private_ip('10.0.0.1') -> true
```
_Class A private_

---

