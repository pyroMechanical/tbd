use enigo::{self, Key, KeyboardControllable};
use gilrs::{Axis, Button, Event, Gilrs};

use druid::{Widget, WindowDesc, Color, RenderContext, WidgetExt};

use std::io::Write;

struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalized(mut self) -> Vector2 {
        let mag = self.magnitude();
        if mag == 0.0f32 {
            return self;
        }
        self.x /= mag;
        self.y /= mag;
        self
    }

    fn scale(&mut self, magnitude: f32) {
        self.x *= magnitude;
        self.y *= magnitude;
    }
    //reports the angle in radians, with up being 0
    fn angle(&self) -> f32 {
        std::f32::consts::PI + (-self.x).atan2(-self.y)
    }
}

fn transform_input_vector(raw: Vector2) -> Vector2 {
    let mag_squared = if raw.magnitude_squared() >= 1.0f32 {
        1.0f32
    } else {
        raw.magnitude_squared()
    };
    let mut normalized = raw.normalized();
    normalized.scale(mag_squared);
    normalized
}

static CHARACTERS: [[Option<char>; 8]; 9] = [
    [None, None, None, None, None, None, None, None],
    [None, Some('a'), None, Some('b'), None, Some('c'), None, None],
    [None, Some('d'), None, Some('e'), None, Some('f'), None, None],
    [None, Some('g'), None, Some('h'), None, Some('i'), None, None],
    [None, Some('j'), None, Some('k'), None, Some('l'), None, None],
    [None, Some('m'), None, Some('n'), None, Some('o'), None, None],
    [None, Some('p'), None, Some('q'), None, Some('r'), None, Some('s')],
    [None, Some('t'), None, Some('u'), None, Some('v'), None, None],
    [None, Some('w'), None, None, None, Some('y'), None, Some('z')],
];

static TYPE_MAGNITUDE_THRESHOLD: f32 = 0.7f32;

static TYPE_RETURN_THRESHOLD: f32 = 0.5f32;

use std::f32::consts::PI;

fn make_widget() -> impl Widget<()> {
    use druid::widget::*;

    let char_label = |maybe_char: Option<_>| {
        let text = maybe_char.map_or(' ', |x| x).to_string();
        AspectRatioBox::new(Flex::column().main_axis_alignment(MainAxisAlignment::Center).with_flex_child(Label::<()>::new(text).with_text_alignment(druid::text::TextAlignment::Center), 1.0), 1.0)
    };

    let character_wheel = |chars: &[Option<_>;8]| {
        Flex::column()
        .with_flex_child(Flex::row()
        .with_flex_child(char_label(chars[0]), 1.0)
        .with_flex_child(char_label(chars[1]), 1.0)
        .with_flex_child(char_label(chars[2]), 1.0)
        .must_fill_main_axis(true), 1.0)
        .with_flex_child(Flex::row()
        .with_flex_child(char_label(chars[7]), 1.0)
        .with_flex_child(char_label(None), 1.0)
        .with_flex_child(char_label(chars[3]), 1.0)
        .must_fill_main_axis(true), 1.0)
        .with_flex_child(Flex::row()
        .with_flex_child(char_label(chars[6]), 1.0)
        .with_flex_child(char_label(chars[5]), 1.0)
        .with_flex_child(char_label(chars[4]), 1.0)
        .must_fill_main_axis(true), 1.0)
        .must_fill_main_axis(true)
    };

    let box_factory = |char_array: &[Option<_>;8]| {
        druid::widget::SizedBox::new(AspectRatioBox::new(Container::new(character_wheel(char_array)).background(Color::rgba8(127, 127, 255, 127)) , 1.0f64)).expand()
    };

    let row_factory = |char_arrays: &[[Option<_>; 8]; 9], index1, index2, index3| {
        druid::widget::Flex::row()
        .must_fill_main_axis(true)
        .with_flex_child(box_factory(&char_arrays[index1]), 1.0)
        .with_default_spacer()
        .with_flex_child(box_factory(&char_arrays[index2]), 1.0)
        .with_default_spacer()
        .with_flex_child(box_factory(&char_arrays[index3]), 1.0)
    };

    druid::widget::Flex::column()
    .must_fill_main_axis(true)
    .with_flex_child(row_factory(&CHARACTERS, 1, 2, 3), 1.0)
    .with_default_spacer()
    .with_flex_child(row_factory(&CHARACTERS, 8, 0, 4), 1.0)
    .with_default_spacer()
    .with_flex_child(row_factory(&CHARACTERS, 7, 6, 5), 1.0)
}

fn main() {
    let main_window = WindowDesc::new(
        make_widget()
    )
    .set_always_on_top(true)
    .window_size((200.0, 200.0))
    .set_position((50.0f64, 50.0f64))
    .transparent(true)
    .show_titlebar(false);

    druid::AppLauncher::with_window(main_window)
        .launch(())
        .expect("Failed to launch application.");
    let mut can_type = true;
    let mut can_toggle_caps = true;
    let mut gilrs = Gilrs::new().unwrap();
    let mut enigo = enigo::Enigo::new();

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    let mut new_event = false;

    loop {
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            //println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
            new_event = true;
        }
        if new_event {
            new_event = false;
            if let Some(id) = active_gamepad {
                let gamepad = gilrs.gamepad(id);

                if gamepad.is_pressed(Button::East) {
                    enigo.key_down(Key::Backspace);
                } else {
                    enigo.key_up(Key::Backspace);
                }

                if gamepad.is_pressed(Button::LeftThumb) {
                    enigo.key_down(Key::Shift);
                } else {
                    enigo.key_up(Key::Shift);
                }
                if gamepad.is_pressed(Button::RightTrigger) && can_toggle_caps {
                    enigo.key_click(Key::CapsLock);
                    can_toggle_caps = false;
                } else if !gamepad.is_pressed(Button::RightTrigger) && !can_toggle_caps {
                    can_toggle_caps = true;
                }

                let left_x = gamepad
                    .axis_data(Axis::LeftStickX)
                    .map(|axis| axis.value())
                    .unwrap_or_default();
                let left_y = gamepad
                    .axis_data(Axis::LeftStickY)
                    .map(|axis| axis.value())
                    .unwrap_or_default();
                let right_x = gamepad
                    .axis_data(Axis::RightStickX)
                    .map(|axis| axis.value())
                    .unwrap_or_default();
                let right_y = gamepad
                    .axis_data(Axis::RightStickY)
                    .map(|axis| axis.value())
                    .unwrap_or_default();

                let left = transform_input_vector(Vector2 {
                    x: left_x,
                    y: left_y,
                });
                let right = transform_input_vector(Vector2 {
                    x: right_x,
                    y: right_y,
                });

                if left.magnitude() < TYPE_RETURN_THRESHOLD {
                    continue;
                }

                let left_turn = left.angle() / (PI * 2.0f32);
                let left_angle: u8 = ((left_turn * 8.0f32) + 0.5f32) as u8 % 8;

                let chars = CHARACTERS[left_angle as usize];

                let right_turn = right.angle() / (PI * 2.0f32);
                let right_angle: u8 = ((right_turn * 4.0f32) + 0.5f32) as u8 % 4;
                //println!("{}", ((right_turn * 4.0f32) + 0.5f32) % 4.0f32);

                assert!(right_angle <= 3);

                if right.magnitude() >= TYPE_MAGNITUDE_THRESHOLD {
                    if can_type {
                        if let Some(c) = chars[right_angle as usize] {
                            enigo.key_click(Key::Layout(c));
                            can_type = false;
                        }
                    }
                } else if right.magnitude() <= TYPE_RETURN_THRESHOLD {
                    can_type = true;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
