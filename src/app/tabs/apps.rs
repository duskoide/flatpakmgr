use crate::app::tabs::TabState;
use crate::flatpak_service::types::{AppDetail, AppRef, Installation};
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct AppsTab {
    pub items: Vec<AppRef>,
    pub cursor: usize,
    pub detail: Option<AppDetail>,
    pub detail_loading: bool,
    pub loading: bool,
    pub filter_text: String,
    pub filter_inst: Option<Installation>,
}

impl AppsTab {
    pub fn selected_ref(&self) -> Option<&AppRef> {
        self.items.get(self.cursor)
    }

    pub fn filtered(&self) -> Vec<&AppRef> {
        self.items
            .iter()
            .filter(|a| {
                self.filter_inst
                    .as_ref()
                    .map_or(true, |i| a.installation == *i)
            })
            .filter(|a| {
                let q = self.filter_text.to_lowercase();
                a.name.to_lowercase().contains(&q) || a.id.to_lowercase().contains(&q)
            })
            .collect()
    }
}

impl TabState for AppsTab {
    fn title(&self) -> &'static str {
        "Apps"
    }

    fn list_items(&self) -> Vec<Line<'_>> {
        self.filtered()
            .iter()
            .map(|a| Line::from(format!("{}  {}", a.name, a.version)))
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
        let len = self.filtered().len();
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
