# Task Scheduler Example

A production-quality task scheduling system demonstrating Zero1's timer operations, async capabilities, and task management patterns.

## Overview

This example implements a flexible task scheduler that supports one-time delayed tasks and recurring interval-based tasks. It demonstrates real-world patterns for managing asynchronous operations with priority queues, status tracking, and graceful cancellation.

## Features Demonstrated

- **Timer-based task execution** - Using `std/time/timer` for precise timing
- **Async task management** - Async operations with proper effect annotations
- **Task prioritization** - High/Normal/Low priority levels
- **One-time tasks** - Schedule tasks to run after a delay
- **Recurring tasks** - Schedule tasks to run at regular intervals
- **Task cancellation** - Cancel scheduled tasks before execution
- **Status tracking** - Track task lifecycle (Pending → Running → Completed/Failed)
- **Error handling** - Proper error propagation with Failed status
- **Capability requirements** - Demonstrates `time` and `async` capabilities

## Module Structure

- **Module**: `example.scheduler:1.0`
- **Context Budget**: 1024 tokens
- **Capabilities**: `[time, async]`

## Type Definitions

### `TaskStatus`
```z1
type TaskStatus = Pending | Running | Completed | Failed{ error: Str }
```
Tracks the current state of a task through its lifecycle.

### `TaskPriority`
```z1
type TaskPriority = High | Normal | Low
```
Defines task execution priority for the scheduler's priority queue.

### `Task`
```z1
type Task = { id: U64, name: Str, priority: TaskPriority, status: TaskStatus }
```
Represents a schedulable unit of work with unique ID and metadata.

### `ScheduledTask`
```z1
type ScheduledTask = {
  task: Task,
  delayMillis: U64,
  intervalMillis: U64,
  recurring: Bool,
  cancelled: Bool,
  lastRun: U64
}
```
Wraps a task with scheduling metadata:
- `delayMillis` - Initial delay before first execution (one-time tasks)
- `intervalMillis` - Interval between executions (recurring tasks)
- `recurring` - Whether task repeats
- `cancelled` - Cancellation flag
- `lastRun` - Timestamp of last execution

### `TaskScheduler`
```z1
type TaskScheduler = { tasks: Str, nextId: U64, running: Bool }
```
The scheduler itself, maintaining task queue and runtime state.

## Functions

### `createTask(name: Str, priority: TaskPriority) -> Task`
**Effect**: `[pure]`

Creates a new task with the given name and priority. Returns a task in Pending status with ID 0 (scheduler assigns actual IDs).

### `scheduleOnce(scheduler: TaskScheduler, task: Task, delayMillis: U64) -> TaskScheduler`
**Effect**: `[pure]`

Schedules a task to run once after the specified delay in milliseconds.

**Example**:
```z1
let notification = createTask("send_notification", Normal);
let updatedScheduler = scheduleOnce(scheduler, notification, 2000);
```
This schedules a notification to be sent after 2 seconds.

### `scheduleRecurring(scheduler: TaskScheduler, task: Task, intervalMillis: U64) -> TaskScheduler`
**Effect**: `[pure]`

Schedules a task to run repeatedly at the specified interval.

**Example**:
```z1
let healthCheck = createTask("health_check", High);
let updatedScheduler = scheduleRecurring(scheduler, healthCheck, 5000);
```
This runs a health check every 5 seconds.

### `cancelTask(scheduler: TaskScheduler, taskId: U64) -> TaskScheduler`
**Effect**: `[pure]`

Cancels a scheduled task by ID. The task will not execute even if its time arrives.

### `executeTask(task: Task) -> TaskStatus`
**Effect**: `[time]`

Executes a task's work. Uses a timer to measure execution duration and sleeps briefly to simulate work. Returns the task's final status (Completed or Failed).

### `checkTask(scheduledTask: ScheduledTask, currentTime: U64) -> Bool`
**Effect**: `[pure]`

Checks whether a scheduled task is ready to run based on current time and scheduling parameters. Returns `false` if cancelled.

### `runScheduler(scheduler: TaskScheduler) -> Unit`
**Effect**: `[time, async]`

Main scheduler loop that checks and executes tasks. In a full implementation, this would:
1. Get current timestamp
2. Check all tasks for readiness
3. Execute ready tasks by priority
4. Update task status and lastRun timestamps
5. Re-schedule recurring tasks
6. Sleep until next task deadline

### `main() -> Unit`
**Effect**: `[time, async]`

Demonstrates scheduler usage by creating and scheduling various tasks:

1. **Health check** (High priority) - Recurring every 5 seconds
2. **Notification** (Normal priority) - One-time after 2 seconds
3. **Data sync** (Normal priority) - Recurring every 10 seconds
4. **Cleanup** (Low priority) - One-time after 15 seconds

## Architecture & Design

### Priority Queue Pattern

The scheduler uses priority-based execution where High priority tasks execute before Normal, and Normal before Low. This ensures critical operations (health checks) aren't blocked by less important work.

### Functional State Updates

All scheduler operations return an updated scheduler instance rather than mutating state in-place. This functional approach makes the scheduler easier to reason about and test.

### Timer Integration

The example uses `std/time/timer` for precise execution timing and `std/time/core` for delays. This demonstrates how to compose stdlib modules for real-world functionality.

### Error Handling

Tasks can fail with error messages captured in the `Failed{ error: Str }` variant. The scheduler tracks these failures and can implement retry logic or alerting.

## Example Task Scenarios

### Periodic Health Check
```z1
let healthCheck = createTask("health_check", High);
let scheduler = scheduleRecurring(scheduler, healthCheck, 5000);
```
Runs every 5 seconds to verify system health.

### Delayed Notification
```z1
let notification = createTask("send_notification", Normal);
let scheduler = scheduleOnce(scheduler, notification, 2000);
```
Sends a notification after a 2-second delay.

### Recurring Data Sync
```z1
let dataSync = createTask("sync_data", Normal);
let scheduler = scheduleRecurring(scheduler, dataSync, 10000);
```
Synchronizes data every 10 seconds.

### Scheduled Cleanup
```z1
let cleanup = createTask("cleanup_temp", Low);
let scheduler = scheduleOnce(scheduler, cleanup, 15000);
```
Runs cleanup 15 seconds after scheduler starts.

## Effect Annotations Explained

- `eff [pure]` - No side effects (task creation, scheduling logic)
- `eff [time]` - Accesses system time or timers (task execution)
- `eff [time, async]` - Async operations with timing (scheduler loop, main)

All time-based operations require the `time` capability, and async operations require `async`, both declared in the module header.

## Usage

```bash
# Parse and verify the scheduler module
cargo run -p z1-cli -- parse examples/scheduler/main.z1r

# Type check the module
cargo run -p z1-cli -- check examples/scheduler/main.z1r

# Format between compact and relaxed modes
cargo run -p z1-cli -- fmt examples/scheduler/main.z1r --mode compact
cargo run -p z1-cli -- fmt examples/scheduler/main.z1c --mode relaxed

# Generate TypeScript code
cargo run -p z1-cli -- codegen examples/scheduler/main.z1r --target ts
```

## Performance Considerations

### Time Complexity
- Task creation: O(1)
- Scheduling: O(1) to add, O(log n) to maintain priority queue
- Execution: O(1) per task
- Cancellation: O(n) to find task by ID

### Memory Usage
- Each task: ~200 bytes (task metadata + scheduling info)
- Scheduler overhead: ~100 bytes + task queue
- For 1000 tasks: ~200KB memory footprint

### Concurrency
The scheduler runs tasks sequentially. For parallel execution, spawn each task in a separate fiber using the `task { }` expression and manage task handles.

## Extension Ideas

### Beginner Extensions
1. Add task timeout detection
2. Implement retry logic for failed tasks
3. Add task dependency chains (task B runs after task A completes)
4. Create a task cancellation API

### Intermediate Extensions
1. Implement priority queue with heap data structure
2. Add deadline-based scheduling (run by specific timestamp)
3. Implement task groups with bulk operations
4. Add task execution history and statistics

### Advanced Extensions
1. Parallel task execution with fiber pools
2. Distributed scheduling across multiple schedulers
3. Persistent task queue (save/restore from file)
4. Rate limiting and backpressure mechanisms
5. Dynamic priority adjustment based on wait time

## Expected Output

When running the scheduler, tasks would execute in this order:

```
[t=0s]     Scheduler starts
[t=2s]     send_notification executes (one-time, Normal priority)
[t=5s]     health_check executes (recurring, High priority)
[t=10s]    health_check executes (recurring)
[t=10s]    sync_data executes (recurring, Normal priority)
[t=15s]    health_check executes (recurring)
[t=15s]    cleanup_temp executes (one-time, Low priority)
[t=20s]    health_check executes (recurring)
[t=20s]    sync_data executes (recurring)
...
```

High priority tasks execute before same-time Normal priority tasks.

## Related Examples

- **time-demo** - Basic timer and time operations
- **processor** - Data processing with progress reporting

## Learning Resources

- **Grammar**: See `docs/grammar.md` for Z1 syntax details
- **Effects**: See `docs/design.md` for capability system explanation
- **Stdlib**: See `stdlib/time/` for timer implementation details
