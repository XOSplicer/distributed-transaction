#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate chrono;
extern crate rocket;
extern crate rocket_contrib;
extern crate clap;
extern crate itertools;
#[macro_use]
extern crate quick_error;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;


mod transaction;
mod transaction_log;
mod client_data;

use std::fs::OpenOptions;
use std::sync::Mutex;
use std::net::Ipv4Addr;

use rocket::response::status;
use rocket::State;
use rocket::http;
use rocket::config::{Config, Environment};

use rocket_contrib::Json;

use clap::{App, Arg};

use transaction::{TransactionData, TransactionTime};
use transaction_log::*;
use client_data::*;

#[derive(Debug)]
struct TransactionLogState(Mutex<DualLog<String>>);


struct ClientListState(Mutex<ClientListHandler>);

#[derive(Debug, Clone)]
struct SettingsState {
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
fn read_all_transactions(
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
fn read_last_transaction(
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
fn read_transaction(
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
fn write_transaction(
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
fn register_client(
    input: Json<RegistrationRequest>,
    client_list: State<ClientListState>,
) -> Result<Json<TransactionClient>, http::Status> {
    unimplemented!()
}

#[put("/unregister", format = "application/json", data = "<input>")]
fn unregister_client(
    input: Json<TransactionClient>,
    client_list: State<ClientListState>,
) -> Result<Json<TransactionClient>, http::Status> {
    unimplemented!()
}

#[get("/client")]
fn list_clients(
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
fn ping() -> &'static str {
    "0"
}

#[put("/prepare", format = "application/json", data = "<input>")]
fn prepare(input: Json<PrepareMessage>) -> Result<(), http::Status> {
    unimplemented!()
}

#[put("/prepareAccept", format = "application/json", data = "<input>")]
fn prepare_accept(
    input: Json<PrepareAcceptMessage>,
) -> Result<(), http::Status> {
    unimplemented!()
}

#[put("/resolved", format = "application/json", data = "<input>")]
fn resolved(input: Json<ResolvedMessage>) -> Result<(), http::Status> {
    unimplemented!()
}


fn main() {
    let matches = App::new("transaction")
        .arg(Arg::with_name("txfile")
            .short("f")
            .long("tx-file")
            .value_name("FILE")
            .default_value("/tmp/tx_log.txt")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .default_value("8000")
            .takes_value(true))
        .arg(Arg::with_name("joincluster")
            .short("j")
            .long("join-cluster")
            .value_name("ADDRESS:PORT")
            .takes_value(true))
        .arg(Arg::with_name("name")
            .short("n")
            .long("name")
            .value_name("NAME")
            .takes_value(true))
        .get_matches();

    let mut settings = SettingsState::default();
    settings.tx_log_file = matches.value_of("txfile").unwrap().into();
    println!("Settings:\n{:#?}", &settings);
    {
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .open(settings.clone().tx_log_file)
            .unwrap();
    }
    let log = DualLog::load(settings.clone().tx_log_file).unwrap();

    let config = Config::build(Environment::Development)
        .port(matches.value_of("port").unwrap().parse().unwrap())
        .unwrap();

    rocket::custom(config, true)
        .manage(TransactionLogState(Mutex::new(log)))
        .manage(ClientListState(Mutex::new(ClientListHandler::new())))
        .manage(settings)
        .mount(
            "/transactions",
            routes![
                read_all_transactions,
                read_last_transaction,
                read_transaction,
                write_transaction
            ],
        )
        .mount(
            "/",
            routes![
                register_client,
                unregister_client,
                list_clients,
                ping,
                prepare,
                prepare_accept,
                resolved
            ],
        )
        .launch();
}
