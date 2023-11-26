use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, routing::post, Json, Router, TypedHeader, headers::{Authorization, authorization::Bearer}};
use azure_data_cosmos::prelude::CollectionClient;
use serde::Deserialize;
use shared::{get_collection_client, get_user_document, AppError, get_user_document_by_id, MatchStatus, Match};

struct AppState {
    collection_client: CollectionClient,
    search_endpoint: String,
    search_index_name: String,
}

#[derive(Deserialize)]
enum MatchOp {
    Add,
    Accept,
    Reject,
}

#[derive(Deserialize)]
struct AddMatchBody {
    operation: MatchOp,
    target_user_id: String,
    target_user_name: String,
    target_description: String,
}

async fn add_match(
    auth_header: TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddMatchBody>,
) -> Result<(), AppError> {
    println!("Start");
    // TODO server checks for the validity of the target user id passed
    let mut user_document =
        get_user_document(auth_header.token(), &state.collection_client).await?;
    let mut target_user_document = get_user_document_by_id(&payload.target_user_id, &state.collection_client).await?;

    let existing_user_match = user_document.matches.iter_mut().find(|el| el.id == payload.target_user_id);
    let existing_target_match = target_user_document.matches.iter_mut().find(|el| el.id == user_document.id);

    match payload.operation {
        MatchOp::Add => {
            if existing_user_match.is_some() || existing_target_match.is_some() {
                return Err(AppError::GenericError);
            }
            user_document.matches.push(Match {
                id: payload.target_user_id.clone(),
                name: payload.target_user_name,
                description: payload.target_description,
                match_status: MatchStatus::Pending,
            });
            target_user_document.matches.push(Match {
                id: user_document.id.clone(),
                name: user_document.name.clone(),
                description: user_document.description.clone().unwrap_or(String::new()),
                match_status: MatchStatus::AwaitingUserAction,
            });
        }
        MatchOp::Accept => {
            if let Some(user_match) = existing_user_match {
                user_match.match_status = MatchStatus::Accepted;
            } else {
                return Err(AppError::GenericError);
            }
            if let Some(target_match) = existing_target_match {
                target_match.match_status = MatchStatus::Accepted;
            } else {
                return Err(AppError::GenericError);
            }
        }
        MatchOp::Reject => {
            if let Some(user_match) = existing_user_match {
                user_match.match_status = MatchStatus::Denied;
            } else {
                return Err(AppError::GenericError);
            }
            if let Some(target_match) = existing_target_match {
                target_match.match_status = MatchStatus::Denied;
            } else {
                return Err(AppError::GenericError);
            }
        }
    };

    state
        .collection_client
        .document_client(user_document.id.clone(), &user_document.id)?
        .replace_document(user_document.clone())
        .await?;

    state
        .collection_client
        .document_client(target_user_document.id.clone(), &target_user_document.id)?
        .replace_document(target_user_document.clone())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let collection_client = get_collection_client().await.unwrap();
    let search_endpoint =
        std::env::var("SEARCH_ENDPOINT").expect("Set env variable SEARCH_ENDPOINT first!");
    let search_index_name =
        std::env::var("SEARCH_INDEX_NAME").expect("Set env variable SEARCH_INDEX_NAME first!");

    let shared_state = Arc::new(AppState { collection_client, search_endpoint, search_index_name });

    // build our application with a single route
    let app = Router::new()
        .route("/api/match", post(add_match))
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
