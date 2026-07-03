use axum::{Router, extract::Query, routing::get};
use std::{collections::HashMap, time::Duration};

async fn root() -> &'static str {
    return "Hello World";
}

async fn test(Query(params): Query<HashMap<String, String>>) -> String {
    let param = params.get("param").map_or("unknown", String::as_str);
    println!("Received request, {}!", param);
    tokio::time::sleep(Duration::from_secs(1)).await;
    return format!("Processed request, {}!", param);
}

#[tokio::main]
async fn main() {
    println!("Server running");

    // our router
    let app = Router::new()
        .route("/", get(root))
        .route("/test", get(test));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
