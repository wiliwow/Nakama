use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

pub struct InputController {
    enigo: Enigo,
}

impl InputController {
    pub fn new() -> Self {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).unwrap();
        InputController { enigo }
    }

    // Mouse controls
    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.enigo.move_mouse(x, y, Coordinate::Abs).unwrap();
    }

    pub fn mouse_click(&mut self, button: Button) {
        self.enigo.button(button, Direction::Click).unwrap();
    }

    // Keyboard controls
    pub fn key_sequence(&mut self, text: &str) {
        self.enigo.text(text).unwrap();
    }

    pub fn key_click(&mut self, key: Key) {
        self.enigo.key(key, Direction::Click).unwrap();
    }
}
