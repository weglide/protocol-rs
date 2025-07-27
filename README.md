# performance-rs

Load tests for data serving via `axum` using different protocol.
Given some clients that periodically want to request a list of around 500 strings, what is the best way to server them.

## Running via Docker

```sh
docker build -t protocol-rs .
docker run -ti -p 3000:3000 protocol-rs
```

## HTTP

Always 5k users making http request every 500ms.

```sh
k6 run k6-rest-test.js
```

## SSE

Always 5k users at the same time. The SSE streams gives them data every 500ms. They connect freshly every 5 seconds.
This equals 1000 HTTP requests per seconds and 10k data SSE per second.

```sh
k6 run k6-sse-test.js
```

You might want to increase the port range for this:

```sh
sudo sysctl -w net.inet.ip.portrange.first=16384
```

This seems to have less performance than HTTP, only achieves around 80% with 5000 concurrent users.

## WebSocket

Always 15k users at the same time. They receive data every 500ms.

```sh
k6 run k6-ws-test.js
```