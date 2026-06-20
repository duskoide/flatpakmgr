use crate::app::tabs::TabState;
use crate::flatpak_service::types::SearchHit;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct InstallTab {
    pub query: String,
    pub results: Vec<SearchHit>,
    pub cursor: usize,
    pub loading: bool,
    pub debounce_token: u64,
}

impl TabState for InstallTab {
    fn title(&self) -> &'static str {
        "Install"
    }

    fn list_items(&self) -> Vec<Line<'_>> {
        self.results
            .iter()
            .map(|r| Line::from(format!("{}  {}", r.name, r.id)))
            .collect()
    }

    fn selected(&self) -> Option<usize> {
        if self.results.is_empty() {
            None
        } else {
            Some(self.cursor)
        }
    }

    fn select(&mut self, idx: usize) {
        self.cursor = idx.min(self.results.len().saturating_sub(1));
    }

    fn move_cursor(&mut self, delta: isize) {
        let len = self.results.len();
        if len == 0 {
            self.cursor = 0;
            return;
        }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    fn loading(&self) -> bool {
        self.loading
    }

    fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }
}
