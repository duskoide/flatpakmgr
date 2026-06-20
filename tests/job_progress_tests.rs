use flatpakmgr::flatpak_service::parse::parse_progress_line;

#[test]
fn parse_full_progress() {
    let line =
        "[####################] 100% Downloading: 198.2 MB/198.2 MB (12.3 MB/s)";
    assert_eq!(parse_progress_line(line), Some(100));
}

#[test]
fn parse_partial_progress() {
    let line =
        "[##########          ]  50% Downloading: 42.3 MB/84.6 MB (8.1 MB/s)";
    assert_eq!(parse_progress_line(line), Some(50));
}

#[test]
fn parse_no_progress() {
    let line = "Installing: app.zen_browser.zen/x86_64/stable from flathub";
    assert_eq!(parse_progress_line(line), None);
}
