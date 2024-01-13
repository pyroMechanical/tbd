use enigo::{self, KeyboardControllable, Key};
use gilrs::{Axis, Button, Event, Gilrs};

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

static CHARACTERS: [[Option<char>; 4]; 8] = [
    [Some('a'), Some('b'), Some('c'), None],
    [Some('d'), Some('e'), Some('f'), None],
    [Some('g'), Some('h'), Some('i'), None],
    [Some('j'), Some('k'), Some('l'), None],
    [Some('m'), Some('n'), Some('o'), None],
    [Some('p'), Some('q'), Some('r'), Some('s')],
    [Some('t'), Some('u'), Some('v'), None],
    [Some('w'), Some('x'), Some('y'), Some('z')],
];

static TYPE_MAGNITUDE_THRESHOLD: f32 = 0.7f32;

static TYPE_RETURN_THRESHOLD: f32 = 0.5f32;

use std::f32::consts::PI;

fn main() {
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
                }
                else {
                    enigo.key_up(Key::Backspace);
                }

                if gamepad.is_pressed(Button::LeftThumb) {
                    enigo.key_down(Key::Shift);
                }
                else {
                    enigo.key_up(Key::Shift);
                }
                if gamepad.is_pressed(Button::RightTrigger) && can_toggle_caps {
                    enigo.key_click(Key::CapsLock);
                    can_toggle_caps = false;
                }
                else if !gamepad.is_pressed(Button::RightTrigger) && !can_toggle_caps {
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

                let left_turn = left.angle()/(PI*2.0f32);
                let left_angle: u8 = ((left_turn * 8.0f32) + 0.5f32) as u8 % 8;

                let chars = CHARACTERS[left_angle as usize];

                let right_turn = right.angle()/(PI*2.0f32);
                let right_angle: u8 = ((right_turn * 4.0f32) + 0.5f32) as u8 % 4;
                //println!("{}", ((right_turn * 4.0f32) + 0.5f32) % 4.0f32);

                assert!(right_angle <= 3);

                if right.magnitude() >= TYPE_MAGNITUDE_THRESHOLD {
                    if can_type {
                        if let Some(c) = chars[right_angle as usize]{
                            enigo.key_click(Key::Layout(c));
                            can_type = false;
                        }
                    }
                }
                else if right.magnitude() <= TYPE_RETURN_THRESHOLD {
                    can_type = true;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
