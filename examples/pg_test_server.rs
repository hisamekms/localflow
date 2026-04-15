// Helper binary for e2e tests: starts an embedded PostgreSQL instance,
// creates test databases, and writes the connection URL prefix to a file.
//
// Usage: pg_test_server <url-file> <db-count>

use std::env;
use std::fs;

use postgresql_embedded::PostgreSQL;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: pg_test_server <url-file> <db-count>");
        std::process::exit(1);
    }
    let url_file = &args[1];
    let db_count: usize = args[2].parse().expect("db-count must be a number");

    eprintln!("pg-test-server: setting up embedded PostgreSQL...");
    let mut pg = PostgreSQL::default();
    pg.setup().await.expect("failed to setup embedded PostgreSQL");
    pg.start().await.expect("failed to start embedded PostgreSQL");

    eprintln!("pg-test-server: creating {db_count} databases...");
    for i in 0..db_count {
        let db_name = format!("senko_e2e_{i}");
        pg.create_database(&db_name)
            .await
            .unwrap_or_else(|e| panic!("failed to create database {db_name}: {e}"));
    }

    // Derive the URL prefix: everything up to and including the last '/' before
    // the database name.  Callers append "senko_e2e_{TEST_INDEX}" to form the
    // full connection URL.
    let marker = "URL_MARKER";
    let url_with_marker = pg.settings().url(marker);
    let url_prefix = url_with_marker
        .strip_suffix(marker)
        .expect("URL should end with the database name marker");

    fs::write(url_file, &url_prefix).expect("failed to write URL file");
    eprintln!("pg-test-server: ready ({db_count} databases, URL prefix written to {url_file})");

    // Wait for SIGINT or SIGTERM.
    // IMPORTANT: Do NOT use blocking I/O (e.g., stdin().read()) here —
    // postgresql_embedded needs the tokio runtime to manage the child process.
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = sigterm.recv() => {}
    }

    eprintln!("pg-test-server: shutting down...");
    pg.stop().await.expect("failed to stop embedded PostgreSQL");
}
