mod common;

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use kcmockserver::KcTestServer;

#[test]
fn test_invalid_config_file_format() {
    let config_file = common::config_invalid_format();
    let config_file = config_file.path();
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!("--config-file={}", config_file.to_string_lossy()))
        .arg("ls")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid config file format"));
}

#[test]
fn test_kofr_use_cluster_success() {
    let server = KcTestServer::new();
    let config_file = common::config_with_one_cluster("test", &server.base_url().to_string());

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("use-cluster")
    .arg("test")
    .assert()
    .success()
    .stdout(predicate::str::contains("Switched to cluster \"test\""));
}

#[test]
fn test_kofr_use_cluster_failure() {
    let server = KcTestServer::new();
    let config_file = common::config_with_one_cluster("test", &server.base_url().to_string());

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("use-cluster")
    .arg("dummy")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Error: Cluster with name \"dummy\" could not be found",
    ));
}
