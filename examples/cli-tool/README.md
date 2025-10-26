# File Processor CLI Tool

A production-quality command-line tool demonstrating Zero1's file I/O, argument parsing, environment variable handling, and exit code management capabilities.

## Overview

This example implements a versatile file processing CLI tool that reads input files, transforms their content, and writes results to output files. It showcases:

- Command-line argument parsing with multiple flags
- Environment variable configuration
- File I/O operations (read, write, exists checks)
- Text transformation pipelines
- Error handling with proper exit codes
- Statistics reporting (lines processed, bytes read/written)
- Stdin/stdout support for Unix pipelines
- Capability-based security model

## Features

### Command-Line Operations

| Command | Description | Exit Code |
|---------|-------------|-----------|
| `process --input <file> --output <file>` | Process file with transformation | 0 on success |
| `process --input <file> --mode <mode>` | Process with specific mode | 0 on success |
| `process --list <directory>` | List files in directory | 0 on success |
| `process --config <env-var>` | Load config from environment | 0 on success |
| `process --help` | Show help message | 0 |
| `process` (no args) | Show help and exit | 0 |

### Processing Modes

- **uppercase** - Convert all text to uppercase
- **lowercase** - Convert all text to lowercase
- **count** - Count lines and report statistics
- **reverse** - Reverse line order

### Data Model

```z1
type Config = {
  inputPath: Str,
  outputPath: Str,
  mode: Str,
  configVar: Str
}

type ProcessStats = {
  linesProcessed: U32,
  bytesRead: U32,
  bytesWritten: U32
}

type ProcessResult = Ok{ stats: ProcessStats } | Err{ error: Str }
```

## Installation

### Prerequisites

- Zero1 toolchain installed
- Rust 1.75+ (for compilation)

### Build

```bash
# From the repository root
cargo build --workspace

# Compile the CLI tool
z1c examples/cli-tool/main.z1c -o processor.wasm
```

## Usage

### Basic File Processing

```bash
# Process file with default mode (uppercase)
z1run processor.wasm --input input.txt --output output.txt

# Process with specific mode
z1run processor.wasm --input data.txt --output result.txt --mode lowercase
```

### Using Environment Variables

```bash
# Load configuration from environment
export CONFIG_FILE="/path/to/config.json"
z1run processor.wasm --config CONFIG_FILE

# Set processing mode via environment
export PROCESS_MODE="uppercase"
z1run processor.wasm --input file.txt
```

### Stdin/Stdout Pipeline

```bash
# Read from stdin, write to stdout
cat input.txt | z1run processor.wasm --stdin --mode uppercase > output.txt

# Chain with other Unix tools
z1run processor.wasm --stdin < data.csv | grep "pattern" > filtered.csv
```

### Directory Listing

```bash
# List files in current directory
z1run processor.wasm --list .

# List files in specific directory
z1run processor.wasm --list /path/to/directory
```

### Help and Information

```bash
# Show help message
z1run processor.wasm --help

# Show version
z1run processor.wasm --version
```

## Example Workflows

### 1. Text Transformation

**Input file (input.txt):**
```
hello world
zero1 is powerful
capabilities rock
```

**Command:**
```bash
z1run processor.wasm --input input.txt --output output.txt --mode uppercase
```

**Output file (output.txt):**
```
HELLO WORLD
ZERO1 IS POWERFUL
CAPABILITIES ROCK
```

**Statistics:**
```
Lines processed: 3
Bytes read: 54
Bytes written: 54
```

### 2. Environment Configuration

**Setup:**
```bash
export PROCESS_MODE="lowercase"
export INPUT_FILE="data.txt"
export OUTPUT_FILE="result.txt"
```

**Command:**
```bash
z1run processor.wasm --config INPUT_FILE
```

### 3. Pipeline Integration

```bash
# Count lines in large file
z1run processor.wasm --input huge.log --mode count

# Transform and filter
z1run processor.wasm --stdin --mode uppercase < input.txt | grep ERROR > errors.txt
```

## Code Walkthrough

### Module Declaration

```z1
module example.cli.processor : 1.0
  ctx = 768
  caps = [env, fs.ro, fs.rw]
```

- **Context budget**: 768 tokens - adequate for CLI argument parsing and file operations
- **Capabilities**:
  - `env` - Required for reading command-line arguments and environment variables
  - `fs.ro` - Required for reading input files
  - `fs.rw` - Required for writing output files

### Imports

```z1
use "std/env/args" as args only [getArgs, argCount, getArg]
use "std/env/vars" as envVars only [getVar, hasVar]
use "std/env/process" as process only [exit, getCwd]
use "std/fs/core" as fs only [readText, writeText, exists, ReadResult, WriteResult]
```

Imports cover three key areas:
1. **Argument parsing** - Get command-line arguments
2. **Environment access** - Read environment variables and control process
3. **File I/O** - Read, write, and check file existence

### Argument Parsing

```z1
fn parseArgs() -> Config
  eff [env]
{
  let argc: U32 = args.argCount();
  // Parse arguments into Config struct
  // Handle --input, --output, --mode, --config, --help, etc.
}
```

Effect annotation `eff [env]` is required because this function reads environment (command-line args).

### File Processing Pipeline

The processing flow follows this pattern:

1. **Validate** - Check input file exists
2. **Read** - Load file contents into memory
3. **Transform** - Apply mode-specific transformation
4. **Write** - Save results to output file
5. **Report** - Display statistics

```z1
fn processFile(config: Config) -> ProcessResult
  eff [fs]
{
  let inputExists: Bool = fs.exists(config.inputPath);
  if inputExists {
    let readResult: fs.ReadResult = fs.readText(config.inputPath);
    // Process content...
    return Ok{ stats: stats };
  } else {
    return Err{ error: "Input file not found" };
  };
}
```

### Error Handling

The `ProcessResult` sum type provides robust error handling:

```z1
type ProcessResult = Ok{ stats: ProcessStats } | Err{ error: Str }
```

Pattern matching on results allows proper error propagation:

```z1
fn main() -> Unit
  eff [env, fs, unsafe]
{
  let result: ProcessResult = processFile(config);
  // Match on result to determine exit code
}
```

### Exit Codes

The tool uses standard Unix exit codes:

- `0` - Success
- `1` - General error (invalid arguments, missing files)
- `2` - Configuration error
- `130` - Interrupted (Ctrl+C)

```z1
fn main() -> Unit
  eff [env, fs, unsafe]
{
  if valid {
    process.exit(0);  // Success
  } else {
    process.exit(1);  // Error
  };
}
```

Note: `unsafe` effect is required for `exit()` as it terminates the process.

### Text Transformation

```z1
fn processLine(line: Str, mode: Str) -> Str
  eff [pure]
{
  // Transform single line based on mode
  // Pure function - no side effects
}

fn transformText(text: Str, mode: Str) -> Str
  eff [pure]
{
  // Transform entire text
  // Calls processLine for each line
}
```

Pure functions ensure transformations are deterministic and testable.

### Statistics Collection

```z1
type ProcessStats = {
  linesProcessed: U32,
  bytesRead: U32,
  bytesWritten: U32
}

fn printStats(stats: ProcessStats) -> Unit
  eff [pure]
{
  // Display statistics to user
}
```

## Capability Requirements

### Environment Capability (`env`)

Required for:
- Reading command-line arguments via `args.getArgs()`, `args.argCount()`
- Accessing environment variables via `envVars.getVar()`, `envVars.hasVar()`
- Getting current working directory via `process.getCwd()`
- Exiting with status codes via `process.exit()`

### Filesystem Read Capability (`fs.ro`)

Required for:
- Reading input files via `fs.readText()`
- Checking file existence via `fs.exists()`
- Listing directory contents

### Filesystem Write Capability (`fs.rw`)

Required for:
- Writing output files via `fs.writeText()`
- Creating new files
- Overwriting existing files

### Why These Capabilities?

Zero1's capability system ensures:
1. **Transparency**: All I/O operations are declared upfront
2. **Security**: Tool cannot access network or other system resources
3. **Auditability**: Users can verify exactly what the tool can do
4. **Sandboxing**: Runtime can enforce capability restrictions

## Extension Ideas

For learners looking to extend this example:

1. **Advanced Parsing**: Add support for complex argument patterns
   - Long and short flags (`--input` / `-i`)
   - Flag validation and type checking
   - Required vs optional arguments
   - Subcommands (`process transform`, `process validate`)

2. **Multiple File Processing**: Process multiple input files
   - Glob pattern matching (`*.txt`)
   - Directory traversal
   - Parallel processing with async effects
   - Progress reporting

3. **Advanced Transformations**: Add more processing modes
   - Regex-based find/replace
   - JSON/CSV parsing and manipulation
   - Line filtering by pattern
   - Column extraction and rearrangement

4. **Configuration Files**: Support external configuration
   - JSON/TOML config file parsing
   - Config file schema validation
   - Merging CLI args with config file
   - Config file discovery (`.processorrc`)

5. **Interactive Mode**: Add REPL for exploratory processing
   - Read-eval-print loop
   - Command history
   - Tab completion
   - Interactive help

6. **Streaming**: Process large files without loading entirely into memory
   - Line-by-line streaming
   - Chunk-based processing
   - Memory-efficient transformations
   - Progress indicators for large files

7. **Error Recovery**: Advanced error handling
   - Partial results on errors
   - Skip invalid lines vs fail-fast modes
   - Detailed error messages with line numbers
   - Dry-run mode for validation

## Exit Codes Reference

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | File processed successfully |
| 1 | Invalid arguments | Missing --input flag |
| 1 | File not found | Input file doesn't exist |
| 1 | Permission denied | Cannot read input file |
| 1 | Write error | Cannot write to output path |
| 2 | Configuration error | Invalid config file format |

## Expected Output

### Successful Processing

```
Processing input.txt...
Mode: uppercase
Lines processed: 42
Bytes read: 1,234
Bytes written: 1,234
Output written to output.txt
```

### Error Handling

```
Error: Input file not found
Path: /path/to/missing.txt
Try: processor --help
Exit code: 1
```

### Help Message

```
Zero1 File Processor v1.0

USAGE:
    processor --input <file> --output <file> [options]

OPTIONS:
    --input <file>      Input file path
    --output <file>     Output file path
    --mode <mode>       Processing mode (uppercase, lowercase, count, reverse)
    --config <var>      Load config from environment variable
    --list <dir>        List files in directory
    --help              Show this help message
    --version           Show version information

EXAMPLES:
    processor --input data.txt --output result.txt
    processor --input file.txt --mode uppercase
    cat input.txt | processor --stdin --mode lowercase > output.txt

For more information, visit: https://github.com/zero1/docs
```

## Testing

Run the parsing tests to verify the code structure:

```bash
cargo test -p z1-parse examples_cli_tool
```

## Performance Characteristics

- **Context budget**: 768 tokens - efficient for CLI tools
- **Memory**: Loads entire file into memory (suitable for typical text files)
- **I/O**: Single read, transform, single write pattern
- **Startup**: Fast startup time due to minimal dependencies
- **Scalability**: Handles files up to several MB efficiently

## Security Considerations

1. **Capabilities**: Minimal required capabilities (`env`, `fs.ro`, `fs.rw`)
2. **Path validation**: Should validate file paths to prevent directory traversal
3. **Input sanitization**: Environment variables should be validated
4. **Effect tracking**: Pure transformations cannot perform I/O
5. **Exit safety**: `unsafe` effect required for `exit()` makes termination explicit

## Related Examples

- `examples/api-server/` - REST API server with HTTP routing
- `examples/config-loader/` - Environment variable configuration
- `stdlib/fs/` - File system operations implementation
- `stdlib/env/` - Environment access implementation

## License

This example is part of the Zero1 project and follows the same license.
