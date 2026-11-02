use std::time;

use super::*;

use k8s::serde_json as json;

#[test]
fn nano() {
    let usage = json::from_str::<Usage>(r#"{"cpu":"1234567n","memory":"123Mi"}"#).unwrap();
    assert_eq!(usage.cpu().unwrap(), 0.001234567_f64);
    assert_eq!(usage.memory().unwrap(), 123 * 1024 * 1024);
}

#[test]
fn milli() {
    let usage = json::from_str::<Usage>(r#"{"cpu":"1234567m","memory":"8748Ki"}"#).unwrap();
    assert_eq!(usage.cpu().unwrap(), 1234.567_f64);
    assert_eq!(usage.memory().unwrap(), 8748 * 1024);
}

#[test]
fn zero() {
    let usage = json::from_str::<Usage>(r#"{"cpu":"0","memory":"51428Ki"}"#).unwrap();
    assert_eq!(usage.cpu().unwrap(), 0_f64);
    assert_eq!(usage.memory().unwrap(), 51428 * 1024);
}

#[test]
fn invalid_cpu() {
    let usage = json::from_str::<Usage>(r#"{"cpu":"123t","memory":"125864Ki"}"#).unwrap();
    let err = usage.cpu().unwrap_err();
    println!("{err}");
    assert!(err.to_string().contains("123t"));
}

#[test]
fn invalid_memory() {
    let usage = json::from_str::<Usage>(r#"{"cpu":"123m","memory":"125864Ai"}"#).unwrap();
    let err = usage.memory().unwrap_err();
    println!("{err}");
    assert!(err.to_string().contains("125864Ai"));
}

#[derive(Debug, Serialize, Deserialize)]
struct D {
    #[serde(with = "duration")]
    window: time::Duration,
}

#[test]
fn valid_duration() {
    let d = json::from_str::<D>(r#"{"window":"12.05s"}"#).unwrap();
    assert_eq!(d.window.as_secs_f64(), 12.05);
}

#[test]
fn invalid_duration() {
    json::from_str::<D>(r#"{"window":"12.05a"}"#).unwrap_err();
}

#[test]
fn go_duration_syntax_and_boundaries() {
    // Expected nanoseconds checked against Go's time.ParseDuration.
    let cases = [
        ("0", 0),
        ("+0", 0),
        ("-0s", 0),
        ("+1s", 1_000_000_000),
        (".5s", 500_000_000),
        ("1.s", 1_000_000_000),
        ("1us", 1_000),
        ("1µs", 1_000),
        ("1μs", 1_000),
        ("1h2m3.4s", 3_723_400_000_000),
        ("1.5h30m", 7_200_000_000_000),
        ("1s1h", 3_601_000_000_000),
        ("1s1s", 2_000_000_000),
        ("0.1ns", 0),
        ("0.999999999999999999ns", 1),
        ("0.999999999999999999h", 3_600_000_000_000),
        ("0.0000000001h", 360),
        ("1.0000000001h", 3_600_000_000_360),
        ("9223372036854775807ns", (1 << 63) - 1),
        ("2562047h47m16.854775807s", (1 << 63) - 1),
    ];
    for (text, nanos) in cases {
        let d: D = json::from_value(json::json!({"window": text})).unwrap();
        assert_eq!(d.window, time::Duration::from_nanos(nanos), "{text}");
    }
}

#[test]
fn invalid_windows_return_errors() {
    for text in [
        "",
        "+",
        "-",
        ".s",
        "1..2s",
        "1",
        "00",
        "-1s",
        "-1ns",
        "1s-1s",
        "1s ",
        "1 hour",
        "1d",
        "PT1S",
        "1💥",
        "1é",
        "1.2.3s",
        "9223372036854775808ns",
        "9223372036854775807ns1ns",
        "4611686018427387904ns4611686018427387904ns",
        "99999999999999999999999999h",
    ] {
        assert!(
            json::from_value::<D>(json::json!({"window": text})).is_err(),
            "{text}"
        );
    }
}

#[test]
fn duration_round_trips_at_nanosecond_precision() {
    for nanos in [0, 1, 1_000, 1_001, 1_000_000_001, (1 << 63) - 1] {
        let d = D {
            window: time::Duration::from_nanos(nanos),
        };
        let value = json::to_value(&d).unwrap();
        let round_trip: D = json::from_value(value).unwrap();
        assert_eq!(round_trip.window, d.window);
    }
    for window in [time::Duration::from_nanos(1 << 63), time::Duration::MAX] {
        assert!(json::to_value(D { window }).is_err());
    }
}
