use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;
use std::borrow::Cow;

use crate::{base_setup::init::ExpectedLogs, constants::INTERNAL_VM_ERRORS};

pub fn assert_expected_logs(logs: Vec<Log>, expected_logs: Vec<ExpectedLogs>) {
    for expected_log in expected_logs {
        let identifier = expected_log.identifier.as_ref();
        let matching_logs: Vec<&Log> = logs
            .iter()
            .filter(|log| log.endpoint == identifier)
            .collect();
        assert!(
            !matching_logs.is_empty(),
            "Expected log '{}' not found. Logs: {:?}",
            identifier,
            logs
        );
        if let OptionalValue::Some(ref topics) = expected_log.topics {
            let expected_data = optional_value_to_str(&expected_log.data);

            validate_expected_topics(topics, &matching_logs, identifier);
            validate_logs_with_topics(&logs, &matching_logs, topics, expected_data, identifier);
        } else if expected_log.data.is_some() {
            panic!(
                "Expected data for log '{}' but no topics provided. Logs: {:?}",
                identifier, logs
            );
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

fn validate_internal_vm_logs(
    logs: &[Log],
    expected_topic: &[u8],
    topic_display: &str,
    expected_data: Option<&str>,
    endpoint: &str,
) {
    let all_internal_vm_logs: Vec<&Log> = logs
        .iter()
        .filter(|log| log.endpoint == INTERNAL_VM_ERRORS)
        .collect();

    let matching_internal_vm_logs: Vec<&Log> = all_internal_vm_logs
        .iter()
        .copied()
        .filter(|log| log_has_first_topic(log, expected_topic))
        .collect();

    match expected_data {
        Some(data) => {
            assert!(
                !matching_internal_vm_logs.is_empty(),
                "Expected internal VM error log with topic '{}' and data '{}' while validating event '{}' but none found. Logs: {:?}",
                topic_display,
                data,
                endpoint,
                logs
            );
            validate_expected_data(&[data], &matching_internal_vm_logs, INTERNAL_VM_ERRORS);
        }
        None => {
            assert!(
                matching_internal_vm_logs.is_empty(),
                "Unexpected internal VM error log with topic '{}' found while validating event '{}'. Logs: {:?}",
                topic_display,
                endpoint,
                logs
            );
        }
    }
}

fn log_has_first_topic(log: &Log, expected_topic: &[u8]) -> bool {
    log.topics
        .first()
        .map(|topic| topic_matches(topic, expected_topic))
        .unwrap_or(false)
}

fn topic_matches(log_topic: &[u8], expected_topic: &[u8]) -> bool {
    if log_topic == expected_topic {
        return true;
    }

    BASE64_STANDARD
        .decode(log_topic)
        .map(|decoded| decoded == expected_topic)
        .unwrap_or(false)
}

fn validate_logs_with_topics(
    all_logs: &[Log],
    matching_logs: &[&Log],
    topics: &[Cow<'_, str>],
    expected_data: Option<&str>,
    endpoint: &str,
) {
    let Some(first_topic) = topics.first() else {
        assert!(
            expected_data.is_none(),
            "Expected at least one topic for data validation in log '{}', but got none. Logs: {:?}",
            endpoint,
            all_logs
        );
        return;
    };

    let first_topic_bytes = first_topic.as_bytes();

    if let Some(data) = expected_data {
        let filtered_logs = filter_logs_by_first_topic(matching_logs, first_topic_bytes);
        validate_expected_data(&[data], &filtered_logs, endpoint);
    }

    validate_internal_vm_logs(
        all_logs,
        first_topic_bytes,
        first_topic.as_ref(),
        expected_data,
        endpoint,
    );
}

fn filter_logs_by_first_topic<'a>(logs: &[&'a Log], expected_topic: &[u8]) -> Vec<&'a Log> {
    logs.iter()
        .copied()
        .filter(|log| log_has_first_topic(log, expected_topic))
        .collect()
}

fn optional_value_to_str<'a>(optional_value: &'a OptionalValue<Cow<'a, str>>) -> Option<&'a str> {
    match optional_value {
        OptionalValue::Some(value) => Some(value.as_ref()),
        OptionalValue::None => None,
    }
}
