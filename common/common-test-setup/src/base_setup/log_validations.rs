use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::scenario_model::Log;

use crate::base_setup::init::ExpectedLogs;

pub fn assert_expected_logs(logs: Vec<Log>, expected_logs: Vec<ExpectedLogs>) {
    for expected_log in expected_logs {
        let matching_logs: Vec<&Log> = logs
            .iter()
            .filter(|log| log.endpoint == expected_log.identifier)
            .collect();
        assert!(
            !matching_logs.is_empty(),
            "Expected log '{}' not found",
            expected_log.identifier
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
                            .map(|t| *t == first_topic_bytes)
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
            expected_topics_bytes
                .iter()
                .all(|expected_topic| log.topics.contains(expected_topic))
        }),
        "Expected topics '{}' not found for event '{}'",
        topics.join(", "),
        endpoint
    );
}

pub fn validate_expected_data(data: &[&str], matching_logs: &[&Log], endpoint: &str) {
    let expected_data_bytes: Vec<Vec<u8>> = data.iter().map(|s| s.as_bytes().to_vec()).collect();
    assert!(
        matching_logs
            .iter()
            .any(|log| log_contains_expected_data(log, &expected_data_bytes)),
        "Expected data '{}' not found for event '{}'",
        data.join(", "),
        endpoint
    );
}

pub fn log_contains_expected_data(log: &Log, expected_data_bytes: &[Vec<u8>]) -> bool {
    expected_data_bytes.iter().all(|expected_data| {
        log.data.iter().any(|log_data| {
            log_data
                .windows(expected_data.len())
                .any(|window| window == expected_data)
        })
    })
}
