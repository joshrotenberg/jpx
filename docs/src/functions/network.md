# Network Functions

Network-related functions: IP addresses, CIDR notation, and network utilities.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`cidr_broadcast`](#cidr-broadcast) | `string -> string` | Get broadcast address from CIDR |
| [`cidr_contains`](#cidr-contains) | `string, string -> boolean` | Check if IP is in CIDR range |
| [`cidr_network`](#cidr-network) | `string -> string` | Get network address from CIDR |
| [`cidr_prefix`](#cidr-prefix) | `string -> number` | Get prefix length from CIDR |
| [`int_to_ip`](#int-to-ip) | `number -> string` | Convert integer to IP address |
| [`ip_to_int`](#ip-to-int) | `string -> number` | Convert IP address to integer |
| [`is_private_ip`](#is-private-ip) | `string -> boolean` | Check if IP is in private range |

## Functions

### cidr_broadcast

Get broadcast address from CIDR

**Signature:** `string -> string`

**Examples:**

```text
# /24 network
cidr_broadcast('192.168.1.0/24') -> \"192.168.1.255\"
# /8 network
cidr_broadcast('10.0.0.0/8') -> \"10.255.255.255\"
# /16 network
cidr_broadcast('172.16.0.0/16') -> \"172.16.255.255\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cidr_broadcast(`"192.168.1.0/24"`)'
```

### cidr_contains

Check if IP is in CIDR range

**Signature:** `string, string -> boolean`

**Examples:**

```text
# IP in range
cidr_contains('192.168.0.0/16', '192.168.1.1') -> true
# IP not in range
cidr_contains('10.0.0.0/8', '192.168.1.1') -> false
# Catch-all range
cidr_contains('0.0.0.0/0', '8.8.8.8') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cidr_contains(`"192.168.0.0/16"`, `"192.168.1.1"`)'
```

### cidr_network

Get network address from CIDR

**Signature:** `string -> string`

**Examples:**

```text
# /24 network
cidr_network('192.168.1.0/24') -> \"192.168.1.0\"
# /8 network
cidr_network('10.0.0.0/8') -> \"10.0.0.0\"
# Host in network
cidr_network('192.168.1.100/24') -> \"192.168.1.0\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cidr_network(`"192.168.1.0/24"`)'
```

### cidr_prefix

Get prefix length from CIDR

**Signature:** `string -> number`

**Examples:**

```text
# /24 prefix
cidr_prefix('192.168.1.0/24') -> 24
# /8 prefix
cidr_prefix('10.0.0.0/8') -> 8
# Default route
cidr_prefix('0.0.0.0/0') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cidr_prefix(`"192.168.1.0/24"`)'
```

### int_to_ip

Convert integer to IP address

**Signature:** `number -> string`

**Examples:**

```text
# Private IP
int_to_ip(`3232235777`) -> \"192.168.1.1\"
# Zero
int_to_ip(`0`) -> \"0.0.0.0\"
# Max value
int_to_ip(`4294967295`) -> \"255.255.255.255\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'int_to_ip(`3232235777`)'
```

### ip_to_int

Convert IP address to integer

**Signature:** `string -> number`

**Examples:**

```text
# Private IP
ip_to_int('192.168.1.1') -> 3232235777
# Zero
ip_to_int('0.0.0.0') -> 0
# Max value
ip_to_int('255.255.255.255') -> 4294967295
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ip_to_int(`"192.168.1.1"`)'
```

### is_private_ip

Check if IP is in private range

**Signature:** `string -> boolean`

**Examples:**

```text
# Class C private
is_private_ip('192.168.1.1') -> true
# Class A private
is_private_ip('10.0.0.1') -> true
# Public IP
is_private_ip('8.8.8.8') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_private_ip(`"192.168.1.1"`)'
```

