#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to trancelatorRT.", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! Welcome to trancelatorRT.");
    }
}
