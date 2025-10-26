# Zero1 Standard Library Reference

The Zero1 standard library provides essential functionality for common programming tasks. All modules are designed with capability-based security and minimal token usage.

## Overview

The standard library is organized into functional areas:

- **[std/http](#stdhttp)** - HTTP client and server
- **[std/time](#stdtime)** - Time, dates, and timers
- **[std/fs](#stdfs)** - File system operations
- **[std/crypto](#stdcrypto)** - Cryptographic primitives
- **[std/env](#stdenv)** - Environment and process control

## std/http

HTTP functionality for building web applications and making requests.

### std/http/server

HTTP server with basic request/response handling.

**Module:** `std/http/server`

**Capabilities Required:** `net`

**Context Budget:** 384 tokens

**Types:**

```z1r
type HttpRequest = {
  method: Str,
  path: Str,
  body: Str
}

type HttpResponse = {
  status: U32,
  body: Str
}

type HttpServer = {
  port: U16
}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `createServer` | `(port: U16) -> HttpServer` | `[pure]` | Create server configuration |
| `listen` | `(server: HttpServer) -> Unit` | `[net, async]` | Start listening for requests |
| `getMethod` | `(req: HttpRequest) -> Str` | `[pure]` | Extract HTTP method |
| `getPath` | `(req: HttpRequest) -> Str` | `[pure]` | Extract request path |
| `setStatus` | `(res: HttpResponse, status: U32) -> HttpResponse` | `[pure]` | Set response status code |
| `setBody` | `(res: HttpResponse, body: Str) -> HttpResponse` | `[pure]` | Set response body |
| `sendResponse` | `(res: HttpResponse) -> Unit` | `[net]` | Send response to client |

**Example:**

```z1r
module web.server : 1.0
  ctx = 256
  caps = [net]

use "std/http/server" as Http only [createServer, listen, HttpRequest, HttpResponse]

fn main() -> Unit
  eff [net, async]
{
  let server = Http.createServer(8080);
  Http.listen(server);
  return ();
}
```

### std/http/client

HTTP client for making outbound requests.

**Module:** `std/http/client`

**Capabilities Required:** `net`

**Context Budget:** 320 tokens

**Types:**

```z1r
type HttpMethod = GET | POST | PUT | DELETE

type HttpClient = {
  baseUrl: Str,
  timeout: U32
}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `get` | `(url: Str) -> Str` | `[net, async]` | HTTP GET request |
| `post` | `(url: Str, body: Str) -> Str` | `[net, async]` | HTTP POST request |
| `put` | `(url: Str, body: Str) -> Str` | `[net, async]` | HTTP PUT request |
| `delete` | `(url: Str) -> Str` | `[net, async]` | HTTP DELETE request |
| `fetch` | `(url: Str, method: HttpMethod) -> Str` | `[net, async]` | Generic HTTP request |

**Example:**

```z1r
use "std/http/client" as Client only [get]

fn fetchData(url: Str) -> Str
  eff [net, async]
{
  return Client.get(url);
}
```

## std/time

Time, date, duration, and timer functionality.

### std/time/core

Core time operations including timestamps and durations.

**Module:** `std/time/core`

**Capabilities Required:** `time`

**Context Budget:** 320 tokens

**Types:**

```z1r
type Timestamp = { millis: U64 }

type Duration = { millis: U64 }

type DateTime = {
  year: U32,
  month: U32,
  day: U32,
  hour: U32,
  minute: U32,
  second: U32
}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `now` | `() -> Timestamp` | `[time]` | Current timestamp |
| `nowMillis` | `() -> U64` | `[time]` | Current time in milliseconds |
| `sleep` | `(seconds: U32) -> Unit` | `[time, async]` | Sleep for N seconds |
| `sleepMillis` | `(millis: U64) -> Unit` | `[time, async]` | Sleep for N milliseconds |
| `add` | `(ts: Timestamp, dur: Duration) -> Timestamp` | `[pure]` | Add duration to timestamp |
| `subtract` | `(ts: Timestamp, dur: Duration) -> Timestamp` | `[pure]` | Subtract duration |
| `fromMillis` | `(millis: U64) -> Timestamp` | `[pure]` | Create from milliseconds |
| `toMillis` | `(ts: Timestamp) -> U64` | `[pure]` | Convert to milliseconds |
| `format` | `(dt: DateTime) -> Str` | `[pure]` | Format as string |
| `parse` | `(s: Str) -> DateTime` | `[pure]` | Parse string to datetime |

**Example:**

```z1r
use "std/time/core" as Time only [now, sleep, add, Duration]

fn delayedOperation() -> Timestamp
  eff [time, async]
{
  let start = Time.now();
  Time.sleep(5);
  let dur = Duration{ millis: 1000 };
  return Time.add(start, dur);
}
```

### std/time/timer

Timer for measuring elapsed time.

**Module:** `std/time/timer`

**Capabilities Required:** `time`

**Context Budget:** 256 tokens

**Types:**

```z1r
type Timer = {
  startMillis: U64,
  running: Bool
}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `create` | `() -> Timer` | `[pure]` | Create new timer |
| `start` | `(timer: Timer) -> Timer` | `[time]` | Start timer |
| `stop` | `(timer: Timer) -> Timer` | `[pure]` | Stop timer |
| `reset` | `(timer: Timer) -> Timer` | `[pure]` | Reset to zero |
| `elapsed` | `(timer: Timer) -> U64` | `[time]` | Get elapsed milliseconds |

**Example:**

```z1r
use "std/time/timer" as Timer only [create, start, elapsed]

fn measurePerformance() -> U64
  eff [time]
{
  let timer = Timer.create();
  let running = Timer.start(timer);
  // ... do work ...
  return Timer.elapsed(running);
}
```

## std/fs

File system operations with fine-grained capability control.

### std/fs/core

Basic file I/O operations.

**Module:** `std/fs/core`

**Capabilities Required:** `fs.ro` (read), `fs.rw` (write)

**Context Budget:** 256 tokens

**Types:**

```z1r
type FsResult<T, E> = Ok{ value: T } | Err{ error: E }
```

**Functions:**

| Function | Signature | Effect | Capability | Description |
|----------|-----------|--------|------------|-------------|
| `readText` | `(path: Str) -> FsResult<Str, Str>` | `[fs]` | `fs.ro` | Read file as text |
| `writeText` | `(path: Str, content: Str) -> FsResult<Unit, Str>` | `[fs]` | `fs.rw` | Write text to file |
| `exists` | `(path: Str) -> Bool` | `[fs]` | `fs.ro` | Check if file exists |
| `remove` | `(path: Str) -> FsResult<Unit, Str>` | `[fs]` | `fs.rw` | Delete file |

**Example:**

```z1r
module file.processor : 1.0
  ctx = 512
  caps = [fs.ro, fs.rw]

use "std/fs/core" as Fs only [readText, writeText, FsResult]

fn copyFile(src: Str, dst: Str) -> FsResult<Unit, Str>
  eff [fs]
{
  let result = Fs.readText(src);
  return match result {
    Ok{ value: content } -> Fs.writeText(dst, content),
    Err{ error: msg } -> Err{ error: msg }
  };
}
```

### std/fs/dir

Directory manipulation operations.

**Module:** `std/fs/dir`

**Capabilities Required:** `fs.ro` (read), `fs.rw` (write)

**Context Budget:** 256 tokens

**Types:**

```z1r
type FileList = { files: Vec<Str> }
```

**Functions:**

| Function | Signature | Effect | Capability | Description |
|----------|-----------|--------|------------|-------------|
| `list` | `(path: Str) -> FsResult<FileList, Str>` | `[fs]` | `fs.ro` | List directory contents |
| `createDir` | `(path: Str) -> FsResult<Unit, Str>` | `[fs]` | `fs.rw` | Create directory |
| `removeDir` | `(path: Str) -> FsResult<Unit, Str>` | `[fs]` | `fs.rw` | Delete directory |

**Example:**

```z1r
use "std/fs/dir" as Dir only [list, createDir]

fn setupDirectories() -> FsResult<Unit, Str>
  eff [fs]
{
  return Dir.createDir("output");
}
```

### std/fs/path

Pure path manipulation (no file system access).

**Module:** `std/fs/path`

**Capabilities Required:** None (all pure)

**Context Budget:** 128 tokens

**Types:**

```z1r
type PathParts = { components: Vec<Str> }
type PathOption<T> = Some{ val: T } | None{}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `join` | `(parts: PathParts) -> Str` | `[pure]` | Join path components |
| `basename` | `(path: Str) -> Str` | `[pure]` | Get filename from path |
| `dirname` | `(path: Str) -> Str` | `[pure]` | Get directory from path |
| `extension` | `(path: Str) -> PathOption<Str>` | `[pure]` | Get file extension |

**Example:**

```z1r
use "std/fs/path" as Path only [join, basename, extension]

fn processPath(fullPath: Str) -> Str
  eff [pure]
{
  let name = Path.basename(fullPath);
  let ext = Path.extension(fullPath);
  return name;
}
```

## std/crypto

Cryptographic primitives for hashing, authentication, and random generation.

### std/crypto/hash

Cryptographic hash functions.

**Module:** `std/crypto/hash`

**Capabilities Required:** `crypto`

**Context Budget:** 256 tokens

**Types:**

```z1r
type HashAlgorithm = Sha256 | Sha3_256 | Blake3
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `sha256` | `(data: Str) -> Str` | `[crypto]` | Compute SHA-256 hash |
| `sha3_256` | `(data: Str) -> Str` | `[crypto]` | Compute SHA3-256 hash |
| `hashBytes` | `(data: Str, alg: HashAlgorithm) -> Str` | `[crypto]` | Hash with chosen algorithm |

**Example:**

```z1r
use "std/crypto/hash" as Hash only [sha256]

fn computeChecksum(data: Str) -> Str
  eff [crypto]
{
  return Hash.sha256(data);
}
```

### std/crypto/hmac

HMAC message authentication codes.

**Module:** `std/crypto/hmac`

**Capabilities Required:** `crypto`

**Context Budget:** 256 tokens

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `hmacSha256` | `(key: Str, msg: Str) -> Str` | `[crypto]` | Generate HMAC-SHA256 |
| `verifyHmac` | `(key: Str, msg: Str, mac: Str) -> Bool` | `[crypto]` | Verify HMAC signature |

**Example:**

```z1r
use "std/crypto/hmac" as Hmac only [hmacSha256, verifyHmac]

fn signRequest(secret: Str, payload: Str) -> Str
  eff [crypto]
{
  return Hmac.hmacSha256(secret, payload);
}

fn validateRequest(secret: Str, payload: Str, signature: Str) -> Bool
  eff [crypto]
{
  return Hmac.verifyHmac(secret, payload, signature);
}
```

### std/crypto/random

Cryptographically secure random generation.

**Module:** `std/crypto/random`

**Capabilities Required:** `crypto`

**Context Budget:** 256 tokens

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `randomBytes` | `(length: U32) -> Str` | `[crypto]` | Generate random bytes (hex) |
| `randomU32` | `() -> U32` | `[crypto]` | Generate random U32 |
| `randomU64` | `() -> U64` | `[crypto]` | Generate random U64 |
| `randomRange` | `(min: U32, max: U32) -> U32` | `[crypto]` | Random integer in range |

**Example:**

```z1r
use "std/crypto/random" as Rand only [randomBytes, randomU32]

fn generateToken() -> Str
  eff [crypto]
{
  return Rand.randomBytes(32);
}

fn rollDice() -> U32
  eff [crypto]
{
  return Rand.randomRange(1, 7);  // 1-6
}
```

## std/env

Environment variables, command-line arguments, and process control.

### std/env/vars

Environment variable access.

**Module:** `std/env/vars`

**Capabilities Required:** `env`

**Context Budget:** 256 tokens

**Types:**

```z1r
type EnvResult<T> = Some{ val: T } | None{}
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `get` | `(key: Str) -> EnvResult<Str>` | `[env]` | Get environment variable |
| `set` | `(key: Str, val: Str) -> Unit` | `[env]` | Set environment variable |
| `remove` | `(key: Str) -> Unit` | `[env]` | Remove environment variable |
| `has` | `(key: Str) -> Bool` | `[env]` | Check if variable exists |

**Example:**

```z1r
use "std/env/vars" as Env only [get, EnvResult]

fn getConfig() -> Str
  eff [env]
{
  let result = Env.get("API_KEY");
  return match result {
    Some{ val: key } -> key,
    None{} -> "default-key"
  };
}
```

### std/env/args

Command-line argument access.

**Module:** `std/env/args`

**Capabilities Required:** `env`

**Context Budget:** 192 tokens

**Types:**

```z1r
type ArgList = { args: Vec<Str> }
```

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `getArgs` | `() -> ArgList` | `[env]` | Get all command-line arguments |
| `getArg` | `(index: U32) -> EnvResult<Str>` | `[env]` | Get argument at index |
| `count` | `() -> U32` | `[env]` | Count arguments |

**Example:**

```z1r
use "std/env/args" as Args only [getArgs, ArgList]

fn processCommandLine() -> Unit
  eff [env]
{
  let argList: ArgList = Args.getArgs();
  // Process argList.args
  return ();
}
```

### std/env/process

Process control and exit codes.

**Module:** `std/env/process`

**Capabilities Required:** `env`

**Context Budget:** 192 tokens

**Functions:**

| Function | Signature | Effect | Description |
|----------|-----------|--------|-------------|
| `exit` | `(code: U32) -> Unit` | `[env]` | Exit with code |
| `getPid` | `() -> U32` | `[env]` | Get process ID |

**Example:**

```z1r
use "std/env/process" as Proc only [exit]

fn failAndExit(msg: Str) -> Unit
  eff [env]
{
  // Log error message
  Proc.exit(1);
  return ();
}
```

## Capability Requirements Summary

| Module | Capability | Read | Write | Async |
|--------|-----------|------|-------|-------|
| `std/http/server` | `net` | ✓ | ✓ | ✓ |
| `std/http/client` | `net` | ✓ | - | ✓ |
| `std/time/core` | `time` | ✓ | - | ✓ |
| `std/time/timer` | `time` | ✓ | - | - |
| `std/fs/core` (read) | `fs.ro` | ✓ | - | - |
| `std/fs/core` (write) | `fs.rw` | ✓ | ✓ | - |
| `std/fs/dir` (list) | `fs.ro` | ✓ | - | - |
| `std/fs/dir` (modify) | `fs.rw` | - | ✓ | - |
| `std/fs/path` | None | - | - | - |
| `std/crypto/hash` | `crypto` | ✓ | - | - |
| `std/crypto/hmac` | `crypto` | ✓ | - | - |
| `std/crypto/random` | `crypto` | ✓ | - | - |
| `std/env/vars` | `env` | ✓ | ✓ | - |
| `std/env/args` | `env` | ✓ | - | - |
| `std/env/process` | `env` | ✓ | ✓ | - |

## Usage Patterns

### Error Handling with Result Types

Many stdlib functions return `Result<T, E>` types:

```z1r
use "std/fs/core" as Fs only [readText, FsResult]

fn loadConfig() -> Str
  eff [fs]
{
  let result: FsResult<Str, Str> = Fs.readText("config.json");
  return match result {
    Ok{ value: content } -> content,
    Err{ error: msg } -> "{}"  // Default empty config
  };
}
```

### Optional Values with Option Types

Handle missing values with `Option<T>`:

```z1r
use "std/env/vars" as Env only [get, EnvResult]

fn getPort() -> U32
  eff [env]
{
  let portStr = Env.get("PORT");
  return match portStr {
    Some{ val: s } -> parseU32(s),
    None{} -> 8080  // Default port
  };
}
```

### Combining Multiple Capabilities

```z1r
module app : 1.0
  ctx = 1024
  caps = [net, fs.ro, time, crypto]

use "std/http/server" as Http
use "std/fs/core" as Fs
use "std/crypto/hash" as Hash

fn serveStaticFile(path: Str) -> Http.HttpResponse
  eff [fs, crypto]
{
  let content = Fs.readText(path);
  let hash = Hash.sha256(content);
  // Return file with ETag
}
```

## Examples

Complete working examples are available in the `examples/` directory:

- **`examples/http-hello/`** - Simple HTTP server
- **`examples/file-copy/`** - File I/O with error handling
- **`examples/password-hash/`** - Cryptographic operations
- **`examples/time-demo/`** - Timestamps and timers
- **`examples/config-loader/`** - Environment variables and args

## Implementation Status

All stdlib modules are currently **MVP (Minimum Viable Product)**:

- Type signatures and APIs are complete and stable
- Function bodies are stubs returning placeholder values
- Parser and type checker fully validate usage
- Full implementation requires backend integration

## Next Steps

- **[Compilation Guide](04-compilation.md)** - Understand how stdlib is compiled
- **[Best Practices](05-best-practices.md)** - Use stdlib effectively
- **[Language Tour](02-language-tour.md)** - Review language features
