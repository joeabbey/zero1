# Password Hash Example

This example demonstrates using the Zero1 `std/crypto` library for secure password hashing.

## Features

- **Salt Generation**: Uses cryptographically secure random bytes for salts
- **SHA-256 Hashing**: Hashes passwords with salt using SHA-256
- **HMAC Verification**: Verifies password hashes using HMAC

## Structure

The example shows three key operations:

1. `generateSalt()` - Creates a random 16-byte salt using `std/crypto/random`
2. `hashPassword(password, salt)` - Combines password with salt and hashes using SHA-256
3. `verifyPassword(password, stored)` - Verifies a password against stored hash

## Security Notes

This is a simplified example for demonstration purposes. In production:

- Use specialized password hashing algorithms (bcrypt, argon2, scrypt)
- Implement proper key derivation (PBKDF2, etc.)
- Use timing-safe comparison for verification
- Store salt and hash securely

## Running

```bash
# Parse and check the example
cargo run -p z1-cli -- z1c examples/password-hash/main.z1r

# View compact form
cat examples/password-hash/main.z1c

# View relaxed form
cat examples/password-hash/main.z1r
```

## Capability Requirements

- `crypto` - Required for all cryptographic operations (hashing, random generation, HMAC)
