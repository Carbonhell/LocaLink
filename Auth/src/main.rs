use axum::{Json, TypedHeader};
use axum::extract::State;
use axum::routing::post;
use axum::{
    routing::get,
    Router,
};
use azure_data_cosmos::prelude::*;
use google_oauth::AsyncClient;
use rand::distributions::{Alphanumeric, DistString};
use shared::{get_collection_client, AppError, get_user_document, UserDocument, get_user_document_by_email};
use uuid::Uuid;
use std::env;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::Arc;
use axum::headers::Authorization;
use axum::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use shared::AppError::AuthError;

#[derive(Deserialize)]
struct AuthBody {
    id_token: String,
}

struct AppState {
    user_collection_client: CollectionClient,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserEntry {
    id: String,
    access_token: String,
}

impl azure_data_cosmos::CosmosEntity for UserEntry {
    type Entity = String;

    fn partition_key(&self) -> Self::Entity {
        self.id.clone()
    }
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();

    Alphanumeric.sample_string(&mut rng, 32)
}

/// Handle login via OAuth (currently only Google). The email is used as the user's identifier.
/// Registers the user if necessary, and generates a new random token to use for stateless authentication.
/// The token is stored in the user's record, which any authenticated endpoint can check when a request arrives.
async fn handle_auth(State(state): State<Arc<AppState>>, Json(payload): Json<AuthBody>) -> Result<Json<UserDocument>, AppError> {
    println!("Auth started");
    let id_token = payload.id_token;
    let client_id_key = "GOOGLE_CLIENT_ID";
    let client_id: String = env::var(client_id_key)?.parse().expect("Google client ID must be filled!");

    let client = AsyncClient::new(client_id);

    let data = client.validate_id_token(id_token).await;
    let user_document = match &data {
        Ok(data) => {
            // https://fly.io/blog/api-tokens-a-tedious-survey/ random tokens are a reasonable choice for a simple auth system like this that doesn't require policies
            let name = data.name.to_owned().unwrap();
            let email = data.email.to_owned().unwrap();
            let access_token = generate_token();
            let user_document = match get_user_document_by_email(&email, &state.user_collection_client).await {
                Ok(mut document) => {
                    document.access_token = access_token;
                    document
                }
                Err(_) => UserDocument {
                    id: String::from(Uuid::new_v4()),
                    email,
                    name,
                    access_token: access_token.clone(),
                    description: None,
                    description_embeddings: None,
                    location: None,
                    matches: Default::default(),
                }
            };

            state
                .user_collection_client
                .create_document(user_document.clone())
                .is_upsert(true)
                .await?;
            println!("User {} saved.", user_document.id);
            Ok(user_document)
        }
        Err(e) => Err(AuthError),
    }?;

    Ok(Json(user_document))
}

async fn refresh_profile(auth_header: TypedHeader<Authorization<Bearer>>, State(state): State<Arc<AppState>>) -> Result<Json<UserDocument>, AppError> {
    println!("Refreshing profile");
    let user_document = get_user_document(auth_header.token(), &state.user_collection_client).await?;

    Ok(Json(user_document))
}

#[tokio::main]
async fn main() -> azure_core::Result<()> {
    let collection_client = get_collection_client().await.unwrap();

    let shared_state = Arc::new(AppState { user_collection_client: collection_client });

    // build our application with a single route
    let app = Router::new()
        .route("/api/auth", post(handle_auth))
        .route("/api/auth", get(refresh_profile))
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

    Ok(())
}