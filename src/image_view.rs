use std::cmp::min;
use std::time::SystemTime;
use cursive::{Printer, Vec2, View, With};
use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::{BaseColor, Color, ColorStyle, ColorType};
use cursive::utils::span::SpannedStr;
use image::imageops::{crop, crop_imm, FilterType, resize};
use image::{RgbImage};

pub struct ImageView {
    source_image: RgbImage,
    view: [[f32; 2]; 2],
    scaled_image: Option<RgbImage>,
    relayout: bool,
    mode: ImageViewMode,
    filter_mode: FilterType,
    rendered_view: Option<[[f32; 2]; 2]>
}

enum ImageViewMode {
    MOVE,
    ZOOM,
    CURSOR{position: Vec2}
}

impl View for ImageView {
    fn draw(&self, printer: &Printer) {
        let mut cursor_color = None;
        match &self.scaled_image {
            None => {
                printer.print((printer.output_size.x/2, printer.output_size.y/2), "Error")
            }
            Some(im) => {
                if let Some(v) = self.rendered_view {
                    if v == self.view {
                        return;
                    }
                }
                let draw_start_time = SystemTime::now();
                let left_padding = printer.output_size.x.checked_sub(im.width() as usize).unwrap_or(0)/2;
                let top_padding = printer.output_size.y.checked_sub(im.height() as usize).unwrap_or(0)/2;
                for (row, row_pixels) in im.rows().enumerate() {
                    let mut color = [0, 0, 0];
                    let mut amount = 0;
                    let mut start_pos = 0;
                    for (column, pixel) in row_pixels.map(|c|c.0).enumerate() {
                        if ImageView::color_distance(&pixel, &color) < 3 {
                            amount += 1;
                        }else{
                            printer.with_color(ColorStyle::front(ColorType::Color(Color::Rgb(color[0], color[1], color[2]))), |printer|{printer.print((start_pos + left_padding, row + top_padding), &"█".repeat(amount))});
                            amount = 1;
                            color = pixel;
                            start_pos = column;
                        }

                        if let ImageViewMode::CURSOR{position} = self.mode{
                            if position.x == column && position.y == row {
                                cursor_color = Some(pixel.clone());
                            }
                        }
                    }
                    printer.with_color(ColorStyle::front(ColorType::Color(Color::Rgb(color[0], color[1], color[2]))), |printer|{printer.print((start_pos + left_padding, row + top_padding), &"█".repeat(amount))});
                }
                printer.print((0, 0), &format!("{}ms", draw_start_time.elapsed().unwrap().as_secs_f32()*1000f32))
            }
        }
        match self.mode {
            ImageViewMode::CURSOR{ position } => {

                printer.with_color(ColorStyle::new(
                    ColorType::Color(Color::Light(BaseColor::Red)),
                    ColorType::Color(Color::Light(BaseColor::Black))
                ), |p|{
                    p.print(position, "X");
                    let color = cursor_color.unwrap();
                    p.print((position.x + 1, position.y + 1), &format!("({}, {}, {})", color[0], color[1], color[2]));
                });
            }
            _ => {}
        }
        let mode_label = match self.mode {
            ImageViewMode::MOVE => {
                "MOVE"
            }
            ImageViewMode::ZOOM => {
                "ZOOM"
            }
            ImageViewMode::CURSOR{..} => {
                "CURSOR"
            }
        };
        let status_text = format!("{:8} {:}%", mode_label, (100f32 / self.view_size().0).round() as usize);
        printer.print((printer.output_size.x - 1 - status_text.len(), printer.output_size.y-1), &status_text);
    }

    fn layout(&mut self, size: Vec2) {
        let (width, height) = self.view_size();
        let view_height = height * self.source_image.height() as f32;
        let view_width = width * self.source_image.width() as f32;
        let view_left = self.view[0][0] * self.source_image.width() as f32;
        let view_top = self.view[1][0] * self.source_image.height() as f32;
        let crop_image = crop_imm(&self.source_image, view_left.round() as u32, view_top.round() as u32, view_width.round() as u32, view_height.round() as u32).to_image();
        let image_aspect_ratio = self.source_image.width() as f32 / self.source_image.height() as f32;
        let window_aspect_ratio = size.x as f32 / (size.y as f32 * 2.5f32);
        if image_aspect_ratio < window_aspect_ratio {
            self.scaled_image = Some(resize(&crop_image, (2.5f32 * image_aspect_ratio * size.y as f32) as u32, size.y as u32, self.filter_mode));
        }else {
            self.scaled_image = Some(resize(&crop_image, size.x as u32, (1f32/(2.5f32 * image_aspect_ratio) * size.x as f32) as u32, self.filter_mode));
        }
    }

    fn needs_relayout(&self) -> bool {
        self.relayout
    }

    fn on_event(&mut self, e: Event) -> EventResult {
        match e {
            Event::Mouse { offset, position, event } => {
                match event {
                    MouseEvent::WheelUp => {
                        self.zoom(0.1f32);
                        EventResult::Consumed(None)
                    },
                    MouseEvent::WheelDown => {
                        self.zoom(-0.1f32);
                        EventResult::Consumed(None)
                    }
                    _ => EventResult::Ignored
                }
            },
            Event::Key(k) => {
                match k {
                    Key::Backspace => {
                        self.view = [[0f32, 1f32], [0f32, 1f32]];
                        self.relayout = true;
                        EventResult::Consumed(None)
                    },
                    Key::Up => {
                        match &mut self.mode {
                            ImageViewMode::MOVE => {
                                let (_, height) = self.view_size();
                                let move_amount = height * 0.01f32;
                                if self.view[1][0] >= move_amount {
                                    self.view[1][0] -= move_amount;
                                    self.view[1][1] -= move_amount;
                                }
                                EventResult::Consumed(None)
                            }
                            ImageViewMode::ZOOM => {
                                self.zoom(0.1f32);
                                EventResult::Consumed(None)
                            }
                            ImageViewMode::CURSOR{position, ..} => {
                                position.y -= 1;
                                EventResult::Consumed(None)
                            }
                        }
                    },
                    Key::Down => {
                        match &mut self.mode {
                            ImageViewMode::MOVE => {
                                let (_, height) = self.view_size();
                                let move_amount = height * 0.01f32;
                                if self.view[1][1] <= 1f32 - move_amount {
                                    self.view[1][0] += move_amount;
                                    self.view[1][1] += move_amount;
                                }
                                EventResult::Consumed(None)
                            }
                            ImageViewMode::ZOOM => {
                                self.zoom(-0.1f32);
                                EventResult::Consumed(None)
                            }
                            ImageViewMode::CURSOR{position, ..} => {
                                position.y += 1;
                                EventResult::Consumed(None)
                            }
                        }
                    },
                    Key::Left => {
                        match &mut self.mode {
                            ImageViewMode::MOVE => {
                                let (width, _) = self.view_size();
                                let move_amount = width * 0.02f32;
                                if self.view[0][0] >= move_amount {
                                    self.view[0][0] -= move_amount;
                                    self.view[0][1] -= move_amount;
                                }
                                EventResult::Consumed(None)
                            },
                            ImageViewMode::CURSOR{position, ..} => {
                                position.x -= 1;
                                EventResult::Consumed(None)
                            },
                            ImageViewMode::ZOOM => {
                                self.mode = ImageViewMode::MOVE;
                                EventResult::Consumed(None)
                            }
                            _ => EventResult::Ignored
                        }
                    },
                    Key::Right => {
                        match &mut self.mode {
                            ImageViewMode::MOVE => {
                                let (width, _) = self.view_size();
                                let move_amount = width * 0.02f32;
                                if self.view[0][1] <= 1f32 - move_amount {
                                    self.view[0][0] += move_amount;
                                    self.view[0][1] += move_amount;
                                }
                                EventResult::Consumed(None)
                            },
                            ImageViewMode::CURSOR{position, ..} => {
                                position.x += 1;
                                EventResult::Consumed(None)
                            },
                            ImageViewMode::ZOOM => {
                                self.mode = ImageViewMode::MOVE;
                                EventResult::Consumed(None)
                            }
                            _ => EventResult::Ignored
                        }
                    }
                    _ => EventResult::Ignored
                }
            },
            Event::Char(c) => {
                match c {
                    'm' => {
                        self.mode = ImageViewMode::MOVE;
                        EventResult::Consumed(None)
                    },
                    'z' => {
                        self.mode = ImageViewMode::ZOOM;
                        EventResult::Consumed(None)
                    },
                    'c' => {
                        let image_dimensions = self.scaled_image.as_ref().unwrap().dimensions();

                        self.mode = ImageViewMode::CURSOR{
                            position: Vec2::new(image_dimensions.0 as usize / 2, image_dimensions.1 as usize / 2)
                        };

                        EventResult::Consumed(None)
                    }
                    _ => EventResult::Ignored
                }
            }
            _ => {EventResult::Ignored}
        }
    }
}

impl ImageView {
    pub fn new(image: RgbImage) -> Self {
        ImageView{
            source_image: image,
            view: [[0f32, 1f32], [0f32, 1f32]],
            scaled_image: None,
            relayout: false,
            mode: ImageViewMode::MOVE,
            filter_mode: FilterType::Nearest,
            rendered_view: None
        }
    }

    fn view_size(&self) -> (f32, f32) {
        (self.view[0][1] - self.view[0][0], self.view[1][1] - self.view[1][0])
    }

    fn zoom(&mut self, amount: f32) {
        let (zoom_size, _) = self.view_size();
        let min_dimension = 1f32/self.source_image.width().min(self.source_image.height()) as f32;
        let view_center = ((self.view[0][0] + self.view[0][1])/2f32, (self.view[1][0] + self.view[1][1])/2f32);
        let new_zoom_size = (zoom_size - zoom_size * amount).clamp(min_dimension, 1f32);
        let new_view = [[view_center.0 - new_zoom_size/2f32, view_center.0 + new_zoom_size/2f32], [view_center.1 - new_zoom_size/2f32, view_center.1 + new_zoom_size/2f32]];
        if new_view != self.view {
            self.view = new_view;
            self.relayout = true;
        }
    }

    fn color_distance(c1: &[u8], c2: &[u8]) -> u8 {
        c1[0].abs_diff(c2[0]).checked_add(c1[1].abs_diff(c2[1])).unwrap_or(255).checked_add((c1[2].abs_diff(c2[2]))).unwrap_or(255)
    }

}