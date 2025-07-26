# performance-rs

Load tests for data serving via `axum` using different protocol.
Given some clients that periodically want to request a list of around 500 strings.

## HTTP

```sh
# Apache Bench, able to serve in 0.85 seconds
ab -n 10000 -c 1000 http://localhost:3000/
```