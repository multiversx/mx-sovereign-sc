use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;
use std::borrow::Cow;

use crate::{
    base_setup::init::ExpectedLogs,
    constants::{EXECUTED_BRIDGE_OP_EVENT, INTERNAL_VM_ERRORS},
};

pub fn assert_expected_logs(logs: Vec<Log>, expected_logs: Vec<ExpectedLogs>) {
    for expected_log in expected_logs {
        let identifier = expected_log.identifier.as_ref();
        let mut matching_logs: Vec<&Log> = logs
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
            matching_logs = validate_expected_topics(matching_logs, topics, identifier);
        }

        if let OptionalValue::Some(ref data) = expected_log.data {
            validate_expected_data(&matching_logs, data.as_ref(), identifier);
        }

        validate_error_log_if_necessary(&logs, &expected_log);
    }
}

fn validate_expected_topics<'a>(
    logs: Vec<&'a Log>,
    topics: &[Cow<'_, str>],
    endpoint: &str,
) -> Vec<&'a Log> {
    let expected_topics_bytes: Vec<Vec<u8>> = topics
        .iter()
        .map(|topic| topic.as_bytes().to_vec())
        .collect();

    let filtered_logs: Vec<&Log> = logs
        .into_iter()
        .filter(|log| log_contains_all_topics(log, &expected_topics_bytes))
        .collect();

    assert!(
        !filtered_logs.is_empty(),
        "Expected topics '{}' not found for event '{}'. Logs: {:?}",
        topics
            .iter()
            .map(|topic| topic.as_ref())
            .collect::<Vec<_>>()
            .join(", "),
        endpoint,
        filtered_logs
    );

    filtered_logs
}

fn validate_expected_data(logs: &[&Log], expected_data: &str, endpoint: &str) {
    let expected_data_bytes = vec![expected_data.as_bytes().to_vec()];
    assert!(
        logs.iter()
            .any(|log| log_contains_expected_data(log, &expected_data_bytes)),
        "Expected data '{}' not found for event '{}'. Logs: {:?}",
        expected_data,
        endpoint,
        logs
    );
}

fn validate_error_log_if_necessary(all_logs: &[Log], expected_log: &ExpectedLogs) {
    let topics = match expected_log.topics {
        OptionalValue::Some(ref topics) if !topics.is_empty() => topics,
        _ => return,
    };

    if topics[0].as_ref() != EXECUTED_BRIDGE_OP_EVENT {
        return;
    }

    let internal_vm_logs: Vec<&Log> = all_logs
        .iter()
        .filter(|log| log.endpoint == INTERNAL_VM_ERRORS)
        .collect();

    match expected_log.data {
        OptionalValue::Some(ref data) => {
            let expected_topic_bytes = data.as_bytes().to_vec();
            let has_matching_topic = internal_vm_logs.iter().any(|log| {
                log.topics
                    .iter()
                    .any(|topic| topic_matches(topic, &expected_topic_bytes))
            });

            assert!(
                has_matching_topic,
                "Expected internal VM error log containing topic '{}' when validating event '{}'. Logs: {:?}",
                data.as_ref(),
                expected_log.identifier.as_ref(),
                all_logs
            );
        }
        OptionalValue::None => {
            assert!(
                internal_vm_logs.is_empty(),
                "Unexpected internal VM error log found while validating event '{}'. Logs: {:?}",
                expected_log.identifier.as_ref(),
                all_logs
            );
        }
    }
}

fn log_contains_all_topics(log: &Log, expected_topics: &[Vec<u8>]) -> bool {
    expected_topics.iter().all(|expected_topic| {
        log.topics
            .iter()
            .any(|topic| topic_matches(topic, expected_topic))
    })
}

fn log_contains_expected_data(log: &Log, expected_data_bytes: &[Vec<u8>]) -> bool {
    expected_data_bytes.iter().all(|expected_data| {
        log.data
            .iter()
            .any(|log_data| data_matches(log_data, expected_data))
    })
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

fn data_matches(log_data: &[u8], expected_data: &[u8]) -> bool {
    if log_data
        .windows(expected_data.len())
        .any(|window| window == expected_data)
    {
        return true;
    }

    BASE64_STANDARD
        .decode(log_data)
        .map(|decoded| {
            decoded
                .windows(expected_data.len())
                .any(|window| window == expected_data)
        })
        .unwrap_or(false)
}
