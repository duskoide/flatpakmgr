use flatpakmgr::flatpak_service::FlatpakService;

#[tokio::test]
#[ignore = "requires live flatpak installation"]
async fn smoke_list_apps() {
    let svc = FlatpakService::new();
    let apps = svc.list_apps(None).await.expect("list apps");
    assert!(!apps.is_empty());
}

#[tokio::test]
#[ignore = "requires live flatpak installation"]
async fn smoke_info_zen() {
    let svc = FlatpakService::new();
    let basic = flatpakmgr::flatpak_service::types::AppRef {
        name: "Zen".into(),
        description: "".into(),
        id: "app.zen_browser.zen".into(),
        version: "".into(),
        branch: "stable".into(),
        arch: "x86_64".into(),
        origin: "".into(),
        installation: flatpakmgr::flatpak_service::types::Installation::System,
        size_bytes: 0,
        ref_: "app/app.zen_browser.zen/x86_64/stable".into(),
        kind: flatpakmgr::flatpak_service::types::Kind::App,
    };
    let detail = svc.info(basic).await.expect("info");
    assert!(!detail.commit.is_empty());
}
