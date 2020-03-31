/*
* MIT License
*
* Copyright (c) 2020 Tuber contributors
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Ecs {
    components: HashMap<TypeId, Vec<Box<dyn Any>>>,
}
impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            components: HashMap::new(),
        }
    }

    pub fn register_component<ComponentType: 'static>(&mut self) {
        self.components
            .insert(TypeId::of::<ComponentType>(), vec![]);
    }

    pub fn is_component_registered<ComponentType: 'static>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<ComponentType>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[test]
    fn can_register_component() {
        let mut ecs = Ecs::new();

        ecs.register_component::<Position>();
        assert!(ecs.is_component_registered::<Position>());
    }
}
