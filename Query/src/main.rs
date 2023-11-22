use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::post,
    Json, Router, TypedHeader,
};
use azure_data_cosmos::prelude::CollectionClient;
use serde::{Deserialize, Serialize};
use shared::{
    cognitive_query, get_collection_client, get_user_document, AppError, CognitiveResponse,
    UserSearchData,
};

struct AppState {
    collection_client: CollectionClient,
    search_endpoint: String,
    search_index_name: String,
    search_admin_key: String,
}

#[derive(Serialize, Deserialize)]
struct SearchResponse {
    data: CognitiveResponse,
}

async fn search(
    auth_header: TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SearchResponse>, AppError> {
    let user_document = get_user_document(auth_header.token(), &state.collection_client).await?;
    println!("Executing cognitive query...");
    let query_response = cognitive_query(
        &state.search_endpoint,
        &state.search_index_name,
        &state.search_admin_key,
        &UserSearchData::from(user_document),
    )
    .await?;

    Ok(Json(SearchResponse {
        data: query_response,
    }))
}

#[tokio::main]
async fn main() {
    println!("...");
    let collection_client = get_collection_client().await.unwrap();

    let search_endpoint =
        std::env::var("SEARCH_ENDPOINT").expect("Set env variable SEARCH_ENDPOINT first!");
    let search_index_name =
        std::env::var("SEARCH_INDEX_NAME").expect("Set env variable SEARCH_INDEX_NAME first!");
    let search_admin_key =
        std::env::var("SEARCH_ADMIN_KEY").expect("Set env variable SEARCH_ADMIN_KEY first!");

    let shared_state = Arc::new(AppState {
        collection_client,
        search_endpoint,
        search_index_name,
        search_admin_key,
    });

    // build our application with a single route
    let app = Router::new()
        .route("/api/query", post(search))
        .with_state(shared_state);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    println!("Ready!");
    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
