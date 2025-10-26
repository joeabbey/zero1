# std/env - Zero1 Environment & Process Standard Library

Environment variable access, command-line argument parsing, and process control for Zero1 applications.

## Modules

### `std/env/vars`

Environment variable operations.

**Types:**
- `EnvVar` - Environment variable with key and value

**Functions:**
- `getVar(name)` - Get environment variable value (requires `env` capability)
- `setVar(name, value)` - Set environment variable (requires `env` capability)
- `allVars()` - Get all environment variables as string (requires `env` capability)
- `hasVar(name)` - Check if environment variable exists (requires `env` capability)
- `removeVar(name)` - Remove environment variable (requires `env` capability)

**Example:**
```z1r
use "std/env/vars" as Env only [getVar, setVar, hasVar]

fn loadApiKey() {
  if Env.hasVar("API_KEY") {
    let key = Env.getVar("API_KEY");
    return key;
  } else {
    return "";
  };
}
```

### `std/env/args`

Command-line argument parsing.

**Types:**
- `Args` - Command-line arguments container

**Functions:**
- `getArgs()` - Get all command-line arguments (requires `env` capability)
- `parseFlags(args)` - Parse --flag=value style arguments (pure function)
- `hasFlag(args, flag)` - Check if flag exists in arguments (pure function)
- `getArg(index)` - Get argument at specific index (requires `env` capability)
- `argCount()` - Get number of command-line arguments (requires `env` capability)

**Example:**
```z1r
use "std/env/args" as Args only [getArgs, parseFlags, hasFlag]

fn main() {
  let args = Args.getArgs();
  let flags = Args.parseFlags(args);

  if Args.hasFlag(flags, "verbose") {
    // Enable verbose mode
  };
}
```

### `std/env/process`

Process control and information.

**Types:**
- `ProcessInfo` - Process information with PID and working directory

**Functions:**
- `exit(code)` - Exit process with status code (requires `env` and `unsafe` capabilities)
- `getPid()` - Get current process ID (requires `env` capability)
- `getCwd()` - Get current working directory (requires `env` capability)
- `getExecPath()` - Get path to current executable (requires `env` capability)

**Example:**
```z1r
use "std/env/process" as Proc only [getPid, getCwd, exit]

fn main() {
  let pid = Proc.getPid();
  let cwd = Proc.getCwd();

  // Validate configuration
  let valid = validateConfig();

  if valid {
    return;
  } else {
    Proc.exit(1);
  };
}
```

## Capabilities Required

All environment and process operations require the `env` capability:
```z1c
m myapp:1.0 caps=[env]
```

**Note:** The `exit()` function additionally requires the `unsafe` capability because it terminates the process.

## Effects

### Environment Variables (vars)
- `getVar()`, `setVar()`, `allVars()`, `hasVar()`, `removeVar()` - `[env]`

### Command-line Arguments (args)
- `getArgs()`, `getArg()`, `argCount()` - `[env]`
- `parseFlags()`, `hasFlag()` - `[pure]` (parsing is pure, reading args requires env)

### Process Control (process)
- `exit()` - `[env, unsafe]`
- `getPid()`, `getCwd()`, `getExecPath()` - `[env]`

## Security Considerations

The `env` capability grants access to:
- Environment variables (may contain secrets)
- Command-line arguments (may contain sensitive data)
- Process information (PID, working directory, executable path)
- Process termination via `exit()` (requires additional `unsafe` capability)

Only grant the `env` capability to trusted code. Use the principle of least privilege:
- If you only need to read specific environment variables, consider a wrapper module
- If you only need process information, avoid importing the `vars` module
- The `exit()` function requires explicit `unsafe` capability to prevent accidental termination

## Examples

See the `examples/config-loader/` directory for a complete example demonstrating:
- Loading configuration from environment variables
- Validating configuration
- Exiting with error codes on validation failure

## Notes

This is a minimal MVP implementation. Advanced features that may be added in future versions:
- Environment variable validation and parsing (to typed values)
- Structured argument parsing with subcommands
- Signal handling
- Process spawning and management
- File descriptor manipulation
- Resource limit queries

## Testing

Comprehensive tests are available in `crates/z1-parse/tests/stdlib_env.rs`:
- Parsing tests for all modules (compact and relaxed syntax)
- Type verification tests
- Effect and capability requirement tests
- Hash preservation tests (round-trip compact ↔ relaxed)
