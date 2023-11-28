use std::io::Write;

use super::Client;

impl Client {
    pub fn write_to_sv(&mut self, line: &str) -> std::io::Result<usize> {
        if let Some(s) = &mut self.stream {
            let _ = s.write(format!("{}\r\n", line).as_bytes())?;
        }
        Ok(0)
    }
}
