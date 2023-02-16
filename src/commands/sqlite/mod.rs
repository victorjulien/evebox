// SPDX-FileCopyrightText: (C) 2020 Jason Ish <jason@codemonkey.net>
//
// SPDX-License-Identifier: MIT

use crate::sqlite::{init_event_db, ConnectionBuilder};
use anyhow::Result;
use clap::{ArgMatches, Command, FromArgMatches, IntoApp, Parser, Subcommand};
use rusqlite::params;
use std::fs::File;
use tracing::info;

mod fts;

#[derive(Parser, Debug)]
#[clap(name = "sqlite", about = "SQLite utilities")]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Dump EVE events from database
    Dump {
        /// Filename of SQLite database
        filename: String,
    },
    /// Load an EVE/JSON file
    Load(LoadArgs),
    /// Check, enable, disable FTS
    Fts(FtsArgs),
    /// Run an SQL query
    Query {
        #[clap(value_name = "DB_FILENAME")]
        filename: String,
        sql: String,
    },
}

#[derive(Parser, Debug)]
struct FtsArgs {
    #[clap(subcommand)]
    command: FtsCommand,
}

#[derive(Subcommand, Debug)]
enum FtsCommand {
    /// Enable FTS
    Enable {
        #[clap(long)]
        force: bool,
        #[clap(value_name = "DB_FILENAME")]
        filename: String,
    },
    /// Disable FTS
    Disable {
        #[clap(long)]
        force: bool,
        #[clap(value_name = "DB_FILENAME")]
        filename: String,
    },
    /// Check FTS integrity
    Check {
        #[clap(value_name = "DB_FILENAME")]
        filename: String,
    },
}

#[derive(Debug, Parser)]
struct LoadArgs {
    /// EVE file to load into database
    #[clap(short, long)]
    input: String,
    /// Filename of SQLite database
    filename: String,
}

pub fn command() -> Command<'static> {
    Args::command()
}

pub async fn main(args: &ArgMatches) -> anyhow::Result<()> {
    let args = Args::from_arg_matches(args)?;
    match &args.command {
        Commands::Dump { filename } => dump(filename),
        Commands::Load(args) => load(args),
        Commands::Fts(args) => fts::fts(args),
        Commands::Query { filename, sql } => query(filename, sql),
    }
}

fn dump(filename: &str) -> Result<()> {
    let conn = ConnectionBuilder::filename(Some(filename)).open(false)?;
    let mut st = conn.prepare("select source from events order by timestamp")?;
    let mut rows = st.query([])?;
    while let Some(row) = rows.next()? {
        let source: String = row.get(0)?;
        println!("{source}");
    }
    Ok(())
}

fn load(args: &LoadArgs) -> Result<()> {
    use std::io::{BufRead, BufReader};
    let input = File::open(&args.input)?;
    let reader = BufReader::new(input).lines();
    let mut conn = ConnectionBuilder::filename(Some(&args.filename)).open(true)?;
    init_event_db(&mut conn)?;
    info!("Loading events");
    let mut count = 0;
    let tx = conn.transaction()?;
    {
        let mut st = tx.prepare("insert into events (timestamp, source) values (?, ?)")?;
        for line in reader {
            let line = line?;
            let eve: serde_json::Value = serde_json::from_str(&line)?;
            let timestamp = eve["timestamp"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("no timestamp"))?;
            let timestamp =
                crate::eve::parse_eve_timestamp(timestamp)?.unix_timestamp_nanos() as u64;
            st.execute(params![&timestamp, &line])?;
            count += 1;
        }
    }
    info!("Committing {count} events");
    tx.commit()?;
    Ok(())
}

fn query(filename: &str, sql: &str) -> Result<()> {
    let conn = ConnectionBuilder::filename(Some(filename)).open(false)?;
    let mut st = conn.prepare(sql)?;
    let mut rows = st.query([])?;
    let mut count = 0;
    while let Some(_row) = rows.next()? {
        count += 1;
    }
    println!("Query returned {count} rows");
    Ok(())
}
