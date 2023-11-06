use tempfile::NamedTempFile;

pub fn config_invalid_format() -> NamedTempFile {
    let config_file = tempfile::Builder::new().tempfile().unwrap();
    let config = format!(
        r#"
current-cluster: test
#clusters:
- name: test
"#,
    );

    std::fs::write(config_file.path(), config).unwrap();
    config_file
}

pub fn config_with_one_cluster(cluster: &str, host: &str) -> NamedTempFile {
    let config_file = tempfile::Builder::new().tempfile().unwrap();
    let config = format!(
        r#"
current-cluster: {}
clusters:
- name: {}
  hosts:
  - {}
"#,
        cluster, cluster, host
    );

    std::fs::write(config_file.path(), config).unwrap();
    config_file
}

pub fn config_with_one_cluster_and_no_context(cluster: &str, host: &str) -> NamedTempFile {
    let config_file = tempfile::Builder::new().tempfile().unwrap();
    let config = format!(
        r#"
clusters:
- name: {}
  hosts:
  - {}
"#,
        cluster, host
    );

    std::fs::write(config_file.path(), config).unwrap();
    config_file
}

pub fn config_file_with_multiple_clusters(clusters: Vec<(String, String)>) -> NamedTempFile {
    let config_file = tempfile::Builder::new().tempfile().unwrap();
    let mut config = format!(
        r#"clusters:
"#,
    );

    for cluster in &clusters {
        config.push_str(&format!(
            r#"
- name: {}
  hosts:
  - {}
        "#,
            cluster.0, cluster.1
        ));
    }
    std::fs::write(config_file.path(), config).unwrap();
    config_file
}
