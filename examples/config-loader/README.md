# Config Loader Example

Demonstrates loading configuration from environment variables using the `std/env` module.

## Features

- Load API keys and environment settings from environment variables
- Validate configuration before use
- Use process control to exit with error codes on validation failure

## Environment Variables

This example uses the following environment variables:

- `API_KEY` - API key for external services
- `ENV` - Environment name (development, staging, production)
- `DEBUG` - Debug mode flag (true/false)

## Usage

```bash
# Set environment variables
export API_KEY="your-api-key"
export ENV="development"
export DEBUG="true"

# Compile and run (future)
z1c examples/config-loader/main.z1r
```

## Code Overview

- `loadConfig()` - Loads configuration from environment variables (requires `env` capability)
- `validateConfig()` - Validates configuration values (pure function)
- `main()` - Entry point that loads, validates, and exits with appropriate code

## Capabilities Required

- `env` - For reading environment variables and process control
- `unsafe` - For exit() which terminates the process
