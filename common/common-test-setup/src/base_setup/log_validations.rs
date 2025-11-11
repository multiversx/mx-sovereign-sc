use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use error_messages::ALL_ERROR_MESSAGES;
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;

use crate::base_setup::init::ExpectedLogs;

pub fn assert_expected_logs(logs: Vec<Log>, expected_logs: Vec<ExpectedLogs>) {
    let should_check_for_errors =
        expected_logs
            .iter()
            .all(|expected_log| match expected_log.data {
                OptionalValue::Some(data) => !ALL_ERROR_MESSAGES.contains(&data),
                OptionalValue::None => true,
            });

    if should_check_for_errors {
        assert_logs_do_not_contain_error_messages(&logs);
    }

    for expected_log in expected_logs {
        let matching_logs: Vec<&Log> = logs
            .iter()
            .filter(|log| log.endpoint == expected_log.identifier)
            .collect();
        assert!(
            !matching_logs.is_empty(),
            "Expected log '{}' not found. Logs: {:?}",
            expected_log.identifier,
            logs
        );
        if let OptionalValue::Some(ref topics) = expected_log.topics {
            validate_expected_topics(topics, &matching_logs, expected_log.identifier);

            if let OptionalValue::Some(data) = expected_log.data {
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
                validate_expected_data(&[data], &filtered_logs, expected_log.identifier);
            }
        }
    }
}

pub fn validate_expected_topics(topics: &[&str], matching_logs: &[&Log], endpoint: &str) {
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
        topics.join(", "),
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

pub fn assert_logs_do_not_contain_error_messages(logs: &[Log]) {
    for error_message in ALL_ERROR_MESSAGES {
        let error_bytes = error_message.as_bytes();

        let contains_error = logs.iter().any(|log| {
            log.endpoint.contains(error_message)
                || log
                    .topics
                    .iter()
                    .any(|topic| contains_bytes_or_base64(topic, error_bytes))
                || log
                    .data
                    .iter()
                    .any(|data| contains_bytes_or_base64(data, error_bytes))
        });

        assert!(
            !contains_error,
            "Unexpected error message '{}' found in logs: {:?}",
            error_message, logs
        );
    }
}

fn contains_bytes_or_base64(source: &[u8], needle: &[u8]) -> bool {
    if source.windows(needle.len()).any(|window| window == needle) {
        return true;
    }

    if let Ok(decoded) = BASE64_STANDARD.decode(source) {
        return decoded.windows(needle.len()).any(|window| window == needle);
    }

    false
}
