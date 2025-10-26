# std/http - Zero1 HTTP Standard Library

Minimal HTTP server and client functionality for Zero1 applications.

## Modules

### `std/http/server`

HTTP server with basic request/response handling.

**Types:**
- `HttpRequest` - HTTP request with method, path, and body
- `HttpResponse` - HTTP response with status and body
- `HttpServer` - Server configuration with port

**Functions:**
- `createServer(port)` - Create server instance for given port
- `listen(server)` - Start listening for requests (async, requires `net` capability)
- `getMethod(req)` - Get HTTP method from request
- `getPath(req)` - Get request path
- `sendResponse(res)` - Send response to client (requires `net` capability)
- `setStatus(res, status)` - Set response HTTP status code
- `setBody(res, body)` - Set response body content

**Example:**
```z1r
use "std/http/server" as Http only [createServer, listen, setStatus, setBody, HttpRequest, HttpResponse]

fn main() {
  let server = Http.createServer(8080);
  Http.listen(server);
}
```

### `std/http/client`

HTTP client for making requests.

**Types:**
- `HttpMethod` - GET | POST | PUT | DELETE
- `HttpClient` - Client configuration with base URL and timeout

**Functions:**
- `get(url)` - HTTP GET request (async, requires `net` capability)
- `post(url, body)` - HTTP POST request (async, requires `net` capability)
- `put(url, body)` - HTTP PUT request (async, requires `net` capability)
- `delete(url)` - HTTP DELETE request (async, requires `net` capability)
- `fetch(url, method)` - Generic HTTP request (async, requires `net` capability)

**Example:**
```z1r
use "std/http/client" as Client only [get]

fn fetchData() {
  let data = Client.get("https://api.example.com/data");
  return data;
}
```

## Capabilities Required

All HTTP operations require the `net` capability:
```z1c
m myapp:1.0 caps=[net]
```

## Effects

- `listen()` - `[net, async]`
- `sendResponse()` - `[net]`
- `get()`, `post()`, `put()`, `delete()`, `fetch()` - `[net, async]`
- All other functions - `[pure]`

## Notes

This is a minimal MVP implementation. Advanced features like:
- HTTP headers
- Query parameters
- Request/response streaming
- Middleware
- Routing

...will be added in future versions as the Zero1 language and type system evolve.
