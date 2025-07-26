use axum::{Router, response::Html, routing::get};

async fn hello_world() -> Html<&'static str> {
    Html("<h1>Hello, world!</h1>")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_world));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
