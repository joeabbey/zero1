# Data Processor Example

A production-quality data processing pipeline demonstrating Zero1's file I/O, cryptographic hashing, and data transformation capabilities.

## Overview

This example implements a complete ETL (Extract, Transform, Load) pipeline for processing CSV data files. It demonstrates real-world patterns for data validation, filtering, transformation, aggregation, and integrity verification using cryptographic hashes.

## Features Demonstrated

- **File I/O operations** - Read/write CSV files using `std/fs`
- **Data transformation pipeline** - Parse → Filter → Transform → Aggregate
- **Cryptographic hashing** - SHA-256 integrity verification using `std/crypto`
- **Statistics and reporting** - Compute aggregates (count, sum, average)
- **Error handling** - Proper validation and error propagation
- **Progress tracking** - Track processing metrics
- **Capability requirements** - Demonstrates `fs.ro`, `fs.rw`, and `crypto` capabilities

## Module Structure

- **Module**: `example.processor:1.0`
- **Context Budget**: 2048 tokens
- **Capabilities**: `[fs.ro, fs.rw, crypto]`

## Type Definitions

### `DataRow`
```z1
type DataRow = { id: U64, name: Str, value: U64, valid: Bool }
```
Represents a single row of data with validation status.

### `ProcessResult`
```z1
type ProcessResult = Ok{ rows: Str, stats: Statistics, hash: Str } | Err{ error: Str }
```
Result type for processing operations, containing processed data, statistics, and integrity hash on success, or error message on failure.

### `Statistics`
```z1
type Statistics = {
  totalRows: U64,
  validRows: U64,
  invalidRows: U64,
  sumValues: U64,
  avgValue: U64
}
```
Aggregate statistics computed during processing:
- `totalRows` - Total number of rows processed
- `validRows` - Number of valid rows after filtering
- `invalidRows` - Number of rows that failed validation
- `sumValues` - Sum of all valid values
- `avgValue` - Average value across valid rows

### `ProcessingPipeline`
```z1
type ProcessingPipeline = {
  inputPath: Str,
  outputPath: Str,
  filterEnabled: Bool,
  minValue: U64
}
```
Configuration for the processing pipeline:
- `inputPath` - Path to input CSV file
- `outputPath` - Path to write processed results
- `filterEnabled` - Whether to apply filtering
- `minValue` - Minimum value threshold for filtering

## Processing Pipeline

The data processor follows a classic ETL pattern:

```
1. Load      → Read CSV file from disk
2. Validate  → Parse and validate each row
3. Filter    → Remove invalid entries and apply criteria
4. Transform → Normalize and convert data types
5. Aggregate → Compute statistics (count, sum, average)
6. Hash      → Generate SHA-256 integrity hash
7. Write     → Save results to output file
8. Report    → Generate summary report
```

## Functions

### `parseRow(line: Str) -> DataRow`
**Effect**: `[pure]`

Parses a CSV line into a DataRow structure. Validates data format and sets the `valid` flag.

**Example**:
```z1
let row = parseRow("1,sensor_temp_01,72");
// Returns: DataRow{ id: 1, name: "sensor_temp_01", value: 72, valid: true }
```

### `filterRow(row: DataRow, minValue: U64) -> Bool`
**Effect**: `[pure]`

Determines if a row passes filtering criteria. Returns `false` if row is invalid or value is below minimum threshold.

**Example**:
```z1
let shouldInclude = filterRow(row, 50);
// Returns true if row.valid && row.value >= 50
```

### `transformRow(row: DataRow) -> DataRow`
**Effect**: `[pure]`

Transforms a data row by normalizing values, converting types, or applying business logic. Returns a new DataRow with transformed data.

**Example**:
```z1
let normalized = transformRow(row);
// Returns row with normalized value
```

### `computeStats(rows: Str, rowCount: U64) -> Statistics`
**Effect**: `[pure]`

Computes aggregate statistics across all processed rows. Calculates totals, counts, and averages.

**Example**:
```z1
let stats = computeStats(processedData, 100);
// Returns Statistics with all metrics populated
```

### `generateHash(data: Str) -> Str`
**Effect**: `[crypto]`

Generates a SHA-256 cryptographic hash of the processed data for integrity verification.

**Example**:
```z1
let hash = generateHash(processedData);
// Returns: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

### `processFile(pipeline: ProcessingPipeline) -> ProcessResult`
**Effect**: `[fs, crypto]`

Main processing function that orchestrates the entire pipeline:

1. Check if input file exists
2. Read file contents
3. Parse each row
4. Filter rows based on criteria
5. Transform valid rows
6. Compute statistics
7. Generate integrity hash
8. Return result with data, stats, and hash

**Example**:
```z1
let pipeline = ProcessingPipeline{
  inputPath: "data.csv",
  outputPath: "output.csv",
  filterEnabled: true,
  minValue: 50
};
let result = processFile(pipeline);
```

### `writeResults(outputPath: Str, rows: Str, hash: Str) -> WriteResult`
**Effect**: `[fs]`

Writes processed results to the output file. Returns success or error status.

### `generateReport(stats: Statistics, hash: Str) -> Str`
**Effect**: `[pure]`

Generates a human-readable summary report of the processing results.

**Example Output**:
```
Processing Report
-----------------
Total Rows:   100
Valid Rows:   95
Invalid Rows: 5
Sum of Values: 7,250
Average Value: 76.32
Integrity Hash: e3b0c4429...
```

### `main() -> Unit`
**Effect**: `[fs, crypto]`

Entry point that demonstrates the full processing pipeline on `sample-data.csv`.

## Sample Data Format

The included `sample-data.csv` contains sensor readings:

```csv
id,name,value
1,sensor_temp_01,72
2,sensor_temp_02,68
3,sensor_pressure_01,101
...
```

Contains 20 rows with:
- Temperature sensors (values 25-150)
- Pressure sensors (values 98-105)
- Humidity sensors (values 45-55)
- One invalid row (row 6) for error handling demonstration

## Architecture & Design

### Functional Pipeline Pattern

Each stage of the pipeline is a pure function that transforms input to output without side effects (except I/O operations). This makes the pipeline:
- **Testable** - Each function can be tested independently
- **Composable** - Functions can be chained in different orders
- **Debuggable** - Easy to inspect data at each stage
- **Reusable** - Functions work in different contexts

### Error Handling Strategy

The pipeline uses Zero1's sum types for explicit error handling:
- `ProcessResult` - Success with data or failure with error message
- `ReadResult` - File read success or failure
- `WriteResult` - File write success or failure

Errors propagate through the pipeline rather than causing crashes.

### Cryptographic Integrity

The SHA-256 hash serves multiple purposes:
1. **Integrity verification** - Detect corrupted or tampered data
2. **Change detection** - Identify if output has changed
3. **Provenance** - Link processed data to specific input
4. **Auditing** - Create verifiable processing trail

### Statistics Tracking

Real-time statistics provide:
- **Progress monitoring** - Track processing completion
- **Quality metrics** - Measure valid vs invalid ratio
- **Data insights** - Understand value distribution
- **Performance tuning** - Identify bottlenecks

## Effect Annotations Explained

- `eff [pure]` - No side effects (parsing, filtering, stats)
- `eff [fs]` - File system operations (read/write files)
- `eff [crypto]` - Cryptographic operations (hashing)
- `eff [fs, crypto]` - Combined file and crypto operations (main pipeline)

All file operations require `fs.ro` (read-only) or `fs.rw` (read-write) capabilities. Crypto operations require the `crypto` capability. All declared in the module header.

## Usage

```bash
# Parse and verify the processor module
cargo run -p z1-cli -- parse examples/processor/main.z1r

# Type check the module
cargo run -p z1-cli -- check examples/processor/main.z1r

# Format between compact and relaxed modes
cargo run -p z1-cli -- fmt examples/processor/main.z1r --mode compact
cargo run -p z1-cli -- fmt examples/processor/main.z1c --mode relaxed

# Generate TypeScript code
cargo run -p z1-cli -- codegen examples/processor/main.z1r --target ts

# Run with sample data (when runtime is available)
z1c run examples/processor/main.z1r
```

## Performance Considerations

### Time Complexity
- Parse: O(n) for n rows
- Filter: O(n) for n rows
- Transform: O(n) for n rows
- Statistics: O(n) for n rows
- Hash: O(m) for m bytes of data
- Overall: O(n + m) linear in rows and data size

### Memory Usage
- Input buffer: File size in memory
- Row storage: ~100 bytes per row
- Statistics: ~40 bytes
- Hash output: 32 bytes (SHA-256)
- For 10,000 rows: ~1MB input + ~1MB row data = ~2MB total

### Streaming Optimization

For large files, the pipeline can be adapted to stream:
1. Read file line-by-line instead of loading all at once
2. Process and write rows incrementally
3. Compute rolling statistics
4. Use streaming hash updates

This reduces memory usage from O(n) to O(1).

## Extension Ideas

### Beginner Extensions
1. Add support for different delimiters (tab, pipe, etc.)
2. Implement column selection (process only specific columns)
3. Add row number tracking in error messages
4. Create a summary-only mode (stats without writing output)

### Intermediate Extensions
1. Support multiple input files with merged output
2. Add data type inference (detect numbers, dates, etc.)
3. Implement data deduplication
4. Add configurable transformation functions
5. Support JSON output format

### Advanced Extensions
1. Parallel processing with chunking
2. Schema validation with type checking
3. SQL-like query language for filtering
4. Incremental processing (only process new data)
5. Data lineage tracking with full provenance chain
6. Compression support (gzip, zstd)
7. Streaming mode for arbitrarily large files

## Expected Output

When processing `sample-data.csv` with minValue=50:

```
Processing Report
-----------------
Total Rows:    20
Valid Rows:    18
Invalid Rows:  2
Sum of Values: 1,504
Average Value: 83.56
Integrity Hash: a7f3c9d2e8b1f0...

Filtered Out:
- Row 6: Invalid value format
- Row 14: Value 25 below minimum 50

Output written to: output.csv
```

The output file contains:
```csv
id,name,value
1,sensor_temp_01,72
2,sensor_temp_02,68
3,sensor_pressure_01,101
...
```

## Error Handling Examples

### Missing Input File
```z1
let result = processFile(pipeline);
// Returns: Err{ error: "Input file not found" }
```

### Invalid Data Format
```z1
let row = parseRow("1,sensor,invalid_number");
// Returns: DataRow{ ..., valid: false }
// Row is counted in invalidRows statistics
```

### Write Permission Error
```z1
let result = writeResults("/readonly/output.csv", data, hash);
// Returns: Err{ error: "Permission denied" }
```

## Related Examples

- **file-copy** - Basic file operations
- **password-hash** - Cryptographic operations
- **scheduler** - Async operations with progress tracking

## Learning Resources

- **Grammar**: See `docs/grammar.md` for Z1 syntax details
- **Capabilities**: See `docs/design.md` for capability system explanation
- **Stdlib**: See `stdlib/fs/` and `stdlib/crypto/` for implementation details
- **Type System**: See `docs/vision.md` for sum types and pattern matching
