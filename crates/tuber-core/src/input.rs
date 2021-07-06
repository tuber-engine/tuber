pub mod keyboard {
    #[derive(Debug, Copy, Clone)]
    pub enum Key {
        A = 0,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        Number0,
        Number1,
        Number2,
        Number3,
        Number4,
        Number5,
        Number6,
        Number7,
        Number8,
        Number9,
        Spacebar,
        Return,
        LShift,
        RShift,
        LControl,
        RControl,
        Escape,
    }
}

pub mod mouse {
    #[derive(Debug, Copy, Clone)]
    pub enum Button {
        Left,
        Right,
        Middle,
    }
}

pub enum Input {
    KeyDown(keyboard::Key),
    KeyUp(keyboard::Key),
    MouseMotion((f32, f32)),
    MouseButtonDown(mouse::Button),
    MouseButtonUp(mouse::Button),
}

pub struct InputState {
    key_state: [bool; 43],
    previous_key_state: [bool; 43],
    mouse_button_state: [bool; 3],
    previous_mouse_button_state: [bool; 3],
    last_mouse_position: (f32, f32),
    mouse_moved: bool,
}
impl InputState {
    pub fn new() -> Self {
        Self {
            key_state: [false; 43],
            previous_key_state: [false; 43],
            mouse_button_state: [false; 3],
            previous_mouse_button_state: [false; 3],
            last_mouse_position: (0.0, 0.0),
            mouse_moved: false,
        }
    }

    pub fn is(&self, input: Input) -> bool {
        match input {
            Input::KeyDown(key) => self.key_state[key as usize],
            Input::KeyUp(key) => !self.key_state[key as usize],
            Input::MouseButtonDown(button) => self.mouse_button_state[button as usize],
            Input::MouseButtonUp(button) => !self.mouse_button_state[button as usize],
            Input::MouseMotion(..) => self.mouse_moved,
        }
    }

    pub fn was(&self, input: Input) -> bool {
        match input {
            Input::KeyDown(key) => self.previous_key_state[key as usize],
            Input::KeyUp(key) => !self.previous_key_state[key as usize],
            Input::MouseButtonDown(button) => self.previous_mouse_button_state[button as usize],
            Input::MouseButtonUp(button) => !self.previous_mouse_button_state[button as usize],
            Input::MouseMotion(..) => unimplemented!(),
        }
    }

    pub fn handle_input(&mut self, input: Input) {
        self.mouse_moved = false;
        self.previous_key_state = self.key_state.clone();
        self.previous_mouse_button_state = self.mouse_button_state.clone();
        match input {
            Input::KeyDown(key) => self.key_state[key as usize] = true,
            Input::KeyUp(key) => self.key_state[key as usize] = false,
            Input::MouseButtonDown(button) => {
                self.mouse_button_state[button as usize] = true;
            }
            Input::MouseButtonUp(button) => {
                self.mouse_button_state[button as usize] = false;
            }
            Input::MouseMotion(new_position) => {
                self.last_mouse_position = new_position;
                self.mouse_moved = true;
            }
        }
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.last_mouse_position
    }
}
