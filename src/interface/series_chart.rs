use crate::ecosystem::PopulationSample;
use fyrox::{
    core::{algebra::Vector2, color::Color, pool::Handle},
    gui::{
        BuildContext, Thickness, UiNode, UserInterface,
        border::BorderBuilder,
        brush::Brush,
        grid::{Column, GridBuilder, Row},
        stack_panel::StackPanelBuilder,
        text::{TextBuilder, TextMessage},
        vector_image::{Primitive, VectorImage, VectorImageBuilder},
        widget::WidgetBuilder,
    },
};

/// Courbes superposées partageant une échelle verticale commune, dessinées en primitives vectorielles.
pub(crate) struct SeriesChart {
    root: Handle<UiNode>,
    guides: Handle<UiNode>,
    lines: Vec<Handle<UiNode>>,
    ceiling: Handle<UiNode>,
    height: f32,
    scale: f32,
    /// Dernier plafond affiché, pour ne renvoyer le libellé que s'il change.
    shown_ceiling: String,
}

impl SeriesChart {
    pub(crate) fn new(
        ctx: &mut BuildContext,
        title: &str,
        series: &[(&str, Color)],
        height: f32,
        scale: f32,
    ) -> Self {
        let label =
            |ctx: &mut BuildContext, text: &str, color: Color, size: f32| -> Handle<UiNode> {
                TextBuilder::new(
                    WidgetBuilder::new()
                        .with_margin(Thickness::uniform(2.0 * scale))
                        .with_foreground(Brush::Solid(color).into()),
                )
                .with_text(text)
                .with_font_size((size * scale).into())
                .build(ctx)
                .transmute()
            };
        let heading = label(ctx, title, Color::opaque(223, 235, 234), 12.0);
        let legend: Vec<_> = series
            .iter()
            .map(|(name, color)| label(ctx, &format!("— {name}"), *color, 11.0))
            .collect();
        let legend = fyrox::gui::wrap_panel::WrapPanelBuilder::new(
            WidgetBuilder::new().with_children(legend),
        )
        .with_orientation(fyrox::gui::Orientation::Horizontal)
        .build(ctx);
        let guides: Handle<UiNode> = VectorImageBuilder::new(
            WidgetBuilder::new().with_foreground(Brush::Solid(Color::opaque(44, 62, 74)).into()),
        )
        .build(ctx)
        .transmute();
        let lines: Vec<Handle<UiNode>> = series
            .iter()
            .map(|(_, color)| {
                VectorImageBuilder::new(
                    WidgetBuilder::new().with_foreground(Brush::Solid(*color).into()),
                )
                .build(ctx)
                .transmute()
            })
            .collect();
        let ceiling = label(ctx, "", Color::opaque(141, 166, 174), 10.0);
        let plot = GridBuilder::new(
            WidgetBuilder::new()
                .with_height(height)
                .with_margin(Thickness::uniform(2.0 * scale))
                .with_child(guides)
                .with_children(lines.iter().copied())
                .with_child(ceiling),
        )
        .add_row(Row::stretch())
        .add_column(Column::stretch())
        .build(ctx);
        let column = StackPanelBuilder::new(WidgetBuilder::new().with_children([
            heading,
            legend.transmute(),
            plot.transmute(),
        ]))
        .build(ctx);
        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(4.0 * scale))
                .with_background(Brush::Solid(Color::opaque(15, 25, 34)).into())
                .with_child(column),
        )
        .with_corner_radius((6.0 * scale).into())
        .build(ctx)
        .transmute();
        Self {
            root,
            guides,
            lines,
            ceiling,
            height,
            scale,
            shown_ceiling: String::new(),
        }
    }

    pub(crate) fn root(&self) -> Handle<UiNode> {
        self.root
    }

    fn set_primitives(ui: &mut UserInterface, handle: Handle<UiNode>, primitives: Vec<Primitive>) {
        if let Some(image) = ui.node_mut(handle).cast_mut::<VectorImage>() {
            image.primitives.set_value_and_mark_modified(primitives);
        }
    }

    /// Redessine les courbes sur une zone de `width` pixels ; `value` extrait de chaque
    /// échantillon la grandeur de la série. Les valeurs inconnues (NaN) interrompent le tracé.
    pub(crate) fn update(
        &mut self,
        ui: &mut UserInterface,
        width: f32,
        samples: &[PopulationSample],
        values: &[fn(&PopulationSample) -> f32],
    ) {
        let height = self.height * self.scale;
        let plot_height = height - 4.0 * self.scale;
        let ceiling = samples
            .iter()
            .flat_map(|sample| values.iter().map(move |value| value(sample)))
            .filter(|value| value.is_finite())
            .fold(1.0_f32, f32::max)
            * 1.1;
        let start = samples.first().map_or(0.0, PopulationSample::time);
        let span = (samples.last().map_or(1.0, PopulationSample::time) - start) as f32;
        let x = |time: f64| {
            if span > 0.0 {
                (time - start) as f32 / span * width
            } else {
                0.0
            }
        };
        let y = |value: f32| 2.0 * self.scale + plot_height * (1.0 - value / ceiling);

        Self::set_primitives(
            ui,
            self.guides,
            (0..=3)
                .map(|step| {
                    let level = plot_height * step as f32 / 3.0 + 2.0 * self.scale;
                    Primitive::Line {
                        begin: Vector2::new(0.0, level),
                        end: Vector2::new(width, level),
                        thickness: self.scale,
                    }
                })
                .collect(),
        );
        for (&handle, value) in self.lines.iter().zip(values) {
            let primitives = samples
                .windows(2)
                .filter(|pair| value(&pair[0]).is_finite() && value(&pair[1]).is_finite())
                .map(|pair| Primitive::Line {
                    begin: Vector2::new(x(pair[0].time()), y(value(&pair[0]))),
                    end: Vector2::new(x(pair[1].time()), y(value(&pair[1]))),
                    thickness: 1.8 * self.scale,
                })
                .collect();
            Self::set_primitives(ui, handle, primitives);
        }
        let label = format!("max {:.0}  ·  {:.0} min simulées", ceiling, span / 60.0);
        if label != self.shown_ceiling {
            ui.send(self.ceiling, TextMessage::Text(label.clone()));
            self.shown_ceiling = label;
        }
    }
}
