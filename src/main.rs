extern crate chrono;
extern crate clap;
//#![feature(plugin)]
//#![plugin(rocket_codegen)]
extern crate rocket;
//extern crate itertools;
extern crate sha2;
//#[macro_use]
//extern crate quick_error;


mod transaction;
use transaction::{
        Transaction,
        TransactionId,
        TransactionTime,
        TransactionData,
        TransactionHash};

/*
use std::sync::{Arc, Mutex};

use clap::{App, Arg};

use rocket::response::status;
use rocket::State;
use rocket::http::Status;

use chrono::TimeZone;

use transaction_log::*;
use transaction::Transaction;

static BASE_URL: &'static str = "http://localhost";

#[derive(Debug)]
struct TransactionLogState {
    log: Mutex<FullTransactionLog>
}


#[get("/")]
fn read_all_transactions(tx_log: State<TransactionLogState>) -> String {
    let res = itertools::join(
        tx_log.log.lock().unwrap().all_transactions()
            .unwrap()
            .into_iter()
            .map(|t| t.to_string()),
        "\n");
    res
}

#[get("/<id>")]
fn read_transaction(id: u32, tx_log: State<TransactionLogState>) -> Option<String> {
    tx_log.log.lock().unwrap().all_transactions()
        .unwrap()
        .into_iter()
        .filter(|ref t| t.id() == id)
        .next()
        .map(|t| t.to_string())
}

#[put("/", data="<input>")]
fn write_transaction(input: String, tx_log: State<TransactionLogState>)
        -> Result<status::Created<String>, status::Custom<String>> {
    let error_status = status::Custom(Status::BadRequest, "Why tho".to_owned());
    let mut parts = input.split(";");
    let mut builder = Transaction::build();

    println!("got {}", &input);

    builder = builder.with_id(tx_log.log.lock().unwrap().next_id().unwrap());
    builder = builder.with_timestamp(
        chrono::FixedOffset::east(Transaction::TZ_OFFSET)
            .datetime_from_str(
                parts.next().ok_or(error_status.clone())?,
                "%d%m%y-%H:%M:%S",
            ).map_err(|_| error_status.clone())?
    );
    println!("ts ok");
    builder = builder.with_group_id(
        parts.next()
            .ok_or(error_status.clone())?
            .parse()
            .map_err(|_| error_status.clone())?
    );
    println!("gid ok");
    builder = builder.with_process_id(
        parts.next()
            .ok_or(error_status.clone())?
            .parse()
            .map_err(|_| error_status.clone())?
    );
    println!("pid ok");
    builder = builder.with_text(
        parts.next()
            .ok_or(error_status.clone())?
            .to_owned()
    );
    println!("txt ok");

    let tx = builder.try_finish_with_log(&*tx_log.log.lock().unwrap())
        .map_err(|_| status::Custom(Status::InternalServerError, "Its me".to_owned()))?;
    println!("creating {}", tx.to_string());

    tx_log.log.lock().unwrap().append(tx.clone());


    Ok(status::Created(
        format!("{}/transactions/{}", BASE_URL, tx.id()),
        Some(tx.to_string()))
    )
}

fn main() {

    let mut log = FullTransactionLog::new();
    let tx = Transaction::build()
            .with_id(log.next_id().unwrap())
            .with_current_timestamp()
            .with_group_id(1)
            .with_process_id(2)
            .with_text("hello".into())
            .try_finish_with_log(&log)
            .unwrap();
    log.append(tx);
    rocket::ignite()
        .manage(TransactionLogState { log: Mutex::new(log) })
        .mount("/transactions",
            routes![read_all_transactions, read_transaction, write_transaction])
        .launch();
}

*/

fn main() {

}