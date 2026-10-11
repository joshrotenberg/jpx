#![cfg(feature = "extensions")]

use jpx_core::Runtime;
use serde_json::json;

// Fixed UTF-8 vectors cross-checked with Python hashlib/hmac. The Jefe case
// includes the standard RFC 2202 / RFC 4231 HMAC vector; the long key exceeds
// both the 64-byte (MD5/SHA-1/SHA-256) and 128-byte (SHA-512) block sizes.
#[test]
fn hashes_and_hmacs_preserve_known_vectors() {
    let runtime = Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build();
    let cases = [
        (
            "empty",
            "",
            "".to_owned(),
            [
                "d41d8cd98f00b204e9800998ecf8427e",
                "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
                "74e6f7298a9c2d168935f58c001bad88",
                "fbdb1d1b18aa6c08324b7d64b71fb76370690e1d",
                "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad",
                "b936cee86c9f87aa5d3c6f2e84cb5a4239a5fe50480a6ec66b70ab5b1f4ac6730c6c515421b327ec1d69402e53dfb49ad7381eb067b338fd7b0cb22247225d47",
            ],
        ),
        (
            "rfc_hmac",
            "what do ya want for nothing?",
            "Jefe".to_owned(),
            [
                "d03cb659cbf9192dcd066272249f8412",
                "8f820394f95335182045da24f34de52bf8bc3432",
                "b381e7fec653fc3ab9b178272366b8ac87fed8d31cb25ed1d0e1f3318644c89c",
                "a3adca3c9c1e45b39ef51b0703692e09ab3a20404dd7617537144720f35e304e89e530db6db9deb50f8b33e38d951f2ffc973085b7f0ccb11178799d113481f2",
                "750c783e6ab0b503eaa86e310a5db738",
                "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79",
                "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
                "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737",
            ],
        ),
        (
            "unicode",
            "café 🦀\0",
            "clé 🔑".to_owned(),
            [
                "d4bf3feed68f5ed402b6b890a14906c4",
                "f4c8ee45e84a178bb63f7dcf6ff1ae92b7a71fa8",
                "e82a74ee809e4c4f232e1033fd6a320bbfbe2d5c8cea538f32d2873d803851f3",
                "6ec13f3d726225b432636e6664c50a379f81543c0fda42365a90960e7a95e20b7e39360d9eea836aa27109de5e4d1a3a3099506926cd8599b2035fcf2a09edd6",
                "880d107a692c94000b1c4e32ae204596",
                "52b9cdf70f0af2d962525a92e223e46d52f8813f",
                "42cf55c338f0feef891f751e2c4a9c18aead86ce9cb5e626ccd7f00d599d5a10",
                "05dd36732146a1541accf525ecede809dda5841fb14c1659699cb241d8e5c646d2268a23c4f009f23cddfbf250a1a527cf099cadb1ef06e1d9c6b8645cbbb266",
            ],
        ),
        (
            "long_key",
            "hello",
            "a".repeat(200),
            [
                "5d41402abc4b2a76b9719d911017c592",
                "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d",
                "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
                "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043",
                "b0114c7c33233a8265912f373cc1923c",
                "eaa4843b012c5f384df34e631ad092480717be35",
                "9f762a968e88215d22c2119a2d0b644d5148194581798a92bd791ebc384a5f5a",
                "de88491b6ad34731f641d8d26eb771c70d9892cabb6f750a6ce42349bf332082ddcc79eea54f60aaa46b9f0327b56855a0331d5f5898fd89092a18ed182296ca",
            ],
        ),
    ];
    let expressions = [
        "md5(message)",
        "sha1(message)",
        "sha256(message)",
        "sha512(message)",
        "hmac_md5(message, key)",
        "hmac_sha1(message, key)",
        "hmac_sha256(message, key)",
        "hmac_sha512(message, key)",
    ];
    for (name, message, key, expected) in cases {
        let input = json!({ "message": message, "key": key });
        for (expression, expected) in expressions.iter().zip(expected) {
            let actual = runtime.compile(expression).unwrap().search(&input).unwrap();
            assert_eq!(actual, json!(expected), "{name}: {expression}");
        }
    }
}

#[test]
fn hashes_and_hmacs_still_require_string_arguments() {
    let runtime = Runtime::builder()
        .with_standard()
        .with_all_extensions()
        .build();
    for algorithm in ["md5", "sha1", "sha256", "sha512"] {
        let hash = runtime.compile(&format!("{algorithm}(@)")).unwrap();
        let mac = runtime
            .compile(&format!("hmac_{algorithm}(message, key)"))
            .unwrap();
        for invalid in [json!(null), json!(false), json!(1), json!([]), json!({})] {
            assert!(hash.search(&invalid).is_err(), "{algorithm}: {invalid}");
            assert!(
                mac.search(&json!({"message": invalid, "key": "key"}))
                    .is_err()
            );
            assert!(
                mac.search(&json!({"message": "message", "key": invalid}))
                    .is_err()
            );
        }
    }
}
