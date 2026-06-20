use flatpakmgr::flatpak_service::types::{AppRef, Installation, Kind};
use flatpakmgr::flatpak_service::parse::{parse_history, parse_list, parse_info, parse_remotes};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/parse_fixtures/{}", name)).unwrap()
}

#[test]
fn parse_list_apps_ok() {
    let text = fixture("list_apps.txt");
    let apps = parse_list(&text, Kind::App).expect("parse apps");
    assert!(!apps.is_empty(), "expected at least one app");
    assert!(apps.iter().all(|a| matches!(a.kind, Kind::App)));
    assert_eq!(apps[0].name, "Zen");
    assert_eq!(apps[0].id, "app.zen_browser.zen");
    assert_eq!(apps[0].installation, flatpakmgr::flatpak_service::types::Installation::System);
    assert!(apps[0].size_bytes > 0);
}

#[test]
fn parse_list_runtimes_ok() {
    let text = fixture("list_runtimes.txt");
    let runtimes = parse_list(&text, Kind::Runtime).expect("parse runtimes");
    assert!(!runtimes.is_empty(), "expected at least one runtime");
    assert!(runtimes.iter().all(|r| matches!(r.kind, Kind::Runtime)));
    assert_eq!(runtimes[0].name, "Freedesktop Platform");
    assert_eq!(runtimes[0].id, "org.freedesktop.Platform");
}

#[test]
fn parse_list_empty_input() {
    let apps = parse_list("", Kind::App).expect("empty input");
    assert!(apps.is_empty());
}

#[test]
fn parse_list_bad_column_count() {
    let err = parse_list("too\tfew", Kind::App).unwrap_err();
    match err {
        flatpakmgr::flatpak_service::FlatpakError::Parse { msg, .. } => {
            assert!(msg.contains("expected 10 or 11 columns"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn parse_info_zen_ok() {
    let text = fixture("info_zen.txt");
    let basic = AppRef {
        name: "Zen".into(),
        description: "".into(),
        id: "app.zen_browser.zen".into(),
        version: "1.21.3b".into(),
        branch: "stable".into(),
        arch: "x86_64".into(),
        origin: "flathub".into(),
        installation: Installation::System,
        size_bytes: 0,
        ref_: "app/app.zen_browser.zen/x86_64/stable".into(),
        kind: Kind::App,
    };
    let detail = parse_info(&text, basic).expect("parse info");
    assert!(!detail.commit.is_empty());
    assert_eq!(detail.runtime.as_deref(), Some("org.freedesktop.Platform/x86_64/25.08"));
    assert_eq!(detail.sdk.as_deref(), Some("org.freedesktop.Sdk/x86_64/25.08"));
    assert_eq!(detail.license.as_deref(), Some("MPL-2.0"));
    assert!(detail.installed_size > 0);
    assert!(detail.subject.contains("Merge pull request"));
    assert!(detail.date.is_some());
}

#[test]
fn parse_remotes_ok() {
    let text = fixture("remotes.txt");
    let remotes = parse_remotes(&text).expect("parse remotes");
    assert!(!remotes.is_empty());
}

#[test]
fn parse_history_ok() {
    let text = fixture("history.txt");
    let entries = parse_history(&text).expect("parse history");
    assert!(!entries.is_empty());
}
