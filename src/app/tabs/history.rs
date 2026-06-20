use crate::app::tabs::TabState;
use crate::flatpak_service::types::HistoryEntry;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct HistoryTab {
    pub items: Vec<HistoryEntry>,
    pub cursor: usize,
    pub loading: bool,
}

impl TabState for HistoryTab {
    fn title(&self) -> &'static str {
        "History"
    }

    fn list_items(&self) -> Vec<Line<'_>> {
        self.items
            .iter()
            .map(|h| {
                Line::from(format!(
                    "{}  {}",
                    h.time.format("%Y-%m-%d %H:%M"),
                    h.ref_
                ))
            })
            .collect()
    }

    fn selected(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.cursor)
        }
    }

    fn select(&mut self, idx: usize) {
        self.cursor = idx.min(self.items.len().saturating_sub(1));
    }

    fn move_cursor(&mut self, delta: isize) {
        let len = self.items.len();
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
