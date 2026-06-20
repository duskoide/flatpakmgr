#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Apps,
    Runtimes,
    Remotes,
    History,
    Install,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Apps,
            Tab::Runtimes,
            Tab::Remotes,
            Tab::History,
            Tab::Install,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Apps => "1.Apps",
            Tab::Runtimes => "2.Runtimes",
            Tab::Remotes => "3.Remotes",
            Tab::History => "4.History",
            Tab::Install => "5.Install",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tabs,
    List,
    Detail,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    Help,
    Jobs,
    Confirm(ConfirmAction),
    Permissions { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Uninstall {
        ref_: String,
        inst: crate::flatpak_service::types::Installation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Modal(Modal),
}
