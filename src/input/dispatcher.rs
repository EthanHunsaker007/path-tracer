use cgmath::{InnerSpace, Vector3, Zero};

use crate::input::Action;
use crate::State;
pub struct ActionDispatcher {
    pub zoom: f32,
}


impl ActionDispatcher {
    pub fn new() -> Self {
        ActionDispatcher { 
            zoom: 0.0 
        }
    }
    pub fn dispatch(&mut self, actions: Vec<Action>, state: &mut State) {
        let mut camera_movement: Vector3<f32> = cgmath::Vector3::zero();
        for action in actions {
            match action {
                Action::MoveForward => {
                    camera_movement += state.camera.camera.forward;
                },
                Action::MoveBack => {
                    camera_movement -= state.camera.camera.forward;
                },
                Action::MoveRight => {
                    camera_movement += state.camera.camera.right;
                },
                Action::MoveLeft => {
                    camera_movement -= state.camera.camera.right;
                },
                Action::MoveUp => {
                    camera_movement -= cgmath::Vector3::unit_y();
                },
                Action::MoveDown => {
                    camera_movement += cgmath::Vector3::unit_y();
                },
                Action::ExitProgram => state.quit(),
                Action::Fullscreen => {
                    state.surface_state.toggle_fullscreen();
                    state.resize(state.surface_state.window.inner_size());
                    state.input_handler.flags.camera_has_moved = true;
                },
                Action::Zoom => {
                    state.camera.set_fov(state.config.fov * (state.config.base_zoom + self.zoom), &state.gpu_context.queue);
                    state.input_handler.flags.is_zoomed = true;
                    state.input_handler.flags.camera_has_moved = true;
                }
                Action::UnZoom => {
                    state.camera.set_fov(state.config.fov, &state.gpu_context.queue);
                    state.input_handler.flags.is_zoomed = false;
                    self.zoom = 0.0;
                    state.input_handler.flags.camera_has_moved = true;
                }
                Action::ZoomIn => {
                    self.zoom += 0.01;
                    state.camera.set_fov(state.config.fov * (state.config.base_zoom + self.zoom), &state.gpu_context.queue);
                    state.input_handler.flags.camera_has_moved = true;
                }
                Action::ZoomOut => {
                    self.zoom -= 0.01;
                    state.camera.set_fov(state.config.fov * (state.config.base_zoom + self.zoom), &state.gpu_context.queue);
                    state.input_handler.flags.camera_has_moved = true;
                }
                Action::SetFlySpeed(speed) => {
                    state.config.speed = speed;
                }
                Action::Test => {
                    state.scene.setup_test_scene(&state.gpu_context.device);
                    
                    state.bind_groups.rebuild_scene_bind_group(&state.gpu_context.device, &state.scene.material_buffer, &state.scene.vertex_buffer, &state.scene.tri_buffer);
                    state.input_handler.flags.camera_has_moved = true;
                }
                _ => ()
            }
        }
        
        if camera_movement.magnitude() > 0.0 {
            let new_camera_pos = state.camera.camera.position + camera_movement.normalize() * state.config.speed;
            state.camera.set_position(new_camera_pos, &state.gpu_context.queue);
            state.input_handler.flags.camera_has_moved = true;
        }
    }
}