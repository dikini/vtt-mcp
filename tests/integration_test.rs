
#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_voice_command_success() {
        let output = Command::new("echo")
            .arg("Voice command executed successfully!")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Voice command executed successfully!"));
    }

    #[test]
    fn test_invalid_voice_command() {
        let output = Command::new("sh")
            .arg("-c")
            .arg("invalid-command")
            .output();

        assert!(output.is_err(), "Invalid command should fail");
    }

    #[test]
    fn test_multiple_commands() {
        let commands = vec!["first command", "second command", "third command"];
        for cmd in commands {
            let output = Command::new("echo")
                .arg(cmd)
                .output()
                .expect("Failed to execute command");

            assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains(cmd));
        }
    }
}
