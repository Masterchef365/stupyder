use egui::{Id, Response, TextEdit, Ui};
use egui_extras::syntax_highlighting::{highlight, CodeTheme};

pub fn code_editor_with_autoindent(
    ui: &mut Ui,
    id: Id,
    code: &mut String,
    lang: &'static str,
) -> Response {
    let mut layouter = move |ui: &Ui, string: &str, wrap_width: f32| {
        let mut layout_job = highlight(
            ui.ctx(),
            ui.style(),
            &CodeTheme::from_style(ui.style()),
            string,
            lang,
        );

        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job.clone()))
    };

    let ret = TextEdit::multiline(code)
        .id(id)
        .desired_width(f32::INFINITY)
        .desired_rows(50)
        .code_editor()
        .layouter(&mut layouter)
        .show(ui);

    // Did we make a new line?
    if ret.response.changed() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Some(cursor) = ret.cursor_range {
            let cursor = cursor.primary.ccursor;

            let prev_newline_idx = code[..cursor.index - 1].rfind('\n');

            if cursor.prefer_next_row {
                if let Some(prev) = prev_newline_idx {
                    // Find the indent
                    let indent_chars: String = code[prev..cursor.index]
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .filter(|c| *c == ' ' || *c == '\t')
                        .collect();

                    // Insert indent
                    code.insert_str(cursor.index, &indent_chars);

                    // Set the new cursor pos
                    let mut new_cursor_range = cursor;
                    new_cursor_range.index += indent_chars.len();
                    let mut new_state = ret.state;
                    new_state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(new_cursor_range)));
                    TextEdit::store_state(ui.ctx(), id, new_state);
                }
            }
        }
    }

    ret.response
}
