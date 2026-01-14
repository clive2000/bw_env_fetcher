use crate::bitwarden::Secret;
use crate::error::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::{debug, info, warn};

pub struct EnvWriter;

impl EnvWriter {
    pub fn write_to_stdout(secrets: &[Secret]) -> Result<()> {
        debug!("Writing {} secrets to stdout", secrets.len());

        let stdout = std::io::stdout();
        let mut writer = stdout.lock();

        Self::write_secrets(&mut writer, secrets)?;

        Ok(())
    }

    pub fn write_to_file(secrets: &[Secret], output_path: &Path) -> Result<()> {
        debug!(
            "Writing {} secrets to {}",
            secrets.len(),
            output_path.display()
        );

        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        Self::write_secrets(&mut writer, secrets)?;

        writer.flush()?;

        info!(
            "Successfully wrote {} environment variables to {}",
            secrets.len(),
            output_path.display()
        );

        Ok(())
    }

    fn write_secrets<W: Write>(writer: &mut W, secrets: &[Secret]) -> Result<()> {
        for secret in secrets {
            let sanitized_key = Self::sanitize_env_key(&secret.key);
            let escaped_value = Self::escape_env_value(&secret.value);

            if sanitized_key != secret.key {
                warn!(
                    "Secret key '{}' sanitized to '{}' for env compatibility",
                    secret.key, sanitized_key
                );
            }

            writeln!(writer, "{}={}", sanitized_key, escaped_value)?;
        }

        Ok(())
    }

    fn sanitize_env_key(key: &str) -> String {
        key.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .to_string()
    }

    fn escape_env_value(value: &str) -> String {
        let needs_quoting = value.contains(' ')
            || value.contains('\n')
            || value.contains('\r')
            || value.contains('\t')
            || value.contains('\'')
            || value.contains('"')
            || value.contains('$')
            || value.contains('`')
            || value.contains('\\')
            || value.contains('#')
            || value.is_empty();

        if !needs_quoting {
            return value.to_string();
        }

        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");

        format!("\"{}\"", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_env_key() {
        assert_eq!(EnvWriter::sanitize_env_key("my-key"), "MY_KEY");
        assert_eq!(EnvWriter::sanitize_env_key("123start"), "START");
        assert_eq!(
            EnvWriter::sanitize_env_key("ALREADY_VALID"),
            "ALREADY_VALID"
        );
        assert_eq!(EnvWriter::sanitize_env_key("with spaces"), "WITH_SPACES");
    }

    #[test]
    fn test_escape_env_value() {
        assert_eq!(EnvWriter::escape_env_value("simple"), "simple");
        assert_eq!(EnvWriter::escape_env_value("with space"), "\"with space\"");
        assert_eq!(
            EnvWriter::escape_env_value("has\"quote"),
            "\"has\\\"quote\""
        );
        assert_eq!(EnvWriter::escape_env_value("has$var"), "\"has\\$var\"");
        assert_eq!(EnvWriter::escape_env_value(""), "\"\"");
    }
}
