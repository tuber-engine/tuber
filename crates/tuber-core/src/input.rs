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

pub enum Input {
    KeyDown(keyboard::Key),
    KeyUp(keyboard::Key),
}

pub struct InputState {
    key_state: [bool; 43],
}
impl InputState {
    pub fn new() -> Self {
        Self {
            key_state: [false; 43],
        }
    }

    pub fn is(&self, input: Input) -> bool {
        match input {
            Input::KeyDown(key) => self.key_state[key as usize],
            Input::KeyUp(key) => !self.key_state[key as usize],
        }
    }

    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(key) => self.key_state[key as usize] = true,
            Input::KeyUp(key) => self.key_state[key as usize] = false,
        }
    }
}
