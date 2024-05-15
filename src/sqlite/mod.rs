// SPDX-FileCopyrightText: (C) 2020 Jason Ish <jason@codemonkey.net>
// SPDX-License-Identifier: MIT

pub mod builder;
pub mod configrepo;
pub mod connection;
pub mod eventrepo;
pub mod importer;
pub(crate) mod info;
pub mod retention;
pub mod util;

pub(crate) use connection::ConnectionBuilder;
use sqlx::SqliteConnection;
use time::macros::format_description;

pub fn format_sqlite_timestamp(dt: &time::OffsetDateTime) -> String {
    let format =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6][offset_hour sign:mandatory][offset_minute]");
    dt.to_offset(time::UtcOffset::UTC).format(&format).unwrap()
}

pub(crate) async fn has_table(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(name)
            .fetch_one(&mut *conn)
            .await?;
    Ok(count > 0)
}
