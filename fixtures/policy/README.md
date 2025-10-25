# Policy Test Fixtures

This directory contains test fixtures demonstrating various policy violations and valid modules.

## Valid Modules

- `valid_small.z1c` - A minimal valid module that passes all policy checks

## Policy Violations

- `too_many_params.z1c` - Function with 7 parameters (limit: 6)
- `missing_capability.z1c` - Function with `net` effect but no `net` capability
- `too_many_exports.z1c` - Module with 6 exports (limit: 5)

## Usage

These fixtures can be used to test the z1-policy crate's enforcement of:
- Cell-level limits (AST nodes, exports, imports)
- Function-level limits (parameters, locals, context budget)
- Module-level constraints (capabilities vs effects)
