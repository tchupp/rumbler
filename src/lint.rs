use std::fs;
use std::path::PathBuf;

use sqruff_lib::core::config::FluffConfig;
use sqruff_lib::core::linter::core::Linter;

use crate::error::RumblerError;

pub fn lint_file(path: &PathBuf) -> Result<bool, RumblerError> {
    let config = FluffConfig::from_source("[sqruff]\ndialect = postgres\n", None);
    let mut linter = Linter::new(config, None, None, false);

    let sql = fs::read_to_string(path)?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();

    let result = linter.lint_string_wrapped(&sql, false);

    let violations = result.violations();
    if !violations.is_empty() {
        log::warn!("{name}:");
        for v in violations {
            log::warn!(
                "  L{}:{} [{}] {}",
                v.line_no,
                v.line_pos,
                v.rule_code(),
                v.desc()
            );
        }
        Ok(false)
    } else {
        log::info!("linting: {name} => OK");

        Ok(true)
    }
}
