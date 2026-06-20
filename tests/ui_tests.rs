use ratatui::{backend::TestBackend, Terminal};
use flatpakmgr::app::App;

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            s.push_str(cell.symbol());
        }
        s.push('\n');
    }
    s
}

#[test]
fn ui_renders_without_panic() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new();

    terminal.draw(|frame| {
        flatpakmgr::ui::draw(frame, &app);
    }).unwrap();

    let buffer = terminal.backend().buffer().clone();
    let content = buffer_to_string(&buffer);
    assert!(content.contains("flatpakmgr"), "should contain title");
    assert!(content.contains("Apps"), "should contain Apps tab");
}

#[test]
fn ui_narrow_terminal_renders() {
    let backend = TestBackend::new(50, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new();

    terminal.draw(|frame| {
        flatpakmgr::ui::draw(frame, &app);
    }).unwrap();

    let buffer = terminal.backend().buffer().clone();
    let content = buffer_to_string(&buffer);
    assert!(!content.trim().is_empty());
}
