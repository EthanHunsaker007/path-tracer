use crate::camera::*;
use std::collections::{HashMap, HashSet};

use winit::{event::*, keyboard::KeyCode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    MoveForward,
    MoveBack,
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
    ExitProgram,
    Zoom,
    UnZoom,
    ZoomIn,
    ZoomOut,
    Test,
    Fullscreen,
    SetFlySpeed(f32),
    None,
}

pub enum Context {
    InGame,
}

pub struct InputFlags {
    pub is_zoomed: bool,
    pub scrolled_up: bool,
    pub scrolled_down: bool,
    pub camera_has_moved: bool,
}

impl InputFlags {
    pub fn default() -> Self {
        InputFlags {
            is_zoomed: false,
            scrolled_down: false,
            scrolled_up: false,
            camera_has_moved: false,
        }
    }
}

pub struct Mouse {
    pub just_clicked: HashSet<MouseButton>,
    pub just_released: HashSet<MouseButton>,
    pub held: HashSet<MouseButton>,
}

impl Mouse {
    fn new() -> Self {
        Self {
            just_clicked: HashSet::new(),
            just_released: HashSet::new(),
            held: HashSet::new(),
        }
    }
    fn mouse_click(&mut self, state: &ElementState, mouse_button: &MouseButton) {
        match state {
            ElementState::Pressed => {
                if self.held.insert(*mouse_button) {
                    self.just_clicked.insert(*mouse_button);
                }
            }
            ElementState::Released => {
                self.held.remove(mouse_button);
                self.just_released.insert(*mouse_button);
            }
        }
    }
}

pub struct Keys {
    pub just_pressed: HashSet<KeyCode>,
    pub just_released: HashSet<KeyCode>,
    pub held: HashSet<KeyCode>,
}

impl Keys {
    fn new() -> Self {
        Self {
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            held: HashSet::new(),
        }
    }

    fn key_press(&mut self, state: &ElementState, key_code: &KeyCode) {
        match state {
            ElementState::Pressed => {
                if self.held.insert(*key_code) {
                    self.just_pressed.insert(*key_code);
                }
            }
            ElementState::Released => {
                self.held.remove(key_code);
                self.just_released.insert(*key_code);
            }
        }
    }
}

pub struct InputHandler {
    pub bindings: HashMap<KeyCode, [Action; 3]>,
    pub flags: InputFlags,
    pub context: Context,
    pub keys: Keys,
    pub mouse: Mouse,
}

impl InputHandler {
    pub fn new_defaults() -> Self {
        let mut bindings = HashMap::new();
        let flags = InputFlags::default();


        bindings.insert(
            KeyCode::KeyW,
            [Action::None, Action::MoveForward, Action::None],
        );
        bindings.insert(
            KeyCode::KeyA,
            [Action::None, Action::MoveLeft, Action::None],
        );
        bindings.insert(
            KeyCode::KeyS,
            [Action::None, Action::MoveBack, Action::None],
        );
        bindings.insert(
            KeyCode::KeyD,
            [Action::None, Action::MoveRight, Action::None],
        );
        bindings.insert(
            KeyCode::Space, 
            [Action::None, Action::MoveUp, Action::None]);
        bindings.insert(
            KeyCode::ShiftLeft,
            [Action::None, Action::MoveDown, Action::None],
        );
        bindings.insert(
            KeyCode::Escape,
            [Action::ExitProgram, Action::None, Action::None],
        );
        bindings.insert(
            KeyCode::F11,
            [Action::Fullscreen, Action::None, Action::None],
        );
        bindings.insert(
            KeyCode::KeyC, 
            [Action::Zoom, Action::None, Action::UnZoom]);
        bindings.insert(
            KeyCode::KeyT, 
            [Action::Test, Action::None, Action::None]);
        bindings.insert(
            KeyCode::ControlLeft,
            [
                Action::SetFlySpeed(0.1),
                Action::None,
                Action::SetFlySpeed(0.04),
            ],
        );

        Self {
            flags,
            bindings,
            context: Context::InGame,
            keys: Keys::new(),
            mouse: Mouse::new(),
        }
    }
    pub fn process_input(
        &mut self,
        process_event: &winit::event::Event<()>,
        camera: &mut Camera,
        queue: &wgpu::Queue,
    ) {
        match process_event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                            state,
                            repeat: false,
                            ..
                        },
                    ..
                } => {
                    self.keys.key_press(&state, key_code);
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        if y > &0.0 {
                            self.flags.scrolled_up = true;
                        } else if y < &0.0 {
                            self.flags.scrolled_down = true;
                        }
                    }
                    _ => (),
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    self.mouse.mouse_click(state, button);
                }
                _ => (),
            },

            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => self.mouse_move(delta, camera, queue),
                _ => (),
            },

            _ => (),
        }
    }

    pub fn get_action(&self, key: &KeyCode) -> Option<[Action; 3]> {
        self.bindings.get(key).cloned()
    }

    pub fn get_actions(&mut self) -> Vec<Action> {
        let mut actions = vec![];
        for key in &self.keys.just_pressed {
            if let Some(action) = self.get_action(key) {
                actions.push(action[0])
            }
        }
        for key in &self.keys.held {
            if let Some(action) = self.get_action(key) {
                actions.push(action[1])
            }
        }
        for key in &self.keys.just_released {
            if let Some(action) = self.get_action(key) {
                actions.push(action[2])
            }
        }

        if self.flags.is_zoomed == true && self.flags.scrolled_up == true {
            actions.push(Action::ZoomIn);
        }
        if self.flags.is_zoomed == true && self.flags.scrolled_down == true {
            actions.push(Action::ZoomOut);
        }

        self.clear_frame_input();
        self.reset_flags();
        actions
    }

    pub fn clear_frame_input(&mut self) {
        self.keys.just_pressed.clear();
        self.keys.just_released.clear();
        self.mouse.just_clicked.clear();
        self.mouse.just_released.clear();
    }

    pub fn reset_flags(&mut self) {
        self.flags.scrolled_down = false;
        self.flags.scrolled_up = false;
    }

    pub fn _rebind_key(
        &mut self,
        key: KeyCode,
        pressed_action: Action,
        held_action: Action,
        released_action: Action,
    ) {
        self.bindings
            .insert(key, [pressed_action, held_action, released_action]);
    }

    pub fn mouse_move(&mut self, delta: &(f64, f64), camera: &mut Camera, queue: &wgpu::Queue) {
        let pitch: f32 = camera.camera.pitch.0 + delta.1 as f32 * -0.1;
        let yaw: f32 = camera.camera.yaw.0 + delta.0 as f32 * 0.1;
        camera.set_rotation(pitch, yaw, &queue);
        self.flags.camera_has_moved = true;
    }
}
