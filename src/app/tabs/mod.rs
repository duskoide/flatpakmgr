use ratatui::text::Line;

pub mod apps;

pub trait TabState {
    fn title(&self) -> &'static str;
    fn list_items(&self) -> Vec<Line<'_>>;
    fn selected(&self) -> Option<usize>;
    fn select(&mut self, idx: usize);
    fn move_cursor(&mut self, delta: isize);
    fn loading(&self) -> bool;
    fn set_loading(&mut self, loading: bool);
}
