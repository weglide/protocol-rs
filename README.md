# performance-rs

Load tests for data serving via `axum` using different protocol.
Given some clients that periodically want to request a list of around 500 strings.

## HTTP

Always 5000 users making http request every 500ms.

```sh
k6 run k6-rest-rest.js
```

## SSE

Always 5000 users at the same time. The SSE streams gives them data every 500ms. They connect freshly every 5 seconds.
This equals 1000 HTTP requests per seconds and 10k data SSE per second.

```sh
k6 run k6-sse-test.js
```

This seems to have less performance than HTTP, only achieves around 80% with 5000 concurrent users.