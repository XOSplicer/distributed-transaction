use std::sync::Mutex;

use rocket::response::status;
use rocket::State;
use rocket::http;
use rocket_contrib::Json;

use itertools;

use transaction::{TransactionData, TransactionTime};
use transaction_log::*;
use client_data::*;

#[derive(Debug)]
pub struct TransactionLogState(pub Mutex<DualLog<String>>);

pub struct ClientListState(pub Mutex<ClientListHandler>);

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub base_url: String,
    pub tx_log_file: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        SettingsState {
            base_url: "http://localhost".into(),
            tx_log_file: "/tmp/tx_log.txt".into(),
        }
    }
}

#[get("/")]
pub fn read_all_transactions(
    tx_log: State<TransactionLogState>,
) -> Result<String, http::Status> {
    Ok(itertools::join(
        tx_log
            .0
            .lock()
            .map_err(|_| http::Status::InternalServerError)?
            .get_all()
            .map_err(|_| http::Status::InternalServerError)?
            .iter()
            .map(|t| t.to_string()),
        "\n",
    ))
}

#[get("/last")]
pub fn read_last_transaction(
    tx_log: State<TransactionLogState>,
) -> Result<Option<String>, http::Status> {
    Ok(
        tx_log
            .0
            .lock()
            .map_err(|_| http::Status::InternalServerError)?
            .last()
            .map_err(|_| http::Status::InternalServerError)?
            .map(|t| t.to_string()),
    )
}

#[get("/<id>")]
pub fn read_transaction(
    id: u32,
    tx_log: State<TransactionLogState>,
) -> Result<Option<String>, http::Status> {
    Ok(
        tx_log
            .0
            .lock()
            .map_err(|_| http::Status::InternalServerError)?
            .get_by_id(id)
            .map_err(|_| http::Status::InternalServerError)?
            .map(|t| t.to_string()),
    )
}

// example: $ curl -X PUT -d '020217-12:00:00;05;06;hello world' \
// http://localhost:8000/transactions/ -v
#[put("/", data = "<input>")]
pub fn write_transaction(
    input: String,
    tx_log: State<TransactionLogState>,
    settings: State<SettingsState>,
) -> Result<status::Created<String>, status::Custom<String>> {
    let mut parts = input.split(";");

    let time: TransactionTime = parts
        .next()
        .ok_or(status::Custom(
            http::Status::BadRequest,
            "No timestamp given".into(),
        ))?
        .parse()
        .map_err(|e| {
            status::Custom(http::Status::BadRequest, format!("{:?}", e))
        })?;

    let data: TransactionData = itertools::join(parts, ";").parse().map_err(
        |e| status::Custom(http::Status::BadRequest, format!("{:?}", e)),
    )?;

    let tx = tx_log
        .0
        .lock()
        .map_err(|_| {
            status::Custom(http::Status::InternalServerError, "".into())
        })?
        .create(data, Some(time))
        .map_err(|_| {
            status::Custom(http::Status::InternalServerError, "".into())
        })?;

    Ok(status::Created(
        format!("{}/transactions/{}", settings.base_url, tx.id().inner()),
        Some(tx.to_string()),
    ))
}


#[put("/register", format = "application/json", data = "<input>")]
pub fn register_client(
    input: Json<RegistrationRequest>,
    client_list: State<ClientListState>,
) -> Result<Json<TransactionClient>, http::Status> {
    unimplemented!()
}

#[put("/unregister", format = "application/json", data = "<input>")]
pub fn unregister_client(
    input: Json<TransactionClient>,
    client_list: State<ClientListState>,
) -> Result<Json<TransactionClient>, http::Status> {
    unimplemented!()
}

#[get("/client")]
pub fn list_clients(
    client_list: State<ClientListState>,
) -> Result<Json<Vec<TransactionClient>>, http::Status> {
    Ok(Json(
        client_list
            .0
            .lock()
            .map_err(|_| http::Status::InternalServerError)?
            .list()
    ))
}

#[get("/client/ping")]
pub fn ping() -> &'static str {
    "0"
}

#[put("/prepare", format = "application/json", data = "<input>")]
pub fn prepare(input: Json<PrepareMessage>) -> Result<(), http::Status> {
    unimplemented!()
}

#[put("/prepareAccept", format = "application/json", data = "<input>")]
pub fn prepare_accept(
    input: Json<PrepareAcceptMessage>,
) -> Result<(), http::Status> {
    unimplemented!()
}

#[put("/resolved", format = "application/json", data = "<input>")]
pub fn resolved(input: Json<ResolvedMessage>) -> Result<(), http::Status> {
    unimplemented!()
}