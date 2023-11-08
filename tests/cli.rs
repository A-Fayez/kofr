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
fn test_config_file_not_found() {
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!("--config-file=does-not-exist"))
        .arg("ls")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: error reading config file"));
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

#[test]
fn test_kofr_config_current_context_failure() {
    let server = KcTestServer::new();
    let config_file =
        common::config_with_one_cluster_and_no_context("test", &server.base_url().to_string());
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("current-context")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Error: No current context was set",
    ));
}

#[test]
fn test_kofr_config_current_context_success() {
    let server = KcTestServer::new();
    let config_file = common::config_with_one_cluster("test", &server.base_url().to_string());
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("current-context")
    .assert()
    .success()
    .stdout(predicate::str::contains("test"));
}

#[test]
fn test_kofr_config_get_clusters() {
    let test_server = KcTestServer::new();
    let dev_server = KcTestServer::new();
    let test_server = test_server.base_url().to_string();
    let dev_server = dev_server.base_url().to_string();
    let clusters = vec![
        ("test".to_string(), test_server),
        ("dev".to_string(), dev_server),
    ];

    let config_file = common::config_file_with_multiple_clusters(clusters);
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("get-clusters")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        r#"test
dev"#,
    ));
}

#[test]
fn test_kofr_config_get_clusters_are_empty() {
    let config_file = tempfile::Builder::new().tempfile().unwrap();
    std::fs::write(
        config_file.path(),
        r#"
    clusters:
    "#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("get-clusters")
    .assert()
    .success()
    .stdout(predicate::str::contains(""));
}

#[test]
fn test_kofr_cluster_status() {
    let test_server = KcTestServer::new();
    let dev_server = KcTestServer::new();
    let test_server_uri = test_server.base_url().to_string();
    let dev_server_uri = dev_server.base_url().to_string();
    let hosts = vec![test_server_uri, dev_server_uri];

    let config_file = common::config_file_with_one_cluster_multiple_hosts("test", hosts);
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("cluster")
    .arg("status")
    .assert()
    .success()
    .stdout(predicate::str::contains("Online").count(2));

    drop(dev_server);

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("cluster")
    .arg("status")
    .assert()
    .success()
    .stdout(predicate::str::contains("Online").count(1))
    .stdout(predicates::str::contains("Offline").count(1));

    drop(test_server);
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("cluster")
    .arg("status")
    .assert()
    .success()
    .stdout(predicate::str::contains("Offline").count(2));
}

#[test]
fn test_kofr_config_add_cluster() {
    let config_file = common::config_with_one_cluster("dev", "http://localhost:8083/");
    let test_server_1 = KcTestServer::new();
    let test_server_2 = KcTestServer::new();
    let hosts = format!("{},{},", test_server_1.base_url(), test_server_2.base_url());
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("add-cluster")
    .arg("test")
    .arg(format!("--hosts={}", hosts))
    .assert()
    .success()
    .stdout(predicate::str::contains("Added cluster \"test\""));

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("get-clusters")
    .assert()
    .success()
    .stdout(predicate::str::contains("test"));

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("current-context")
    .assert()
    .success()
    .stdout(predicate::str::contains("test"));

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
fn kofr_config_add_cluster_that_already_exists() {
    let config_file = common::config_with_one_cluster("dev", "http://localhost:8083/");
    let dev_server_1 = KcTestServer::new();
    let dev_server_2 = KcTestServer::new();
    let hosts = format!("{},{},", dev_server_1.base_url(), dev_server_2.base_url());
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("add-cluster")
    .arg("dev")
    .arg(format!("--hosts={}", hosts))
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Error: Cluster \"dev\" already exists.",
    ));
}

#[test]
fn kofr_config_delete_cluster() {
    let config_file = common::config_with_one_cluster("dev", "http://localhost:8083/");
    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("remove-cluster")
    .arg("dev")
    .assert()
    .success()
    .stdout(predicate::str::contains("Removed cluster \"dev\""));

    let mut cmd = Command::cargo_bin("kofr").unwrap();
    cmd.arg(format!(
        "--config-file={}",
        config_file.path().to_string_lossy()
    ))
    .arg("config")
    .arg("remove-cluster")
    .arg("dev")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Could not delete cluster: cluster with name 'dev' does not exists",
    ));
}
