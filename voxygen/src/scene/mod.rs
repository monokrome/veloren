pub mod camera;
pub mod figure;
pub mod terrain;

use self::{camera::Camera, figure::FigureMgr, terrain::Terrain};
use crate::{
    render::{
        create_pp_mesh, create_skybox_mesh, Consts, Globals, Model, PostProcessLocals,
        PostProcessPipeline, Renderer, SkyboxLocals, SkyboxPipeline,
    },
    window::Event,
};
use client::Client;
use common::comp;
use vek::*;

// TODO: Don't hard-code this.
const CURSOR_PAN_SCALE: f32 = 0.005;

struct Skybox {
    model: Model<SkyboxPipeline>,
    locals: Consts<SkyboxLocals>,
}

struct PostProcess {
    model: Model<PostProcessPipeline>,
    locals: Consts<PostProcessLocals>,
}

pub struct Scene {
    globals: Consts<Globals>,
    camera: Camera,

    skybox: Skybox,
    postprocess: PostProcess,
    terrain: Terrain,
    loaded_distance: f32,

    figure_mgr: FigureMgr,
}

impl Scene {
    /// Create a new `Scene` with default parameters.
    pub fn new(renderer: &mut Renderer, client: &Client) -> Self {
        let resolution = renderer.get_resolution().map(|e| e as f32);

        Self {
            globals: renderer.create_consts(&[Globals::default()]).unwrap(),
            camera: Camera::new(resolution.x / resolution.y),

            skybox: Skybox {
                model: renderer.create_model(&create_skybox_mesh()).unwrap(),
                locals: renderer.create_consts(&[SkyboxLocals::default()]).unwrap(),
            },
            postprocess: PostProcess {
                model: renderer.create_model(&create_pp_mesh()).unwrap(),
                locals: renderer
                    .create_consts(&[PostProcessLocals::default()])
                    .unwrap(),
            },
            terrain: Terrain::new(),
            loaded_distance: 0.0,
            figure_mgr: FigureMgr::new(),
        }
    }

    /// Get a reference to the scene's globals
    pub fn globals(&self) -> &Consts<Globals> {
        &self.globals
    }

    /// Get a reference to the scene's camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the scene's camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Handle an incoming user input event (e.g.: cursor moved, key pressed, window closed).
    ///
    /// If the event is handled, return true.
    pub fn handle_input_event(&mut self, event: Event) -> bool {
        match event {
            // When the window is resized, change the camera's aspect ratio
            Event::Resize(dims) => {
                self.camera.set_aspect_ratio(dims.x as f32 / dims.y as f32);
                true
            }
            // Panning the cursor makes the camera rotate
            Event::CursorPan(delta) => {
                self.camera.rotate_by(Vec3::from(delta) * CURSOR_PAN_SCALE);
                true
            }
            // Zoom the camera when a zoom event occurs
            Event::Zoom(delta) => {
                self.camera.zoom_by(delta * 0.3);
                true
            }
            // All other events are unhandled
            _ => false,
        }
    }

    /// Maintain data such as GPU constant buffers, models, etc. To be called once per tick.
    pub fn maintain(&mut self, renderer: &mut Renderer, client: &Client) {
        // Get player position.
        let player_pos = client
            .state()
            .ecs()
            .read_storage::<comp::phys::Pos>()
            .get(client.entity())
            .map_or(Vec3::zero(), |pos| pos.0);

        // Alter camera position to match player.
        self.camera.set_focus_pos(player_pos + Vec3::unit_z() * 2.1);

        // Tick camera for interpolation.
        self.camera.update(client.state().get_time());

        // Compute camera matrices.
        let (view_mat, proj_mat, cam_pos) = self.camera.compute_dependents(client);

        // Update chunk loaded distance smoothly for nice shader fog
        let loaded_distance = client.loaded_distance().unwrap_or(0) as f32 * 32.0;
        self.loaded_distance = (0.98 * self.loaded_distance + 0.02 * loaded_distance).max(0.01);

        // Update global constants.
        renderer
            .update_consts(
                &mut self.globals,
                &[Globals::new(
                    view_mat,
                    proj_mat,
                    cam_pos,
                    self.camera.get_focus_pos(),
                    self.loaded_distance,
                    client.state().get_time_of_day(),
                    client.state().get_time(),
                    renderer.get_resolution(),
                )],
            )
            .expect("Failed to update global constants");

        // Maintain the terrain.
        self.terrain.maintain(
            renderer,
            client,
            self.camera.get_focus_pos(),
            self.loaded_distance,
            view_mat,
            proj_mat,
        );

        // Maintain the figures.
        self.figure_mgr.maintain(renderer, client);

        // Remove unused figures.
        self.figure_mgr.clean(client.get_tick());
    }

    /// Render the scene using the provided `Renderer`.
    pub fn render(&mut self, renderer: &mut Renderer, client: &mut Client) {
        // Render the skybox first (it appears over everything else so must be rendered first).
        renderer.render_skybox(&self.skybox.model, &self.globals, &self.skybox.locals);

        // Render terrain and figures.
        self.terrain.render(renderer, &self.globals);
        self.figure_mgr.render(renderer, client, &self.globals);

        renderer.render_post_process(
            &self.postprocess.model,
            &self.globals,
            &self.postprocess.locals,
        );
    }
}
