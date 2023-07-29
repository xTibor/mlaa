use eframe::egui::{self, PointerButton, Sense};
use eframe::emath::{lerp, remap};
use eframe::epaint::{vec2, Color32, Rect, Rgba, Stroke, Vec2};

use itertools::Itertools;

const IMAGE_WIDTH: usize = 32;
const IMAGE_HEIGHT: usize = 24;

pub struct Seam {
    pub x: usize,
    pub y: usize,
    pub length: usize,
    pub color_a: Option<Color32>,
    pub color_b: Option<Color32>,
}

pub struct Gradient {
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub color_a: Color32,
    pub color_b: Color32,
}

struct MlaaApplication {
    selected_color: Color32,
    image_pixels: [[Color32; IMAGE_WIDTH]; IMAGE_HEIGHT],

    show_vertical_seam_outlines: bool,
    show_vertical_gradient_outlines: bool,
    show_vertical_gradients: bool,
    vertical_seams: Vec<Seam>,
    vertical_gradients: Vec<Gradient>,

    show_horizontal_seam_outlines: bool,
    show_horizontal_gradient_outlines: bool,
    show_horizontal_gradients: bool,
    horizontal_seams: Vec<Seam>,
    horizontal_gradients: Vec<Gradient>,
}

impl Default for MlaaApplication {
    fn default() -> MlaaApplication {
        let mut mlaa_application = MlaaApplication {
            selected_color: Color32::BLACK,
            image_pixels: Default::default(),

            show_vertical_seam_outlines: true,
            show_vertical_gradient_outlines: true,
            show_vertical_gradients: false,
            vertical_seams: Vec::new(),
            vertical_gradients: Vec::new(),

            show_horizontal_seam_outlines: true,
            show_horizontal_gradient_outlines: true,
            show_horizontal_gradients: false,
            horizontal_seams: Vec::new(),
            horizontal_gradients: Vec::new(),
        };

        mlaa_application.generate_test_image();
        mlaa_application.recalculate_seams();
        mlaa_application
    }
}

impl MlaaApplication {
    fn generate_test_image(&mut self) {
        let aspect_ratio = if IMAGE_WIDTH > IMAGE_HEIGHT {
            vec2(IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32, 1.0)
        } else {
            vec2(1.0, IMAGE_HEIGHT as f32 / IMAGE_WIDTH as f32)
        };

        for y in 0..IMAGE_HEIGHT {
            for x in 0..IMAGE_WIDTH {
                let v = vec2(
                    remap(x as f32, 0.0..=(IMAGE_WIDTH as f32), -1.0..=1.0),
                    remap(y as f32, 0.0..=(IMAGE_HEIGHT as f32), -1.0..=1.0),
                ) * aspect_ratio;

                self.image_pixels[y][x] = if v.length() <= 0.9 && v.length() >= 0.5 {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };
            }
        }
    }

    fn pixel(&self, x: isize, y: isize) -> Option<Color32> {
        // TODO: Clamp, Wraparound, Extend, etc.

        if (x < 0) || (x >= IMAGE_WIDTH as isize) {
            return None;
        }

        if (y < 0) || (y >= IMAGE_HEIGHT as isize) {
            return None;
        }

        Some(self.image_pixels[y as usize][x as usize])
    }

    fn recalculate_seams(&mut self) {
        self.vertical_seams = (-1..=IMAGE_WIDTH as isize)
            .tuple_windows()
            .flat_map(|(x1, x2)| {
                (0..IMAGE_HEIGHT as isize)
                    .map(|y| (y, self.pixel(x1, y), self.pixel(x2, y)))
                    .dedup_by_with_count(|a, b| (a.1 == b.1) && (a.2 == b.2))
                    .filter(|(_, (_, color_a, color_b))| color_a != color_b)
                    .map(|(length, (y, color_a, color_b))| Seam {
                        x: x2 as usize,
                        y: y as usize,
                        length,
                        color_a,
                        color_b,
                    })
                    .collect_vec()
            })
            .collect_vec();

        self.horizontal_seams = (-1..=IMAGE_HEIGHT as isize)
            .tuple_windows()
            .flat_map(|(y1, y2)| {
                (0..IMAGE_WIDTH as isize)
                    .map(|x| (x, self.pixel(x, y1), self.pixel(x, y2)))
                    .dedup_by_with_count(|a, b| (a.1 == b.1) && (a.2 == b.2))
                    .filter(|(_, (_, color_a, color_b))| color_a != color_b)
                    .map(|(length, (x, color_a, color_b))| Seam {
                        x: x as usize,
                        y: y2 as usize,
                        length,
                        color_a,
                        color_b,
                    })
                    .collect_vec()
            })
            .collect_vec();

        self.vertical_gradients = self
            .vertical_seams
            .iter()
            .map(|seam_a| {
                self.vertical_seams
                    .iter()
                    .find(|seam_b| {
                        ((seam_b.x == seam_a.x + 1) || (seam_b.x + 1 == seam_a.x))
                            && (seam_b.y == seam_a.y + seam_a.length)
                            && (seam_b.color_a == seam_a.color_a)
                            && (seam_b.color_b == seam_a.color_b)
                    })
                    .map(|seam_b| (seam_a, seam_b))
            })
            .flatten()
            .map(|(seam_a, seam_b)| {
                let gradient_y = (seam_a.y as f32) + (seam_a.length as f32 / 2.0);
                let gradient_length = (seam_a.length as f32 / 2.0) + (seam_b.length as f32 / 2.0);

                if seam_a.x < seam_b.x {
                    Gradient {
                        x: seam_a.x as f32,
                        y: gradient_y,
                        length: gradient_length,
                        color_a: seam_a.color_b.or(seam_b.color_b).unwrap(),
                        color_b: seam_a.color_a.or(seam_b.color_a).unwrap(),
                    }
                } else {
                    Gradient {
                        x: seam_b.x as f32,
                        y: gradient_y,
                        length: gradient_length,
                        color_a: seam_a.color_a.or(seam_b.color_a).unwrap(),
                        color_b: seam_a.color_b.or(seam_b.color_b).unwrap(),
                    }
                }
            })
            .collect_vec();

        self.horizontal_gradients = self
            .horizontal_seams
            .iter()
            .map(|seam_a| {
                self.horizontal_seams
                    .iter()
                    .find(|seam_b| {
                        ((seam_b.y == seam_a.y + 1) || (seam_b.y + 1 == seam_a.y))
                            && (seam_b.x == seam_a.x + seam_a.length)
                            && (seam_b.color_a == seam_a.color_a)
                            && (seam_b.color_b == seam_a.color_b)
                    })
                    .map(|seam_b| (seam_a, seam_b))
            })
            .flatten()
            .map(|(seam_a, seam_b)| {
                let gradient_x = (seam_a.x as f32) + (seam_a.length as f32 / 2.0);
                let gradient_length = (seam_a.length as f32 / 2.0) + (seam_b.length as f32 / 2.0);

                if seam_a.y < seam_b.y {
                    Gradient {
                        x: gradient_x,
                        y: seam_a.y as f32,
                        length: gradient_length,
                        color_a: seam_a.color_b.or(seam_b.color_b).unwrap(),
                        color_b: seam_a.color_a.or(seam_b.color_a).unwrap(),
                    }
                } else {
                    Gradient {
                        x: gradient_x,
                        y: seam_b.y as f32,
                        length: gradient_length,
                        color_a: seam_a.color_a.or(seam_b.color_a).unwrap(),
                        color_b: seam_a.color_b.or(seam_b.color_b).unwrap(),
                    }
                }
            })
            .collect_vec();
    }
}

impl eframe::App for MlaaApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.selected_color);
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("New image");

                    if ui.button("Test image").clicked() {
                        self.generate_test_image();
                        self.recalculate_seams();
                    }

                    if ui.button("Blank image").clicked() {
                        self.image_pixels = [[Color32::WHITE; IMAGE_WIDTH]; IMAGE_HEIGHT];
                        self.recalculate_seams();
                    }
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Seam outlines");
                    ui.checkbox(&mut self.show_vertical_seam_outlines, "Vertical");
                    ui.checkbox(&mut self.show_horizontal_seam_outlines, "Horizontal");
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Gradient outlines");
                    ui.checkbox(&mut self.show_vertical_gradient_outlines, "Vertical");
                    ui.checkbox(&mut self.show_horizontal_gradient_outlines, "Horizontal");
                });
                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Gradients");
                    ui.checkbox(&mut self.show_vertical_gradients, "Vertical");
                    ui.checkbox(&mut self.show_horizontal_gradients, "Horizontal");
                });
                ui.separator();
            });

            ui.separator();

            ui.scope(|ui| {
                let cell_size = vec2(24.0, 24.0);
                let mut needs_seam_recalculation = false;

                // Draw widget base
                let widget_size = cell_size * vec2(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
                let (rect, _response) = ui.allocate_exact_size(widget_size, Sense::hover());
                ui.painter().rect_filled(rect, 0.0, Color32::GRAY);

                // Draw pixels
                for y in 0..IMAGE_HEIGHT {
                    for x in 0..IMAGE_WIDTH {
                        let pixel_rect =
                            Rect::from_min_size(rect.left_top() + cell_size * vec2(x as f32, y as f32), cell_size);

                        let pixel_response = ui.allocate_rect(pixel_rect, Sense::click_and_drag());

                        if pixel_response.clicked_by(PointerButton::Primary) {
                            self.image_pixels[y][x] = self.selected_color;
                            needs_seam_recalculation = true;
                        }

                        if pixel_response.clicked_by(PointerButton::Secondary) {
                            self.selected_color = self.image_pixels[y][x];
                        }

                        ui.painter()
                            .rect_filled(pixel_rect.shrink(1.0), 0.0, self.image_pixels[y][x]);
                    }
                }

                // Recalculate seams if neccessary
                if needs_seam_recalculation {
                    self.recalculate_seams();
                }

                // Draw gradients
                {
                    if self.show_vertical_gradients {
                        for gradient in &self.vertical_gradients {
                            let y1 = gradient.y.floor() as usize;
                            let y2 = (gradient.y + gradient.length).ceil() as usize;
                            let x = gradient.x as usize;

                            for y in y1..y2 {
                                let pixel_rect = Rect::from_min_size(
                                    rect.left_top() + cell_size * vec2(x as f32, y as f32),
                                    cell_size,
                                );

                                let color = lerp(
                                    Rgba::from(gradient.color_a)..=Rgba::from(gradient.color_b),
                                    remap(y as f32 + 0.5, y1 as f32..=y2 as f32, 0.0..=1.0),
                                );

                                ui.painter().rect_filled(pixel_rect.shrink(1.0), 0.0, color);
                            }
                        }
                    }

                    if self.show_horizontal_gradients {
                        for gradient in &self.horizontal_gradients {
                            let x1 = gradient.x.floor() as usize;
                            let x2 = (gradient.x + gradient.length).ceil() as usize;
                            let y = gradient.y as usize;

                            for x in x1..x2 {
                                let pixel_rect = Rect::from_min_size(
                                    rect.left_top() + cell_size * vec2(x as f32, y as f32),
                                    cell_size,
                                );

                                let color = lerp(
                                    Rgba::from(gradient.color_a)..=Rgba::from(gradient.color_b),
                                    remap(x as f32 + 0.5, x1 as f32..=x2 as f32, 0.0..=1.0),
                                );

                                ui.painter().rect_filled(pixel_rect.shrink(1.0), 0.0, color);
                            }
                        }
                    }
                }

                // Draw seam outlines
                {
                    let draw_seam = |seam: &Seam, axis: Vec2, color: Color32| {
                        let seam_line_start = rect.left_top() + cell_size * vec2(seam.x as f32, seam.y as f32);
                        let seam_line_end = seam_line_start + cell_size * Vec2::splat(seam.length as f32) * axis;

                        ui.painter()
                            .line_segment([seam_line_start, seam_line_end], Stroke { width: 3.0, color });

                        ui.painter().circle_filled(seam_line_start, 4.0, color);
                        ui.painter().circle_filled(seam_line_end, 4.0, color);
                    };

                    if self.show_vertical_seam_outlines {
                        for seam in &self.vertical_seams {
                            draw_seam(seam, vec2(0.0, 1.0), Color32::RED);
                        }
                    }

                    if self.show_horizontal_seam_outlines {
                        for seam in &self.horizontal_seams {
                            draw_seam(seam, vec2(1.0, 0.0), Color32::BLUE);
                        }
                    }
                }

                // Draw gradient outlines
                {
                    if self.show_vertical_gradient_outlines {
                        for gradient in &self.vertical_gradients {
                            let gradient_rect = Rect::from_min_size(
                                rect.left_top() + cell_size * vec2(gradient.x, gradient.y),
                                cell_size * vec2(1.0, gradient.length),
                            );

                            let color = Color32::GREEN;
                            let stroke_thin = Stroke { width: 2.0, color };
                            let stroke_bold = Stroke { width: 3.0, color };

                            ui.painter().rect_stroke(gradient_rect, 0.0, stroke_thin);

                            ui.painter()
                                .line_segment([gradient_rect.center_top(), gradient_rect.center_bottom()], stroke_bold);

                            ui.painter()
                                .circle(gradient_rect.center_top(), 4.0, gradient.color_a, stroke_thin);
                            ui.painter()
                                .circle(gradient_rect.center_bottom(), 4.0, gradient.color_b, stroke_thin);
                        }
                    }

                    if self.show_horizontal_gradient_outlines {
                        for gradient in &self.horizontal_gradients {
                            let gradient_rect = Rect::from_min_size(
                                rect.left_top() + cell_size * vec2(gradient.x, gradient.y),
                                cell_size * vec2(gradient.length, 1.0),
                            );

                            let color = Color32::YELLOW;
                            let stroke_thin = Stroke { width: 2.0, color };
                            let stroke_bold = Stroke { width: 3.0, color };

                            ui.painter().rect_stroke(gradient_rect, 0.0, stroke_thin);

                            ui.painter()
                                .line_segment([gradient_rect.left_center(), gradient_rect.right_center()], stroke_bold);

                            ui.painter()
                                .circle(gradient_rect.left_center(), 4.0, gradient.color_a, stroke_thin);
                            ui.painter()
                                .circle(gradient_rect.right_center(), 4.0, gradient.color_b, stroke_thin);
                        }
                    }
                }
            })
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(800.0, 640.0)),
        ..Default::default()
    };

    eframe::run_native("MLAA", options, Box::new(|_| Box::<MlaaApplication>::default()))
}
