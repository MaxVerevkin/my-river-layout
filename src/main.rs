use std::collections::HashMap;
use std::convert::Infallible;

use river_layout_toolkit::{run, GeneratedLayout, Layout, Rectangle};

const DEFAULTS: [LayoutKind; 2] = [
    LayoutKind::Tile {
        gaps: 6,
        main_ratio: 0.60,
    },
    LayoutKind::Stack,
];

fn main() {
    run(MyLayout::default()).unwrap();
}

#[derive(Default)]
struct MyLayout {
    overrides: HashMap<(String, u32), [LayoutKind; 2]>,
}

#[derive(Debug, Clone, Copy)]
enum LayoutKind {
    Tile { gaps: u32, main_ratio: f64 },
    Stack,
}

impl LayoutKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Tile { .. } => "[]=",
            Self::Stack => "[[]",
        }
    }
}

impl Layout for MyLayout {
    type Error = Infallible;

    const NAMESPACE: &'static str = "my-layout";

    fn user_cmd(
        &mut self,
        cmd: String,
        tags: Option<u32>,
        output: &str,
    ) -> Result<(), Self::Error> {
        let tag = tags.expect("this river is too old").trailing_zeros();
        let layouts = self
            .overrides
            .entry((output.to_string(), tag))
            .or_insert(DEFAULTS);

        if cmd == "toggle_layout" {
            layouts.rotate_left(1);
        } else if let Some(ratio_delta) = cmd.strip_prefix("inc_main_ratio ") {
            let Ok(ratio_delta) = ratio_delta.parse::<f64>() else { return Ok(()) };
            let LayoutKind::Tile { gaps: _, main_ratio } = layouts.first_mut().unwrap() else { return Ok(()) };
            *main_ratio += ratio_delta;
        }

        Ok(())
    }

    fn generate_layout(
        &mut self,
        view_count: u32,
        usable_width: u32,
        usable_height: u32,
        tags: u32,
        output: &str,
    ) -> Result<GeneratedLayout, Self::Error> {
        let tag = tags.trailing_zeros();
        let layout = self
            .overrides
            .get(&(output.to_string(), tag))
            .unwrap_or(&DEFAULTS)
            .first()
            .unwrap();

        let mut res = GeneratedLayout {
            layout_name: layout.name().to_string(),
            views: Vec::with_capacity(view_count as usize),
        };

        match layout {
            LayoutKind::Tile { gaps, main_ratio } => {
                if view_count == 1 {
                    res.views.push(Rectangle {
                        x: 0,
                        y: 0,
                        width: usable_width,
                        height: usable_height,
                    });
                } else {
                    let main_width = (usable_width as f64 * main_ratio - *gaps as f64 / 2.0) as u32;
                    let stack_width = usable_width - main_width - *gaps;
                    let stack_height =
                        (usable_height - *gaps * (view_count - 2)) / (view_count - 1);
                    res.views.push(Rectangle {
                        x: 0,
                        y: 0,
                        width: main_width,
                        height: usable_height,
                    });
                    for i in 0..(view_count - 1) {
                        res.views.push(Rectangle {
                            x: (main_width + *gaps) as i32,
                            y: ((stack_height + *gaps) * i) as i32,
                            width: stack_width,
                            height: stack_height,
                        });
                    }
                }
            }
            LayoutKind::Stack => {
                let dx = 15;
                let dy = 15;
                let width = usable_width - dx * (view_count - 1);
                let height = usable_height - dy * (view_count - 1);
                for i in 0..view_count {
                    res.views.push(Rectangle {
                        x: (dx * i) as i32,
                        y: (dy * i) as i32,
                        width,
                        height,
                    });
                }
            }
        }

        Ok(res)
    }
}
