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

#[derive(Debug, Deserialize)]
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
