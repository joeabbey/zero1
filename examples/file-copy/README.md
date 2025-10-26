# File Copy Example

This example demonstrates the usage of the `std/fs` standard library module to implement a simple file copy utility.

## Features Demonstrated

- Reading file contents with `fs.readText()`
- Writing file contents with `fs.writeText()`
- Checking file existence with `fs.exists()`
- Proper error handling with `FsResult<T, E>` result type
- Capability-based security with `fs.ro` and `fs.rw`

## Module Structure

- **Module**: `example.filecopy:1.0`
- **Context Budget**: 512 tokens
- **Capabilities**: `[fs.ro, fs.rw]`

## Functions

### `copyFile(src: Str, dest: Str) -> FsResult<Unit, Str>`

Copies a file from source to destination path.

- Checks if source file exists
- Reads source file content
- Writes content to destination file
- Returns error if source doesn't exist

### `main() -> Unit`

Entry point that demonstrates copying `input.txt` to `output.txt`.

## Effect Annotations

Both functions use `eff [fs]` to indicate they perform file system operations, which requires the `fs.ro` and/or `fs.rw` capabilities declared at the module level.

## Usage

```bash
# Parse and verify the module
z1c examples/file-copy/main.z1r --check

# Format between compact and relaxed modes
z1fmt examples/file-copy/main.z1r --mode compact
z1fmt examples/file-copy/main.z1c --mode relaxed
```
