use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;
use std::borrow::Cow;

use crate::base_setup::init::ExpectedLogs;

pub fn assert_expected_logs(logs: Vec<Log>, expected_logs: Vec<ExpectedLogs>) {
    for expected_log in expected_logs {
        let matching_logs: Vec<&Log> = logs
            .iter()
            .filter(|log| log.endpoint == expected_log.identifier.as_ref())
            .collect();
        assert!(
            !matching_logs.is_empty(),
            "Expected log '{}' not found. Logs: {:?}",
            expected_log.identifier.as_ref(),
            logs
        );
        if let OptionalValue::Some(ref topics) = expected_log.topics {
            validate_expected_topics(topics, &matching_logs, expected_log.identifier.as_ref());

            if let OptionalValue::Some(data) = expected_log.data {
                if topics.is_empty() {
                    panic!(
                        "Expected at least one topic for data validation in log '{}', but got none. Logs: {:?}",
                        expected_log.identifier,
                        logs
                    );
                }
                let first_topic_bytes = topics[0].as_bytes().to_vec();
                let filtered_logs: Vec<&Log> = matching_logs
                    .iter()
                    .copied()
                    .filter(|log| {
                        log.topics
                            .first()
                            .map(|t| {
                                // Check raw bytes match (blackbox scenario format)
                                if *t == first_topic_bytes {
                                    return true;
                                }
                                // Check if log topic, when decoded from base64, matches (chain simulator format)
                                BASE64_STANDARD
                                    .decode(t)
                                    .map(|decoded| decoded == first_topic_bytes)
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false)
                    })
                    .collect();
                validate_expected_data(
                    &[data.as_ref()],
                    &filtered_logs,
                    expected_log.identifier.as_ref(),
                );
            }
        }
    }
}

pub fn validate_expected_topics(topics: &[Cow<'_, str>], matching_logs: &[&Log], endpoint: &str) {
    let expected_topics_bytes: Vec<Vec<u8>> =
        topics.iter().map(|s| s.as_bytes().to_vec()).collect();

    assert!(
        matching_logs.iter().any(|log| {
            expected_topics_bytes.iter().all(|expected_topic| {
                // Check raw bytes match (blackbox scenario format)
                if log.topics.contains(expected_topic) {
                    return true;
                }

                // Check if any log topic, when decoded from base64, matches (chain simulator format)
                log.topics.iter().any(|log_topic| {
                    BASE64_STANDARD
                        .decode(log_topic)
                        .map(|decoded| decoded == *expected_topic)
                        .unwrap_or(false)
                })
            })
        }),
        "Expected topics '{}' not found for event '{}' \n Logs: {:?}",
        topics
            .iter()
            .map(|topic| topic.as_ref())
            .collect::<Vec<_>>()
            .join(", "),
        endpoint,
        matching_logs
    );
}

pub fn validate_expected_data(data: &[&str], matching_logs: &[&Log], endpoint: &str) {
    let expected_data_bytes: Vec<Vec<u8>> = data.iter().map(|s| s.as_bytes().to_vec()).collect();
    assert!(
        matching_logs
            .iter()
            .any(|log| log_contains_expected_data(log, &expected_data_bytes)),
        "Expected data '{}' not found for event '{}'. Logs: {:?}",
        data.join(", "),
        endpoint,
        matching_logs
    );
}

pub fn log_contains_expected_data(log: &Log, expected_data_bytes: &[Vec<u8>]) -> bool {
    expected_data_bytes.iter().all(|expected_data| {
        log.data.iter().any(|log_data| {
            // Check raw bytes match (blackbox scenario format)
            if log_data
                .windows(expected_data.len())
                .any(|window| window == expected_data)
            {
                return true;
            }

            // Check if any log data, when decoded from base64, matches (chain simulator format)
            if let Ok(decoded_data) = BASE64_STANDARD.decode(log_data) {
                decoded_data
                    .windows(expected_data.len())
                    .any(|window| window == expected_data)
            } else {
                false
            }
        })
    })
}
