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
mod routes;

use std::fs::OpenOptions;
use std::sync::Mutex;

use rocket::config::{Config, Environment};

use clap::{App, Arg};

use transaction_log::*;
use client_data::*;
use routes::*;


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
