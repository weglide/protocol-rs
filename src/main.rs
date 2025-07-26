use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{
        IntoResponse, Json,
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
        let event = Event::default().json_data(&data).unwrap();
        Some((Ok(event), api))
    });

    Sse::new(stream)
}

async fn websocket_handler<T: GatewayApi + Send + Sync + Clone + 'static>(
    ws: WebSocketUpgrade,
    State(AppState(api)): State<AppState<T>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, api))
}

async fn handle_websocket<T: GatewayApi + Send + Sync + Clone + 'static>(
    mut socket: WebSocket,
    api: T,
) {
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let data = api.read().await;
        let json_data = serde_json::to_string(&data).unwrap();

        if socket.send(Message::Text(json_data)).await.is_err() {
            // Client disconnected
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let gateway = Gateway::new();
    let app_state = AppState(gateway.api());

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/sse", get(sse_stream))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("REST API: http://localhost:3000/");
    println!("SSE Stream: http://localhost:3000/sse");
    println!("WebSocket: ws://localhost:3000/ws");

    axum::serve(listener, app).await.unwrap();
}
