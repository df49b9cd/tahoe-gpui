//! Example: workflow canvas with nodes, edges, and overlay controls.

use gpui::prelude::*;
use gpui::{
    App, Bounds, Entity, FontWeight, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_platform::application;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use tahoe_gpui::workflow::*;

struct WorkflowDemo {
    canvas: Entity<WorkflowCanvas>,
}

impl WorkflowDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let canvas = cx.new(|cx| {
            let mut canvas = WorkflowCanvas::new(cx);

            let node1 = cx.new(|cx| {
                let mut n = WorkflowNode::new(cx, "input", "User Input");
                n.set_position(100.0, 100.0, cx);
                n.add_output_port("output", cx);
                n
            });
            let node2 = cx.new(|cx| {
                let mut n = WorkflowNode::new(cx, "llm", "LLM Process");
                n.set_position(400.0, 100.0, cx);
                n.add_input_port("input", cx);
                n.add_output_port("output", cx);
                n.set_toolbar(
                    || {
                        NodeToolbar::new()
                            .position(ToolbarPosition::Bottom)
                            .child(
                                Button::new("tb-edit")
                                    .icon(Icon::new(IconName::Pencil))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSmall)
                                    .tooltip("Edit"),
                            )
                            .child(
                                Button::new("tb-copy")
                                    .icon(Icon::new(IconName::Copy))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSmall)
                                    .tooltip("Copy"),
                            )
                            .child(
                                Button::new("tb-delete")
                                    .icon(Icon::new(IconName::X))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSmall)
                                    .tooltip("Delete"),
                            )
                    },
                    cx,
                );
                n
            });
            let node3 = cx.new(|cx| {
                let mut n = WorkflowNode::new(cx, "output", "Response");
                n.set_position(700.0, 100.0, cx);
                n.add_input_port("input", cx);
                n
            });

            canvas.add_node(node1, cx);
            canvas.add_node(node2, cx);
            canvas.add_node(node3, cx);

            canvas.add_connection(
                Connection::new(
                    "conn1",
                    PortId::new("input", "output"),
                    PortId::new("llm", "input"),
                ),
                cx,
            );
            canvas.add_connection(
                Connection::new(
                    "conn2",
                    PortId::new("llm", "output"),
                    PortId::new("output", "input"),
                ),
                cx,
            );

            canvas
        });

        Self { canvas }
    }
}

impl Render for WorkflowDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();
        let canvas_entity = self.canvas.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(theme.border)
                    .px(theme.spacing_md)
                    .py(theme.spacing_sm)
                    .child(
                        div()
                            .text_style(TextStyle::Title3, theme)
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text)
                            .child("AI Elements - Workflow Demo"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .relative()
                    .child(canvas_entity.clone())
                    .child({
                        let canvas_zi = canvas_entity.clone();
                        let canvas_zo = canvas_entity.clone();
                        let canvas_fit = canvas_entity.clone();
                        WorkflowControls::new()
                            .on_zoom_in(move |_event, _window, cx| {
                                canvas_zi.update(cx, |c, cx| c.zoom_in(cx));
                            })
                            .on_zoom_out(move |_event, _window, cx| {
                                canvas_zo.update(cx, |c, cx| c.zoom_out(cx));
                            })
                            .on_fit_view(move |opts, _event, window, cx| {
                                let vs = window.viewport_size();
                                let vw = f32::from(vs.width);
                                let vh = f32::from(vs.height);
                                canvas_fit
                                    .update(cx, |c, cx| c.fit_view_with_options(vw, vh, opts, cx));
                            })
                    })
                    .child(
                        WorkflowPanel::new(WorkflowPanelPosition::TopRight).child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("3 nodes"),
                        ),
                    )
                    .child(
                        WorkflowMiniMap::new(canvas_entity.clone())
                            .position(MinimapPosition::BottomLeft),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.set_global(TahoeTheme::dark());
        let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(WorkflowDemo::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
