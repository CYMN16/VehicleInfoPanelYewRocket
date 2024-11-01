use std::process::Command;

fn main() {
  Command::new("docker").args(&["start", "db"]).status().unwrap();
}