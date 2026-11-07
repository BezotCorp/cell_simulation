use fyrox::{
    core::{color::Color, pool::Handle},
    gui::{
        BuildContext, Thickness, UiNode, UserInterface,
        border::BorderBuilder,
        brush::Brush,
        stack_panel::StackPanelBuilder,
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
    },
};

/// Pastille « libellé / valeur ». La valeur n'est renvoyée à l'interface que si son texte change.
pub(crate) struct StatCard {
    root: Handle<UiNode>,
    value: Handle<UiNode>,
    shown: String,
}

impl StatCard {
    pub(crate) fn new(
        ctx: &mut BuildContext,
        label: &str,
        initial: &str,
        color: Color,
        scale: f32,
    ) -> Self {
        let caption = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::left_right(8.0 * scale))
                .with_foreground(Brush::Solid(Color::opaque(141, 166, 174)).into()),
        )
        .with_text(label)
        .with_font_size((10.0 * scale).into())
        .build(ctx)
        .transmute();
        let value: Handle<UiNode> = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::left_right(8.0 * scale))
                .with_foreground(Brush::Solid(color).into()),
        )
        .with_text(initial)
        .with_font_size((17.0 * scale).into())
        .build(ctx)
        .transmute();
        let column =
            StackPanelBuilder::new(WidgetBuilder::new().with_children([caption, value])).build(ctx);
        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(3.0 * scale))
                .with_background(Brush::Solid(Color::opaque(23, 35, 45)).into())
                .with_child(column),
        )
        .with_corner_radius((6.0 * scale).into())
        .build(ctx)
        .transmute();
        Self {
            root,
            value,
            shown: initial.to_owned(),
        }
    }

    pub(crate) fn root(&self) -> Handle<UiNode> {
        self.root
    }

    pub(crate) fn set(&mut self, ui: &UserInterface, text: String) {
        if self.shown != text {
            ui.send(self.value, TextMessage::Text(text.clone()));
            self.shown = text;
        }
    }
}
