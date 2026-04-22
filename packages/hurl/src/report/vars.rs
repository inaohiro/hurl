/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2026 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
//! Variables report in `.env` format.
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::report::ReportError;
use crate::runner::{Value, VariableSet};
use crate::util::path::create_dir_all;

/// Writes variables from `runs` to `filename` in `.env` format.
///
/// Each run's variables are preceded by a comment with the source filename.
/// Variables within each run are sorted alphabetically.
/// Secret variables are written with their actual values (no redaction).
pub fn write_report(
    filename: &Path,
    runs: &[(String, &VariableSet)],
) -> Result<(), ReportError> {
    create_dir_all(filename)
        .map_err(|e| ReportError::from_io_error(&e, filename, "Issue writing vars report"))?;

    let mut content = String::new();
    for (name, variables) in runs.iter() {
        content.push_str(&format!("# {name}\n"));
        let mut pairs: Vec<_> = variables.iter().collect();
        pairs.sort_by_key(|(n, _)| n.as_str());
        for (var_name, variable) in pairs {
            let str_value = value_to_string(variable.value());
            content.push_str(&format!("{}={}\n", var_name, quote_env_value(&str_value)));
        }
        content.push('\n');
    }

    let mut file = File::create(filename)
        .map_err(|e| ReportError::from_io_error(&e, filename, "Issue writing vars report"))?;
    file.write_all(content.as_bytes())
        .map_err(|e| ReportError::from_io_error(&e, filename, "Issue writing vars report"))?;
    Ok(())
}

fn value_to_string(value: &Value) -> String {
    value.render().unwrap_or_else(|| value.to_string())
}

fn quote_env_value(s: &str) -> String {
    if s.contains('"') || s.contains('\\') || s.contains('\n') || s.contains('\r')
        || s.contains(' ') || s.contains('\t')
    {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}
