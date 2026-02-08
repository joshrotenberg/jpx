//! Network and IP address functions.

use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::str::FromStr;

use ipnetwork::{IpNetwork, Ipv4Network};
use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// ip_to_int(s) -> number
// =============================================================================

defn!(IpToIntFn, vec![arg!(string)], None);

impl Function for IpToIntFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        match Ipv4Addr::from_str(s) {
            Ok(ip) => {
                let int_val: u32 = ip.into();
                Ok(number_value(int_val as f64))
            }
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// int_to_ip(n) -> string
// =============================================================================

defn!(IntToIpFn, vec![arg!(number)], None);

impl Function for IntToIpFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();

        if n < 0.0 || n > u32::MAX as f64 {
            return Ok(Value::Null);
        }

        let ip = Ipv4Addr::from(n as u32);
        Ok(Value::String(ip.to_string()))
    }
}

// =============================================================================
// cidr_contains(cidr, ip) -> bool
// =============================================================================

defn!(CidrContainsFn, vec![arg!(string), arg!(string)], None);

impl Function for CidrContainsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_str().unwrap();
        let ip_str = args[1].as_str().unwrap();

        let network = match IpNetwork::from_str(cidr_str) {
            Ok(n) => n,
            Err(_) => return Ok(Value::Null),
        };

        let ip: std::net::IpAddr = match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => return Ok(Value::Null),
        };

        Ok(Value::Bool(network.contains(ip)))
    }
}

// =============================================================================
// cidr_network(cidr) -> string
// =============================================================================

defn!(CidrNetworkFn, vec![arg!(string)], None);

impl Function for CidrNetworkFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_str().unwrap();

        match Ipv4Network::from_str(cidr_str) {
            Ok(network) => Ok(Value::String(network.network().to_string())),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// cidr_broadcast(cidr) -> string
// =============================================================================

defn!(CidrBroadcastFn, vec![arg!(string)], None);

impl Function for CidrBroadcastFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_str().unwrap();

        match Ipv4Network::from_str(cidr_str) {
            Ok(network) => Ok(Value::String(network.broadcast().to_string())),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// cidr_prefix(cidr) -> number
// =============================================================================

defn!(CidrPrefixFn, vec![arg!(string)], None);

impl Function for CidrPrefixFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let cidr_str = args[0].as_str().unwrap();

        match IpNetwork::from_str(cidr_str) {
            Ok(network) => Ok(Value::Number(Number::from(network.prefix()))),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// is_private_ip(ip) -> bool
// =============================================================================

defn!(IsPrivateIpFn, vec![arg!(string)], None);

impl Function for IsPrivateIpFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let ip_str = args[0].as_str().unwrap();

        match Ipv4Addr::from_str(ip_str) {
            Ok(ip) => Ok(Value::Bool(ip.is_private())),
            Err(_) => Ok(Value::Null),
        }
    }
}

/// Register network functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "ip_to_int", enabled, Box::new(IpToIntFn::new()));
    register_if_enabled(runtime, "int_to_ip", enabled, Box::new(IntToIpFn::new()));
    register_if_enabled(
        runtime,
        "cidr_contains",
        enabled,
        Box::new(CidrContainsFn::new()),
    );
    register_if_enabled(
        runtime,
        "cidr_network",
        enabled,
        Box::new(CidrNetworkFn::new()),
    );
    register_if_enabled(
        runtime,
        "cidr_broadcast",
        enabled,
        Box::new(CidrBroadcastFn::new()),
    );
    register_if_enabled(
        runtime,
        "cidr_prefix",
        enabled,
        Box::new(CidrPrefixFn::new()),
    );
    register_if_enabled(
        runtime,
        "is_private_ip",
        enabled,
        Box::new(IsPrivateIpFn::new()),
    );
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_ip_to_int() {
        let runtime = setup_runtime();
        let data = json!("192.168.1.1");
        let expr = runtime.compile("ip_to_int(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // 192.168.1.1 = 192*256^3 + 168*256^2 + 1*256 + 1 = 3232235777
        assert_eq!(result.as_f64().unwrap(), 3232235777.0);
    }

    #[test]
    fn test_int_to_ip() {
        let runtime = setup_runtime();
        let data = json!(3232235777_u64);
        let expr = runtime.compile("int_to_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_ip_roundtrip() {
        let runtime = setup_runtime();
        let data = json!("10.0.0.1");
        let expr = runtime.compile("int_to_ip(ip_to_int(@))").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "10.0.0.1");
    }

    #[test]
    fn test_cidr_contains_true() {
        let runtime = setup_runtime();
        let data = json!({"cidr": "192.168.1.0/24", "ip": "192.168.1.100"});
        let expr = runtime.compile("cidr_contains(cidr, ip)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_cidr_contains_false() {
        let runtime = setup_runtime();
        let data = json!({"cidr": "192.168.1.0/24", "ip": "192.168.2.1"});
        let expr = runtime.compile("cidr_contains(cidr, ip)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_cidr_network() {
        let runtime = setup_runtime();
        let data = json!("192.168.1.100/24");
        let expr = runtime.compile("cidr_network(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.0");
    }

    #[test]
    fn test_cidr_broadcast() {
        let runtime = setup_runtime();
        let data = json!("192.168.1.0/24");
        let expr = runtime.compile("cidr_broadcast(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "192.168.1.255");
    }

    #[test]
    fn test_cidr_prefix() {
        let runtime = setup_runtime();
        let data = json!("10.0.0.0/8");
        let expr = runtime.compile("cidr_prefix(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 8.0);
    }

    #[test]
    fn test_is_private_ip_true() {
        let runtime = setup_runtime();
        // 192.168.x.x is private
        let data = json!("192.168.1.1");
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_private_ip_10() {
        let runtime = setup_runtime();
        // 10.x.x.x is private
        let data = json!("10.0.0.1");
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_private_ip_false() {
        let runtime = setup_runtime();
        // 8.8.8.8 is public (Google DNS)
        let data = json!("8.8.8.8");
        let expr = runtime.compile("is_private_ip(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }
}
