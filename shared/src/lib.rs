use anyhow::Error;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use azure_data_cosmos::prelude::{
    AuthorizationToken, CloudLocation, CollectionClient, CosmosClient, Param, Query, GetDocumentResponse,
};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use futures::{stream, StreamExt};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    env::VarError,
    error::Error as StdError,
    fmt::{self, Display},
};
use log::log;
use crate::AppError::NotFoundError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MatchStatus {
    /// Requires an action from the user the match was sent to.
    Pending = 1,
    /// Requires an action from the user who received the match.
    AwaitingUserAction = 2,
    /// Allows visualizing location data to allow users to meet.
    Accepted = 3,
    /// Discarded match, could be useful in future to prevent matching the same user again.
    Denied = 4,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Match {
    pub id: String,
    pub name: String,
    pub description: String,
    pub match_status: MatchStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDocument {
    pub id: String,
    pub email: String,
    pub name: String,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_embeddings: Option<Vec<f64>>,
    // todo vec length is constant, we can optimize this
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Point>,
    /// Denotes matches related to this user.
    pub matches: Vec<Match>,
}

impl azure_data_cosmos::CosmosEntity for UserDocument {
    type Entity = String;

    fn partition_key(&self) -> Self::Entity {
        self.id.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSearchData {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_embeddings: Option<Vec<f64>>,
    // todo vec length is constant, we can optimize this
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Point>,
}

impl From<UserDocument> for UserSearchData {
    fn from(user_doc: UserDocument) -> Self {
        UserSearchData {
            id: user_doc.id,
            name: user_doc.name,
            description: user_doc.description,
            description_embeddings: user_doc.description_embeddings,
            location: user_doc.location,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Point {
    #[serde(rename = "type")]
    r#type: String,
    pub coordinates: [f64; 2],
}

impl Point {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Point {
            r#type: "Point".to_owned(),
            coordinates: [latitude, longitude],
        }
    }
}

pub async fn get_collection_client() -> azure_core::Result<CollectionClient> {
    // CosmosDB configuration
    let primary_key =
        std::env::var("COSMOS_PRIMARY_KEY").expect("Set env variable COSMOS_PRIMARY_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    let main_db = std::env::var("COSMOS_DB").expect("Specify the main DB name first!");
    let users_collection =
        std::env::var("USERS_TABLE").expect("Specify the name of the users collection!");

    let authorization_token = match AuthorizationToken::primary_from_base64(&primary_key) {
        Ok(token) => token,
        Err(err) => panic!("Error while fetching auth token for Cosmos DB: {:?}", err)
    };

    /* let client = CosmosClient::builder(account, authorization_token)
    .cloud_location(CloudLocation::Emulator {
        address: "localhost".to_owned(),
        port: 8081,
    })
    .build(); */
    let client = CosmosClient::new(account, authorization_token);

    println!("Client built");

    let database_client = client.database_client(main_db);

    let collection_client = database_client.collection_client(users_collection.clone());

    Ok(collection_client)
}

pub async fn get_user_document(
    token: &str,
    collection: &CollectionClient,
) -> Result<UserDocument, AuthError> {
    println!("Querying user doc with token: {:?}", token);
    let mut docs_stream = collection
        .query_documents(Query::with_params(
            "SELECT * FROM users AS u WHERE u.access_token = @token".to_owned(),
            vec![Param::new("@token".into(), token)],
        ))
        .query_cross_partition(true) //TODO deep dive and figure out how to do in partition queries for this case
        .max_item_count(1)
        .into_stream::<UserDocument>();
    if let Some(query_response) = docs_stream.next().await {
        println!("User document found, {:?}", query_response);
        // In this page, the documents are under results
        if let Err(_) = query_response {
            return Err(AuthError);
        }
        let query_response = query_response.unwrap();
        if query_response.item_count > 0 {
            let (user_document, _) = query_response.results.first().unwrap();
            return Ok(user_document.clone());
        }
        println!("Empty doc array");
        return Err(AuthError);
    }
    Err(AuthError)
}

pub async fn get_user_document_by_id(
    id: &str,
    collection: &CollectionClient,
) -> Result<UserDocument, AppError> {
    println!("Querying user doc with id: {:?}", id);
    let doc_client = collection
        .document_client(id.clone(), &id)
        .map_err(|_| AuthError)?
        .get_document::<UserDocument>()
        .await;

    let result = doc_client.map_err(|_| AuthError).map(|doc| {
        if let GetDocumentResponse::Found(document) = doc {
            return Ok(document.document.document);
        } else {
            return Err(NotFoundError);
        };
    })?;
    println!("Result: {:?}", result);
    result
}

pub async fn get_user_document_by_email(
    email: &str,
    collection: &CollectionClient,
) -> Result<UserDocument, AuthError> {
    println!("Querying user doc with email: {:?}", email);
    let mut query_response = collection
        .query_documents(Query::with_params(
            "SELECT * FROM users AS u WHERE u.email = @email".to_owned(),
            vec![Param::new("@email".into(), email)],
        ))
        .query_cross_partition(true) //TODO deep dive and figure out how to do in partition queries for this case
        .max_item_count(1)
        .into_stream::<UserDocument>();

    if let Some(query_response) = query_response.next().await {
        println!("User document found, {:?}", query_response);
        // In this page, the documents are under results
        if let Err(_) = query_response {
            return Err(AuthError);
        }
        let query_response = query_response.unwrap();
        if query_response.item_count > 0 {
            let (user_document, _) = query_response.results.first().unwrap();
            return Ok(user_document.clone());
        }
        println!("Empty doc array");
        return Err(AuthError);
    }
    Err(AuthError)
}

#[derive(Debug)]
pub struct AuthError;

#[derive(Debug)]
pub enum AppError {
    GenericError,
    AuthError,
    NotFoundError,
    MissingLocationData,
}

/// This makes it possible to use `?` to automatically convert a `AuthError`
/// into an `AppError`.
impl From<AuthError> for AppError {
    fn from(inner: AuthError) -> Self {
        AppError::AuthError
    }
}

impl From<azure_core::error::Error> for AppError {
    fn from(inner: azure_core::error::Error) -> Self {
        println!("Azure error: {:?}", inner);
        AppError::GenericError
    }
}

impl From<VarError> for AppError {
    fn from(inner: VarError) -> Self {
        println!("Var error: {:?}", inner);
        AppError::GenericError
    }
}

impl From<reqwest::Error> for AppError {
    fn from(inner: reqwest::Error) -> Self {
        println!("Reqwest error: {:?}", inner);
        AppError::GenericError
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, code) = match self {
            AppError::GenericError => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error", 0),
            AppError::AuthError => (StatusCode::UNAUTHORIZED, "Authentication error", 0),
            AppError::NotFoundError => (StatusCode::NOT_FOUND, "Resource not found", 0),
            AppError::MissingLocationData => (StatusCode::NOT_FOUND, "Missing location data", 1),
        };

        let body = Json(json!({
            "code": code,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Function used to index data in a cognitive search index.
/// Currently needed since the azure rust sdk lacks any data operation on the cognitive search index.
pub async fn index_documents(
    endpoint: &str,
    index_name: &str,
    admin_key: &str,
    index_actions: &[IndexAction],
) -> Result<(), reqwest::Error> {
    let mut map = HashMap::new();
    map.insert("value", index_actions);

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{endpoint}/indexes('{index_name}')/docs/search.index"
        ))
        .query(&[("api-version", "2023-10-01-Preview")])
        .header("api-key", admin_key)
        .json(&map)
        .send()
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct VectorQuery {
    kind: String,
    vector: Vec<f64>,
    fields: String,
    k: u32,
}

#[derive(Serialize, Deserialize)]
struct CognitiveQueryBody {
    select: String,
    filter: String,
    vectorFilterMode: String,
    vectorQueries: Vec<VectorQuery>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CognitiveResponse {
    #[serde(rename = "@odata.context")]
    context: String,
    value: Vec<CognitiveResponseValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CognitiveResponseValue {
    #[serde(rename = "@search.score")]
    search_score: f32,
    id: String,
    name: String,
    description: String,
}

pub async fn cognitive_query(
    endpoint: &str,
    index_name: &str,
    admin_key: &str,
    user_document: &UserSearchData,
) -> Result<CognitiveResponse, AppError> {
    let user_location = user_document
        .location
        .as_ref()
        .ok_or(AppError::MissingLocationData)?;
    let user_description_embeddings = user_document
        .description_embeddings
        .as_ref()
        .ok_or(AppError::NotFoundError)?;

    let cognitive_query_body = CognitiveQueryBody {
        select: "id, name, description".to_owned(),
        filter: format!("id ne '{}' and geo.distance(location, geography'POINT({} {})') le 5",
                        user_document.id,
                        user_location.coordinates[0],
                        user_location.coordinates[1]
        ),
        vectorFilterMode: "preFilter".to_owned(),
        vectorQueries: vec![VectorQuery {
            kind: "vector".to_owned(),
            vector: user_description_embeddings.to_vec(),
            fields: "description_embeddings".to_owned(),
            k: 3,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{endpoint}/indexes('{index_name}')/docs/search.post.search"
        ))
        .query(&[("api-version", "2023-10-01-Preview")])
        .header("api-key", admin_key)
        .json(&cognitive_query_body)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<CognitiveResponse>().await?),
        _ => Err(AppError::GenericError),
    }
}

/// Struct info: https://learn.microsoft.com/en-us/rest/api/searchservice/2023-10-01-preview/documents/?tabs=HTTP#indexaction
#[derive(Serialize, Deserialize, Debug)]
pub struct IndexAction {
    #[serde(rename = "@search.action")]
    pub action_type: IndexActionType,
    #[serde(flatten)]
    pub user_document: UserSearchData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum IndexActionType {
    Delete,
    Merge,
    MergeOrUpload,
    Upload,
}
