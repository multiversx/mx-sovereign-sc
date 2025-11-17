use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;
use std::borrow::Cow;

use crate::{
    base_setup::init::ExpectedLogs,
    constants::{
        EXECUTED_BRIDGE_OP_EVENT, EXECUTE_BRIDGE_OPS_ENDPOINT, EXECUTE_OPERATION_ENDPOINT,
        INTERNAL_VM_ERRORS,
    },
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
        } else {
            if identifier != EXECUTE_BRIDGE_OPS_ENDPOINT && identifier != EXECUTE_OPERATION_ENDPOINT
            {
                continue;
            }

            if matches!(&expected_log.topics, OptionalValue::None)
                || matches!(&expected_log.topics, OptionalValue::Some(topics) if topics[0] != EXECUTED_BRIDGE_OP_EVENT)
            {
                continue;
            }

            assert!(
                matching_logs.len() == 1,
                "Expected exactly one log for event '{}', but found {}. Logs: {:?}",
                identifier,
                matching_logs.len(),
                matching_logs
            );

            check_for_internal_vm_error_log(logs.clone(), &expected_log);

            let first_log = matching_logs.first().unwrap();
            let has_data = first_log.data.iter().any(|data| !data.is_empty());
            assert!(
                !has_data,
                "Expected no data (None or empty) for event '{}', but found one. Logs: {:?}",
                identifier, first_log
            );
        }
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

fn check_for_internal_vm_error_log(expected_logs: Vec<Log>, expected_log: &ExpectedLogs) {
    let internal_vm_logs: Vec<Log> = expected_logs
        .into_iter()
        .filter(|log| log.endpoint == INTERNAL_VM_ERRORS)
        .collect();

    assert!(
        internal_vm_logs.is_empty(),
        "Unexpected internal VM error log found while validating event '{}'. Logs: {:?}",
        expected_log.identifier.as_ref(),
        internal_vm_logs
    );
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
