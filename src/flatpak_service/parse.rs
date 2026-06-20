use crate::flatpak_service::types::{AppDetail, AppRef, Installation, Kind, Permission};
use crate::flatpak_service::Result;

pub fn parse_list(input: &str, kind: Kind) -> Result<Vec<AppRef>> {
    let mut out = Vec::new();
    for line in input.lines() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        match cols.len() {
            11 => {
                // Full format with description (apps)
                out.push(AppRef {
                    name: cols[0].to_string(),
                    description: cols[1].to_string(),
                    id: cols[2].to_string(),
                    version: cols[3].to_string(),
                    branch: cols[4].to_string(),
                    arch: cols[5].to_string(),
                    origin: cols[6].to_string(),
                    installation: parse_installation(cols[7])?,
                    size_bytes: parse_size(cols[8])?,
                    ref_: cols[9].to_string(),
                    kind,
                });
            }
            10 => {
                // Compact format without description (runtimes)
                out.push(AppRef {
                    name: cols[0].to_string(),
                    description: String::new(),
                    id: cols[1].to_string(),
                    version: cols[2].to_string(),
                    branch: cols[3].to_string(),
                    arch: cols[4].to_string(),
                    origin: cols[5].to_string(),
                    installation: parse_installation(cols[6])?,
                    size_bytes: parse_size(cols[7])?,
                    ref_: cols[8].to_string(),
                    kind,
                });
            }
            n => {
                return Err(crate::flatpak_service::FlatpakError::Parse {
                    line: line.to_string(),
                    msg: format!("expected 10 or 11 columns, got {}", n),
                });
            }
        }
    }
    Ok(out)
}

fn parse_installation(s: &str) -> Result<Installation> {
    match s {
        "system" => Ok(Installation::System),
        "user" => Ok(Installation::User),
        other => Err(crate::flatpak_service::FlatpakError::Parse {
            line: other.to_string(),
            msg: "expected 'system' or 'user'".to_string(),
        }),
    }
}

fn parse_size(s: &str) -> Result<u64> {
    let trimmed = s.trim();
    if trimmed == "0 bytes" || trimmed.is_empty() {
        return Ok(0);
    }
    let numeric: String = trimmed.chars().filter(|c| c.is_digit(10) || *c == '.').collect();
    let value: f64 = numeric.parse().map_err(|_| crate::flatpak_service::FlatpakError::Parse {
        line: s.to_string(),
        msg: "cannot parse size".to_string(),
    })?;
    if trimmed.contains("GB") {
        Ok((value * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if trimmed.contains("MB") {
        Ok((value * 1024.0 * 1024.0) as u64)
    } else if trimmed.contains("kB") {
        Ok((value * 1024.0) as u64)
    } else {
        Err(crate::flatpak_service::FlatpakError::Parse {
            line: s.to_string(),
            msg: "unrecognized size unit".to_string(),
        })
    }
}

pub fn parse_info(text: &str, basic: AppRef) -> Result<AppDetail> {
    let mut runtime = None;
    let mut sdk = None;
    let mut license = None;
    let mut installed_size = 0u64;
    let mut commit = String::new();
    let mut subject = String::new();
    let mut date: Option<chrono::DateTime<chrono::Utc>> = None;

    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Runtime" => runtime = Some(value.to_string()),
                "Sdk" => sdk = Some(value.to_string()),
                "License" => license = Some(value.to_string()),
                "Installed" => installed_size = parse_size(value).unwrap_or(0),
                "Commit" => commit = value.to_string(),
                "Subject" => subject = value.to_string(),
                "Date" => {
                    date = chrono::DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S %z")
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc));
                }
                _ => {}
            }
        }
    }

    Ok(AppDetail {
        basic,
        runtime,
        sdk,
        license,
        installed_size,
        commit,
        subject,
        date,
        permissions: Vec::new(),
    })
}

pub fn parse_permissions(text: &str) -> Vec<Permission> {
    let mut perms = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((table, rest)) = line.split_once('\t') {
            let entries: Vec<String> = rest.split_whitespace().map(|s| s.to_string()).collect();
            perms.push(Permission {
                table: table.to_string(),
                entries,
            });
        }
    }
    perms
}
