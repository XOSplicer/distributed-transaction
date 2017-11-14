#![feature(plugin)]

#![plugin(rocket_codegen)]
extern crate rocket;
extern crate chrono;
//extern crate clap;
extern crate itertools;
extern crate sha2;
//#[macro_use]
//extern crate quick_error;

mod transaction;
mod transaction_log;

use std::sync::Mutex;

use rocket::response::status;
use rocket::State;
use rocket::http;

use transaction::{TransactionData, TransactionTime};
use transaction_log::*;

static BASE_URL: &'static str = "http://localhost";

#[derive(Debug)]
struct TransactionLogState(Mutex<FullTransactionLog>);

#[get("/")]
fn read_all_transactions(
    tx_log: State<TransactionLogState>,
) -> Result<String, http::Status> {
    Ok(itertools::join(
        tx_log.0
            .lock()
            .map_err(|_| http::Status::InternalServerError)?
            .get_all()
            .map_err(|_| http::Status::InternalServerError)?
            .iter()
            .map(|t| t.to_string())
        , "\n"
    ))
}

#[get("/<id>")]
fn read_transaction(
    id: u32,
    tx_log: State<TransactionLogState>,
) -> Result<Option<String>, http::Status> {
    Ok(tx_log.0
        .lock()
        .map_err(|_| http::Status::InternalServerError)?
        .get_by_id(id)
        .map_err(|_| http::Status::InternalServerError)?
        .map(|t| t.to_string())
    )
}

// example: $ curl -X PUT -d '020217-12:00:00;05;06;hello world' \
// http://localhost:8000/transactions/ -v
#[put("/", data = "<input>")]
fn write_transaction(
    input: String,
    tx_log: State<TransactionLogState>,
) -> Result<status::Created<String>, status::Custom<String>> {
    let mut parts = input.split(";");

    let time: TransactionTime = parts.next()
        .ok_or(
            status::Custom(http::Status::BadRequest, "No timestamp given".into()
        ))?
        .parse()
        .map_err(|e|
            status::Custom(http::Status::BadRequest, format!("{:?}", e))
        )?;

    let data: TransactionData = itertools::join(parts, ";")
        .parse()
        .map_err(|e|
            status::Custom(http::Status::BadRequest, format!("{:?}", e))
        )?;

    let tx = tx_log.0
        .lock()
        .map_err(|_| status::Custom(http::Status::InternalServerError, "".into()))?
        .create(data, Some(time))
        .map_err(|_| status::Custom(http::Status::InternalServerError, "".into()))?;

    Ok(status::Created(
        format!("{}/transactions/{}", BASE_URL, tx.id().inner()),
        Some(tx.to_string()))
    )
}


fn main() {
    let mut log = FullTransactionLog::new();
    log.create(
        TransactionData::new(0,0, "example").unwrap(),
        None
    ).unwrap();

    rocket::ignite()
        .manage(TransactionLogState(Mutex::new(log)))
        .mount("/transactions",
            routes![
                read_all_transactions,
                read_transaction,
                write_transaction
            ])
        .launch();
}