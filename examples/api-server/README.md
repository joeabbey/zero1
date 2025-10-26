# REST API Server Example

A production-ready REST API server demonstrating Zero1's HTTP server capabilities, routing patterns, JSON handling, and capability-based security.

## Overview

This example implements a user management REST API with full CRUD operations (Create, Read, Update, Delete). It showcases:

- HTTP server setup and lifecycle management
- RESTful routing patterns (GET, POST, PUT, DELETE)
- JSON request/response handling
- Error handling and HTTP status codes
- Static file serving with filesystem capabilities
- Proper effect annotations and capability requirements

## Features

### API Endpoints

| Method | Path | Description | Status |
|--------|------|-------------|--------|
| GET | `/api/status` | Health check endpoint | 200 OK |
| GET | `/api/users` | List all users | 200 OK |
| GET | `/api/users/:id` | Get user by ID | 200 OK |
| POST | `/api/users` | Create new user | 201 Created |
| PUT | `/api/users/:id` | Update existing user | 200 OK |
| DELETE | `/api/users/:id` | Delete user | 204 No Content |
| GET | `/static/*` | Serve static files | 200 OK |

### Data Model

```z1
type User = {
  id: U32,
  name: Str,
  email: Str
}

type StatusResponse = {
  ok: Bool,
  message: Str
}
```

## Installation

### Prerequisites

- Zero1 toolchain installed
- Rust 1.75+ (for compilation)

### Build

```bash
# From the repository root
cargo build --workspace

# Compile the API server
z1c examples/api-server/main.z1c -o api-server.wasm
```

## Usage

### Starting the Server

```bash
# Run the compiled WASM module
z1run api-server.wasm
```

The server will start on port 8080 by default.

### Example Requests

#### Health Check
```bash
curl http://localhost:8080/api/status
# Response: {"ok": true, "message": "API Server Running"}
```

#### List All Users
```bash
curl http://localhost:8080/api/users
# Response: [{"id": 1, "name": "Alice", "email": "alice@example.com"}, ...]
```

#### Get User by ID
```bash
curl http://localhost:8080/api/users/1
# Response: {"id": 1, "name": "Alice", "email": "alice@example.com"}
```

#### Create New User
```bash
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Bob", "email": "bob@example.com"}'
# Response: {"id": 2, "name": "Bob", "email": "bob@example.com"}
```

#### Update User
```bash
curl -X PUT http://localhost:8080/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice Smith", "email": "alice.smith@example.com"}'
# Response: {"id": 1, "name": "Alice Smith", "email": "alice.smith@example.com"}
```

#### Delete User
```bash
curl -X DELETE http://localhost:8080/api/users/1
# Response: 204 No Content
```

#### Serve Static File
```bash
curl http://localhost:8080/static/index.html
# Response: (contents of static/index.html)
```

## Code Walkthrough

### Module Declaration

```z1
module example.api.server : 1.0
  ctx = 1024
  caps = [net, fs.ro]
```

- **Context budget**: 1024 tokens - sufficient for handling HTTP requests and routing logic
- **Capabilities**:
  - `net` - Required for HTTP server operations
  - `fs.ro` - Required for serving static files (read-only filesystem access)

### Imports

```z1
use "std/http/server" as http only [HttpRequest, HttpResponse, createServer, listen, getMethod, getPath, setStatus, setBody]
```

Imports only the necessary HTTP server functions, following Zero1's principle of minimal capability exposure.

### Request Routing

The `routeRequest` function demonstrates pattern matching on HTTP method and path:

```z1
fn routeRequest(req: http.HttpRequest) -> http.HttpResponse
  eff [fs]
{
  let method: Str = http.getMethod(req);
  let path: Str = http.getPath(req);
  // Route to appropriate handler based on method and path
  // ...
}
```

Effect annotation `eff [fs]` is required because this function may serve static files.

### CRUD Operations

Each CRUD operation is implemented as a separate handler function:

- **Create**: `handleCreateUser` - Parses JSON body, creates user, returns 201
- **Read**: `handleGetUser`, `handleListUsers` - Fetches user(s), returns JSON
- **Update**: `handleUpdateUser` - Parses ID and body, updates user, returns 200
- **Delete**: `handleDeleteUser` - Parses ID, deletes user, returns 204

All handlers are marked `eff [pure]` except those that perform I/O.

### JSON Handling

Helper functions handle JSON serialization/deserialization:

```z1
fn userToJson(user: User) -> Str
  eff [pure]
{
  // Convert User struct to JSON string
}

fn parseUserJson(json: Str) -> User
  eff [pure]
{
  // Parse JSON string to User struct
}
```

### Error Handling

The `Response` sum type provides error handling:

```z1
type Response = Ok{ response: http.HttpResponse } | Err{ error: Str }
```

This allows handlers to return either a successful response or an error message.

## Capability Requirements

### Network Capability (`net`)

Required for:
- Creating HTTP server
- Listening for connections
- Sending responses to clients

### Filesystem Read Capability (`fs.ro`)

Required for:
- Serving static files from `/static/*` paths
- Reading file contents

### Why These Capabilities?

Zero1's capability system ensures:
1. **Principle of least privilege**: Server only has read-only filesystem access
2. **Security**: Cannot write files or access system resources without explicit grants
3. **Auditability**: Capability requirements are declared upfront in module header

## Extension Ideas

For learners looking to extend this example:

1. **Database Integration**: Replace in-memory user storage with persistent database
   - Add `fs.rw` capability for SQLite
   - Implement proper user ID generation
   - Add database migration functions

2. **Authentication**: Add JWT-based authentication
   - Import `std/crypto` for token signing/verification
   - Add `crypto` capability
   - Implement login/logout endpoints

3. **Validation**: Add input validation for user data
   - Email format validation
   - Required field checks
   - Custom validation functions

4. **Middleware**: Implement middleware for logging, CORS, rate limiting
   - Add `time` capability for timestamps
   - Implement request/response interceptors

5. **WebSocket Support**: Add real-time communication
   - Import WebSocket types from stdlib
   - Implement connection upgrade logic
   - Add async message handling

6. **Advanced Routing**: Implement pattern matching for path parameters
   - Parse complex routes like `/api/users/:userId/posts/:postId`
   - Support query parameters
   - Handle wildcards and optional segments

## Expected Output

### Successful Health Check
```json
{
  "ok": true,
  "message": "API Server Running"
}
```

### User List Response
```json
[
  {
    "id": 1,
    "name": "Alice",
    "email": "alice@example.com"
  },
  {
    "id": 2,
    "name": "Bob",
    "email": "bob@example.com"
  }
]
```

### Error Response (404 Not Found)
```json
{
  "error": "Route not found",
  "path": "/api/invalid"
}
```

## Testing

Run the parsing tests to verify the code structure:

```bash
cargo test -p z1-parse examples_api_server
```

## Performance Characteristics

- **Context budget**: 1024 tokens fits comfortably in most LLM context windows
- **Memory**: In-memory user storage suitable for demos; use database for production
- **Concurrency**: Async effects allow handling multiple requests concurrently
- **Static files**: Efficient streaming with `fs.ro` capability

## Security Considerations

1. **Capabilities**: Minimal required capabilities (`net`, `fs.ro`)
2. **Input validation**: JSON parsing validates data structure
3. **Effect tracking**: Pure functions cannot perform I/O
4. **Static files**: Read-only prevents file modification attacks

## Related Examples

- `examples/http-hello/` - Simple HTTP server hello world
- `examples/cli-tool/` - File processing CLI with argument parsing
- `stdlib/http/` - HTTP server implementation details

## License

This example is part of the Zero1 project and follows the same license.
