///
/// ```rust
///  use rzmq::validation::validate_socket;
///  assert!(validate_socket("asdasdas".to_string()).is_err());
///  assert!(validate_socket("ipc://socket".to_string()).is_ok());
///  assert!(validate_socket("ipc://socket/in/path".to_string()).is_ok());
///  assert!(validate_socket("ipc:///tmp/socket/in/tmp".to_string()).is_ok());
///  assert!(validate_socket("tcp://localhost:5559".to_string()).is_ok());
///  assert!(validate_socket("tcp://127.0.0.1:666".to_string()).is_ok());
///  assert!(validate_socket("tcp:://not/ok".to_string()).is_err());
/// ```
pub fn validate_socket(input: String) -> Result<(), String> {
    if regex::Regex::new("ipc://.*|tcp://.*").unwrap().is_match(input.as_str())
    {
        Ok(())
    } else {
        Err("Incorrect address".to_string())
    }
}
