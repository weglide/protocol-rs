use axum::{Router, extract::State, response::Json, routing::get};
use protocol_rs::{Gateway, GatewayApi};

#[derive(Clone)]
struct AppState<T: GatewayApi + Send + Sync + Clone>(T);

async fn hello_world<T: GatewayApi + Send + Sync + Clone>(
    State(AppState(api)): State<AppState<T>>,
) -> Json<Vec<String>> {
    let strings = api.read().await;
    Json(strings)
}

#[tokio::main]
async fn main() {
    let gateway = Gateway::new();
    let app_state = AppState(gateway.api());

    let app = Router::new()
        .route("/", get(hello_world))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
