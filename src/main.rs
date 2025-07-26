use axum::{
    Router,
    extract::State,
    response::{
        Json,
        sse::{Event, Sse},
    },
    routing::get,
};
use futures::stream::{self, Stream};
use protocol_rs::{Gateway, GatewayApi};
use std::time::Duration;

#[derive(Clone)]
struct AppState<T: GatewayApi + Send + Sync + Clone>(T);

async fn hello_world<T: GatewayApi + Send + Sync + Clone>(
    State(AppState(api)): State<AppState<T>>,
) -> Json<Vec<String>> {
    let strings = api.read().await;
    Json(strings)
}

async fn sse_stream<T: GatewayApi + Send + Sync + Clone + 'static>(
    State(AppState(api)): State<AppState<T>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let stream = stream::unfold(api, |api| async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let data = api.read().await;
        let json_data = serde_json::to_string(&data).unwrap();
        let event = Event::default().data(json_data);
        Some((Ok(event), api))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[tokio::main]
async fn main() {
    let gateway = Gateway::new();
    let app_state = AppState(gateway.api());

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/sse", get(sse_stream))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("REST API: http://localhost:3000/");
    println!("SSE Stream: http://localhost:3000/sse");

    axum::serve(listener, app).await.unwrap();
}
