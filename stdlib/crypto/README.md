# Zero1 Crypto Standard Library

The `std/crypto` module provides cryptographic operations with capability-based security. All functions require the `crypto` capability.

## Modules

### `std/crypto/hash` - Cryptographic Hashing

Provides industry-standard hash functions for data integrity and security.

**Types:**
- `HashAlgorithm` - Sum type of supported algorithms: `Sha256 | Sha3_256 | Blake3`

**Functions:**
- `sha256(data: Str) -> Str` - Compute SHA-256 hash (256-bit)
- `sha3_256(data: Str) -> Str` - Compute SHA3-256 hash (FIPS 202)
- `hashBytes(data: Str, algorithm: HashAlgorithm) -> Str` - Generic hashing with algorithm selection

**Capability:** `crypto`

**Context Budget:** 256 tokens

**Example:**
```z1r
use "std/crypto/hash" as Hash

fn computeDigest(content: Str) -> Str
  eff [crypto]
{
  ret Hash.sha256(content);
}
```

### `std/crypto/hmac` - Message Authentication Codes

HMAC (Hash-based Message Authentication Code) for verifying data integrity and authenticity.

**Functions:**
- `hmacSha256(key: Str, message: Str) -> Str` - Generate HMAC-SHA256
- `verifyHmac(key: Str, message: Str, mac: Str) -> Bool` - Verify HMAC signature

**Capability:** `crypto`

**Context Budget:** 256 tokens

**Example:**
```z1r
use "std/crypto/hmac" as Hmac

fn signMessage(secret: Str, data: Str) -> Str
  eff [crypto]
{
  ret Hmac.hmacSha256(secret, data);
}
```

### `std/crypto/random` - Secure Random Generation

Cryptographically secure random number generation for keys, nonces, and unpredictable values.

**Functions:**
- `randomBytes(length: U32) -> Str` - Generate random bytes (hex-encoded)
- `randomU32() -> U32` - Generate random 32-bit unsigned integer
- `randomU64() -> U64` - Generate random 64-bit unsigned integer
- `randomRange(min: U32, max: U32) -> U32` - Generate random integer in range [min, max)

**Capability:** `crypto`

**Context Budget:** 256 tokens

**Example:**
```z1r
use "std/crypto/random" as Rand

fn generateToken() -> Str
  eff [crypto]
{
  ret Rand.randomBytes(32);
}
```

## Security Considerations

1. **Capability Requirement:** All crypto operations require the `crypto` capability. This must be declared in your module:
   ```z1r
   module myapp : 1
     caps = [crypto]
   ```

2. **Randomness Quality:** The `random` module uses cryptographically secure RNGs suitable for security-critical applications (keys, tokens, nonces). Do not use for gaming or simulations if reproducibility is needed.

3. **Hash Selection:**
   - **SHA-256:** Widely deployed, FIPS-validated, suitable for general use
   - **SHA3-256:** Modern alternative (Keccak), immune to length-extension attacks
   - **Blake3:** High-performance modern hash, parallel-friendly

4. **HMAC Usage:** Always use unique keys per application context. Never reuse keys across different security boundaries.

5. **Key Management:** This library provides primitives only. For production systems:
   - Use key derivation functions (KDF) for password-based keys
   - Implement proper key rotation policies
   - Store keys securely (hardware security modules, key management services)

## Common Patterns

### Password Hashing
See `examples/password-hash/` for a complete example demonstrating:
- Salt generation with `randomBytes()`
- Password hashing with `sha256()`
- Verification with `verifyHmac()`

**Note:** For production password storage, use specialized algorithms like Argon2, bcrypt, or scrypt (not yet available in Zero1 stdlib).

### Token Generation
```z1r
fn createSessionToken() -> Str
  eff [crypto]
{
  ret Rand.randomBytes(32);
}
```

### Data Integrity
```z1r
fn checksumFile(contents: Str) -> Str
  eff [crypto]
{
  ret Hash.sha256(contents);
}
```

### API Signature
```z1r
fn signApiRequest(key: Str, payload: Str) -> Str
  eff [crypto]
{
  ret Hmac.hmacSha256(key, payload);
}
```

## Future Additions

Planned additions to `std/crypto`:
- Asymmetric encryption (RSA, Ed25519)
- Key derivation functions (PBKDF2, Argon2)
- Authenticated encryption (AES-GCM, ChaCha20-Poly1305)
- Certificate handling (X.509, TLS)

## Files

- `hash.z1c` / `hash.z1r` - Hashing functions
- `hmac.z1c` / `hmac.z1r` - HMAC operations
- `random.z1c` / `random.z1r` - Secure random generation

Each module provides both compact (`.z1c`) and relaxed (`.z1r`) syntax versions that are semantically identical.
