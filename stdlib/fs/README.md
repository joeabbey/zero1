# stdlib/fs - File System Standard Library

The `std.fs` module provides file system operations with capability-based security for Zero1 applications.

## Modules

### `std.fs.core` - Core File Operations

Provides basic file I/O operations.

**Capabilities Required**: `fs.ro` (read-only), `fs.rw` (read-write)

**Types**:
- `FsResult<T, E>` - Result type for file operations (Ok/Err variants)

**Functions**:

- `readText(path: Str) -> FsResult<Str, Str>` - Read file contents as text
  - Effect: `[fs]`
  - Returns: File contents on success, error message on failure

- `writeText(path: Str, content: Str) -> FsResult<Unit, Str>` - Write text to file
  - Effect: `[fs]`
  - Returns: Unit on success, error message on failure

- `exists(path: Str) -> Bool` - Check if file exists
  - Effect: `[fs]`
  - Returns: true if file exists, false otherwise

- `remove(path: Str) -> FsResult<Unit, Str>` - Delete a file
  - Effect: `[fs]`
  - Returns: Unit on success, error message on failure

### `std.fs.dir` - Directory Operations

Provides directory manipulation operations.

**Capabilities Required**: `fs.ro` (read-only), `fs.rw` (read-write)

**Types**:
- `FsResult<T, E>` - Result type for directory operations
- `FileList` - Container for directory listing results

**Functions**:

- `list(path: Str) -> FsResult<FileList, Str>` - List directory contents
  - Effect: `[fs]`
  - Returns: FileList on success, error message on failure

- `createDir(path: Str) -> FsResult<Unit, Str>` - Create directory
  - Effect: `[fs]`
  - Returns: Unit on success, error message on failure

- `removeDir(path: Str) -> FsResult<Unit, Str>` - Delete directory
  - Effect: `[fs]`
  - Returns: Unit on success, error message on failure

### `std.fs.path` - Path Manipulation

Provides pure path manipulation functions (no file system access required).

**Capabilities Required**: None (all functions are pure)

**Types**:
- `PathParts` - Container for path components
- `PathOption<T>` - Optional value (Some/None variants)

**Functions**:

- `join(parts: PathParts) -> Str` - Join path components
  - Effect: `[pure]`
  - Returns: Joined path string

- `basename(path: Str) -> Str` - Get filename from path
  - Effect: `[pure]`
  - Returns: Base filename

- `dirname(path: Str) -> Str` - Get directory from path
  - Effect: `[pure]`
  - Returns: Directory path

- `extension(path: Str) -> PathOption<Str>` - Get file extension
  - Effect: `[pure]`
  - Returns: Some(extension) or None

## Capability Model

The fs module uses fine-grained capabilities:

- **`fs.ro`** - Read-only file system access
  - Allows: `readText`, `exists`, `list`
  - Safe for untrusted code that needs to inspect files

- **`fs.rw`** - Read-write file system access
  - Allows: `writeText`, `remove`, `createDir`, `removeDir`
  - Required for any modification operations

- **No capability** - Pure path manipulation
  - `std.fs.path` requires no capabilities
  - All operations are pure string transformations

## Usage Example

```z1r
module example.fileops : 1.0
  ctx = 512
  caps = [fs.ro, fs.rw]

use "std/fs/core" as fs only [readText, writeText, exists]
use "std/fs/path" as path only [basename, extension]

fn processFile(filepath: Str) -> FsResult<Unit, Str>
  eff [fs]
{
  let fileExists: Bool = fs.exists(filepath);

  if fileExists {
    let content: FsResult<Str, Str> = fs.readText(filepath);
    let name: Str = path.basename(filepath);
    let ext: PathOption<Str> = path.extension(filepath);

    return fs.writeText("output.txt", name);
  } else {
    return Err{ error: "File not found" };
  }
}
```

## Compact vs Relaxed Syntax

All modules are available in both compact (`.z1c`) and relaxed (`.z1r`) formats:

- **Compact**: Token-efficient, ideal for LLM consumption
- **Relaxed**: Human-readable, ideal for editing

Both formats parse to identical AST and have the same semantic hash.

## Error Handling

All file operations return `FsResult<T, E>` which is a sum type:
- `Ok{ value: T }` - Operation succeeded
- `Err{ error: E }` - Operation failed with error message

Always pattern match or handle both cases:

```z1r
let result: FsResult<Str, Str> = fs.readText("config.txt");

match result {
  Ok{ value: content } -> {
    // Use content
  },
  Err{ error: msg } -> {
    // Handle error
  }
}
```

## Testing

Comprehensive tests are available in `crates/z1-parse/tests/stdlib_fs.rs`:
- 18 tests covering parsing, capabilities, effects, and hash preservation
- Tests verify both compact and relaxed formats
- Round-trip tests ensure semantic hash stability

## Examples

See `examples/file-copy/` for a complete file copy utility demonstrating:
- File existence checking
- Reading file contents
- Writing file contents
- Error handling with FsResult
- Proper capability declarations

## Context Budgets

- `std.fs.core`: 256 tokens
- `std.fs.dir`: 256 tokens
- `std.fs.path`: 128 tokens (smaller due to pure functions)

## Implementation Status

This is an MVP (Minimum Viable Product) implementation:
- Core API surface is complete
- Function bodies are stubs (return placeholder values)
- Types and signatures are production-ready
- Full implementation requires backend integration

## Future Enhancements

Planned features for future releases:
- Async file operations
- Streaming read/write for large files
- File metadata (size, permissions, timestamps)
- Symbolic link operations
- File watching/monitoring
- Glob pattern matching for directory listings
