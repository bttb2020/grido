pub use makepad_widgets;

use makepad_widgets::*;
use std::path::PathBuf;

use grido_core::{CellValue, SortOrder};
use grido_grid::{CellPos, GridViewState, Selection};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1200, 800)
                window.title: "Grido"
                body +: {
                    View{
                        width: Fill
                        height: Fill
                        flow: Down

                        // 顶部公式栏
                        View{
                            width: Fill
                            height: 32
                            flow: Right
                            padding: {left: 8, right: 8}
                            align: {y: 0.5}
                            draw_bg.color: #x2a2a2a

                            formula_label := Label{
                                text: ""
                                draw_text.text_style.font_size: 11
                                draw_text.color: #xaaa
                            }
                        }

                        // 主网格区域 — 欢迎页或网格
                        welcome_view := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            align: Center
                            padding: 40
                            draw_bg.color: #x1e1e1e

                            Label{
                                text: "Grido"
                                draw_text.text_style.font_size: 32
                                draw_text.color: #xddd
                            }
                            Label{
                                text: "GPU-accelerated spreadsheet editor"
                                draw_text.text_style.font_size: 14
                                draw_text.color: #x888
                                margin: {top: 8}
                            }
                            Label{
                                text: "Drop a CSV file here or run: grido <file.csv>"
                                draw_text.text_style.font_size: 12
                                draw_text.color: #x666
                                margin: {top: 16}
                            }
                        }

                        // 底部状态栏
                        View{
                            width: Fill
                            height: 24
                            flow: Right
                            padding: {left: 8, right: 8}
                            align: {y: 0.5}
                            draw_bg.color: #x252525

                            status_label := Label{
                                text: "Ready"
                                draw_text.text_style.font_size: 10
                                draw_text.color: #x888
                            }

                            Spacer{width: Fill}

                            stats_label := Label{
                                text: ""
                                draw_text.text_style.font_size: 10
                                draw_text.color: #x888
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    state: GridViewState,
    #[rust]
    file_path: Option<PathBuf>,
}

impl MatchEvent for App {
    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        if self.state.document.is_none() {
            return;
        }

        let is_cmd = event.modifiers.logo || event.modifiers.control;
        let is_shift = event.modifiers.shift;

        match event.key_code {
            KeyCode::ArrowUp => {
                if !self.state.editing {
                    self.state.move_cursor(-1, 0, is_shift);
                    self.update_ui(cx);
                }
            }
            KeyCode::ArrowDown => {
                if !self.state.editing {
                    self.state.move_cursor(1, 0, is_shift);
                    self.update_ui(cx);
                }
            }
            KeyCode::ArrowLeft => {
                if !self.state.editing {
                    if is_cmd {
                        let row = self.state.selection.cursor.row;
                        self.state.selection = Selection::single(row, 0);
                        self.state.viewport.ensure_visible(row, 0);
                    } else {
                        self.state.move_cursor(0, -1, is_shift);
                    }
                    self.update_ui(cx);
                }
            }
            KeyCode::ArrowRight => {
                if !self.state.editing {
                    if is_cmd {
                        let row = self.state.selection.cursor.row;
                        let max_col = self.state.document.as_ref()
                            .map(|d| d.col_count().saturating_sub(1))
                            .unwrap_or(0);
                        self.state.selection = Selection::single(row, max_col);
                        self.state.viewport.ensure_visible(row, max_col);
                    } else {
                        self.state.move_cursor(0, 1, is_shift);
                    }
                    self.update_ui(cx);
                }
            }
            KeyCode::Tab => {
                if self.state.editing {
                    self.state.commit_edit();
                }
                self.state.move_cursor(0, if is_shift { -1 } else { 1 }, false);
                self.update_ui(cx);
            }
            KeyCode::ReturnKey => {
                if self.state.editing {
                    self.state.commit_edit();
                    self.state.move_cursor(if is_shift { -1 } else { 1 }, 0, false);
                } else {
                    self.state.start_editing();
                }
                self.update_ui(cx);
            }
            KeyCode::Escape => {
                if self.state.editing {
                    self.state.cancel_edit();
                    self.update_ui(cx);
                }
            }
            KeyCode::F2 => {
                if !self.state.editing {
                    self.state.start_editing();
                    self.update_ui(cx);
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if self.state.editing {
                    self.state.edit_buffer.pop();
                    self.update_ui(cx);
                } else {
                    self.state.delete_selection();
                    self.update_ui(cx);
                }
            }
            KeyCode::KeyZ if is_cmd => {
                if is_shift {
                    self.state.redo();
                } else {
                    self.state.undo();
                }
                self.update_ui(cx);
            }
            KeyCode::KeyS if is_cmd => {
                self.save_file(cx);
            }
            KeyCode::KeyA if is_cmd => {
                if let Some(doc) = &self.state.document {
                    let max_row = doc.row_count().saturating_sub(1);
                    let max_col = doc.col_count().saturating_sub(1);
                    self.state.selection = Selection::range(
                        CellPos::new(0, 0),
                        CellPos::new(max_row, max_col),
                    );
                    self.update_ui(cx);
                }
            }
            KeyCode::PageUp => {
                let visible = self.state.viewport.visible_row_count();
                self.state.move_cursor(-(visible as isize), 0, is_shift);
                self.update_ui(cx);
            }
            KeyCode::PageDown => {
                let visible = self.state.viewport.visible_row_count();
                self.state.move_cursor(visible as isize, 0, is_shift);
                self.update_ui(cx);
            }
            KeyCode::Home => {
                if is_cmd {
                    self.state.selection = Selection::single(0, 0);
                    self.state.viewport.ensure_visible(0, 0);
                } else {
                    let row = self.state.selection.cursor.row;
                    self.state.selection = Selection::single(row, 0);
                    self.state.viewport.ensure_visible(row, 0);
                }
                self.update_ui(cx);
            }
            KeyCode::End => {
                if let Some(doc) = &self.state.document {
                    let max_row = doc.row_count().saturating_sub(1);
                    let max_col = doc.col_count().saturating_sub(1);
                    if is_cmd {
                        self.state.selection = Selection::single(max_row, max_col);
                        self.state.viewport.ensure_visible(max_row, max_col);
                    } else {
                        let row = self.state.selection.cursor.row;
                        self.state.selection = Selection::single(row, max_col);
                        self.state.viewport.ensure_visible(row, max_col);
                    }
                }
                self.update_ui(cx);
            }
            _ => {}
        }
    }

    fn handle_text_input(&mut self, cx: &mut Cx, event: &TextInputEvent) {
        if self.state.document.is_none() {
            return;
        }

        if !self.state.editing && !event.input.is_empty() {
            let c = event.input.chars().next().unwrap_or('\0');
            if !c.is_control() {
                self.state.edit_buffer = event.input.clone();
                self.state.editing = true;
                self.update_ui(cx);
                return;
            }
        }

        if self.state.editing {
            self.state.edit_buffer.push_str(&event.input);
            self.update_ui(cx);
        }
    }

    fn handle_startup(&mut self, cx: &mut Cx) {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            self.open_file(cx, &path);
        }
    }
}

impl App {
    fn open_file(&mut self, cx: &mut Cx, path: &PathBuf) {
        match grido_io::open_file(path.as_path()) {
            Ok(doc) => {
                self.file_path = Some(path.clone());
                self.state.load_document(doc);
                self.update_ui(cx);
            }
            Err(e) => {
                eprintln!("Error opening file: {e}");
            }
        }
    }

    fn save_file(&mut self, cx: &mut Cx) {
        if let (Some(doc), Some(path)) = (&self.state.document, &self.file_path) {
            match grido_io::save_file(doc, path.as_path()) {
                Ok(()) => {
                    if let Some(doc) = &mut self.state.document {
                        doc.mark_saved();
                    }
                    self.state.modified = false;
                    self.update_ui(cx);
                }
                Err(e) => {
                    eprintln!("Error saving file: {e}");
                }
            }
        }
    }

    fn update_ui(&mut self, cx: &mut Cx) {
        if let Some(doc) = &self.state.document {
            let rows = doc.row_count();
            let cols = doc.col_count();
            let cursor = &self.state.selection.cursor;
            let modified_mark = if self.state.modified { " ●" } else { "" };

            let status = format!(
                "{}×{}  |  R{}:C{}{}",
                rows, cols, cursor.row + 1, cursor.col + 1, modified_mark
            );
            self.ui.label(id!(status_label)).set_text(cx, &status);

            // 选区统计
            if !self.state.selection.is_single() {
                if let Some(stats) = self.state.selection_stats() {
                    self.ui.label(id!(stats_label)).set_text(cx, &stats.status_text());
                }
            } else {
                self.ui.label(id!(stats_label)).set_text(cx, "");
            }

            // 公式栏
            let cell = doc.cell(cursor.row, cursor.col);
            let preview = if self.state.editing {
                format!("✎ {}", self.state.edit_buffer)
            } else {
                let col_name = doc.column_name(cursor.col).unwrap_or("");
                format!("{}: {}", col_name, cell.display_string())
            };
            self.ui.label(id!(formula_label)).set_text(cx, &preview);
        }

        self.ui.redraw(cx);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
