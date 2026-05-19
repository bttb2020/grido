pub use makepad_widgets;

use makepad_widgets::*;

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
                        align: Center
                        padding: 40

                        Label{
                            text: "Grido"
                            draw_text.text_style.font_size: 32
                        }
                        Label{
                            text: "Drop a CSV file here to open"
                            draw_text.text_style.font_size: 14
                            draw_text.color: #x888
                            margin: {top: 12}
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
}

impl MatchEvent for App {}

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
