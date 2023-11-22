use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, routing::post, Json, Router, TypedHeader, headers::{Authorization, authorization::Bearer}};
use azure_data_cosmos::prelude::CollectionClient;
use serde::Deserialize;
use shared::{get_collection_client, get_user_document, AppError, Point, index_documents, IndexAction, UserSearchData};

struct AppState {
    collection_client: CollectionClient,
    search_endpoint: String,
    search_index_name: String,
    search_admin_key: String,
}

#[derive(Deserialize)]
struct SyncPositionBody {
    latitude: f64,
    longitude: f64,
}

async fn sync_position(
    auth_header: TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SyncPositionBody>
) -> Result<(), AppError> {
    let mut user_document =
        get_user_document(auth_header.token(), &state.collection_client).await?;

    user_document.location = Some(Point::new(payload.latitude, payload.longitude));

    state
        .collection_client
        .document_client(user_document.id.clone(), &user_document.id)?
        .replace_document(user_document.clone())
        .await?;

    index_documents(&state.search_endpoint, &state.search_index_name, &state.search_admin_key, &[
        IndexAction {
            action_type: shared::IndexActionType::Merge,
            user_document: UserSearchData::from(user_document)
        }
    ]).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let collection_client = get_collection_client().await.unwrap();
    let search_endpoint =
        std::env::var("SEARCH_ENDPOINT").expect("Set env variable SEARCH_ENDPOINT first!");
    let search_index_name =
        std::env::var("SEARCH_INDEX_NAME").expect("Set env variable SEARCH_INDEX_NAME first!");
    let search_admin_key =
        std::env::var("SEARCH_ADMIN_KEY").expect("Set env variable SEARCH_ADMIN_KEY first!");

    let shared_state = Arc::new(AppState { collection_client, search_endpoint, search_index_name, search_admin_key });

    // build our application with a single route
    let app = Router::new()
        .route("/api/sync_position", post(sync_position))
        .with_state(shared_state);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
