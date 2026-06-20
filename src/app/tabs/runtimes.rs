use crate::app::tabs::TabState;
use crate::flatpak_service::types::AppRef;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct RuntimesTab {
    pub items: Vec<AppRef>,
    pub cursor: usize,
    pub loading: bool,
}

impl RuntimesTab {
    pub fn selected_ref(&self) -> Option<&AppRef> {
        self.items.get(self.cursor)
    }
}

impl TabState for RuntimesTab {
    fn title(&self) -> &'static str {
        "Runtimes"
    }

    fn list_items(&self) -> Vec<Line<'_>> {
        self.items
            .iter()
            .map(|r| Line::from(format!("{}  {}", r.name, r.version)))
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
