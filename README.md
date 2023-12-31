# kofr

Kafka connect CLI inspired by [kaf](https://github.com/birdayz/kaf) and kubectl.
Kofr wraps all of [kafka connect REST API](https://docs.confluent.io/platform/current/connect/references/restapi.html) operations.

# Table of Contents

- [Installation](#Installation)
- [Usage](#Usage)
- [Configuration](#Configuration)
- [Contributions](#Contributions)

# Installation

- [Archives of precompiled binary for Linux and Macos are available.](https://github.com/A-Fayez/kofr/releases)

- Using cargo. Requires [rust](https://www.rust-lang.org/tools/install) to be installed on the system.

  ```bash
  $ cargo install kofr
  ```

# Usage

Add a new connect cluster

```bash
$ kofr config add-cluster dev --hosts http://localhost:8083
```

Change context to a cluster:

```bash
$ kofr config use-cluster dev
```

List available clusters

```bash
$ kofr config get-clusters
```

## Connectors operations

List running connectors

```bash
$ kofr ls
 NAME                STATE     TASKS   TYPE     WORKER_ID
 load-kafka-config   RUNNING   1       SOURCE   127.0.1.1:8083
 test-connector      RUNNING   1       SINK     127.0.1.1:8083
```

List current connect cluster status

```bash
$ kofr cluster status
 Current Cluster: dev
 id : "GnW0xXSmqeO-t6CQVTJg"
 ...........................................
 HOST                      STATE
 http://localhost:8083     Online
 http://localhost:8080     Offline
```

Describing a connector

```bash
$ kofr cn describe <connector-name>
{
  "name": "test-connector",
  "config": {
    "name": "test-connector",
    "tasks.max": "1",
    "topics": "test-topic",
    "connector.class": "org.apache.kafka.connect.file.FileStreamSinkConnector"
  },
  "connector": {
    "state": "RUNNING",
    "worker_id": "127.0.1.1:8083"
  },
  "tasks": [
    {
      "id": 0,
      "state": "RUNNING",
      "worker_id": "127.0.1.1:8083"
    }
  ],
  "type": "sink"
}
```

Create a connector from a configuration file, like kubectl.

```bash
$ echo '{"name":"loadd-kafka-config", "config":{"connector.class":"FileStreamSource","file":"config/server.properties","topic":"kafka-config-topic"}}' \
| kofr cn create -f -
```

Edit a running connector config, this will open $EDITOR, similar to kubectl.

```bash
$ kofr cn edit <connector-name>
```

Restarting, pausing and resuming a connector.

```bash
$ kofr cn restart <connector-name>
# alternatively, you can specify tasks options
$ kofr cn restart <connector-name> --include-tasks
$ kofr cn restart <connector-name> --only-failed

$ kofr cn pause <connector-name>
$ kofr cn resume <connector-name>
```

Patch a running connector with new configuration
```bash
$ kofr cn patch test-connector -d '{"file": "config/server.properties","name": "load-kafka-config","connector.class": "FileStreamSource","topic": "kafka-config-topic"'
```

Delete a running connector

```bash
$ kofr cn delete <connector-name>
```

## Tasks operations

List tasks of a running connector

```bash
$ kofr tasks ls test-connector
Active tasks of connector: 'test-connector'
 ID   STATE     WORKER_ID        TRACE
 0    RUNNING   127.0.1.1:8083   -
 1    PAUSED    127.0.1.1:8083   -
```

Restarting a task

```bash
$ kofr task restart sink-connector 0
```

Getting a task status

```bash
$ kofr task status test-connector 0
{
  "config": {
    "task.class": "org.apache.kafka.connect.file.FileStreamSinkTask",
    "topics": "test-topic"
  },
  "status": {
    "id": 0,
    "state": "RUNNING",
    "worker_id": "127.0.1.1:8083"
  }
}
```

## Connect plugins

List installed plugins on the cluster

```bash
$ kofr plugin ls
```

validate a given connector confiugration with a connector plugin.

```bash
$ echo '{"connector.class": "org.apache.kafka.connect.file.FileStreamSinkConnector",
        "tasks.max": "1",
        "topics": "test-topic"
}' | kofr plugin validate-config -f -
```

# Configuration

By default, kofr reads config from `~/.kofr/config` See [examples](https://github.com/A-Fayez/kofr/tree/main/examples) for a basic config file.

# Contributions

I welcome fixes for bugs or better ways of doing things or more importantly, code reviews. Kofr was made by the motivation of solving a problem when having to deal with multiple kafka connect clusters at my work was mundane and more importantly, learning rust wink-wink. I use it personally like I use kubectl or kaf.
See [issues](https://github.com/A-Fayez/kofr/issues) for things I'd like to improve. If you have a question about the codebase or simply want to discuss something feel free to open an issue. See also [kcmockserver](https://github.com/A-Fayez/kcmockserver) which I used in testing kofr. It's incomplete and could use some contributions.

# Related Projects

[kcmockserver](https://github.com/A-Fayez/kcmockserver)
