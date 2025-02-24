use crate::{
    mesh::Meshable,
    render::{Consts, Globals, Mesh, Model, Renderer, TerrainLocals, TerrainPipeline},
};
use client::Client;
use common::{
    terrain::{TerrainChunkSize, TerrainMap},
    vol::{SampleVol, VolSize},
    volumes::vol_map_2d::VolMap2dErr,
};
use frustum_query::frustum::Frustum;
use std::{collections::HashMap, i32, ops::Mul, sync::mpsc, time::Duration};
use vek::*;

struct TerrainChunk {
    // GPU data
    model: Model<TerrainPipeline>,
    locals: Consts<TerrainLocals>,
    visible: bool,
    z_bounds: (f32, f32),
}

struct ChunkMeshState {
    pos: Vec2<i32>,
    started_tick: u64,
    active_worker: bool,
}

/// A type produced by mesh worker threads corresponding to the position and mesh of a chunk.
struct MeshWorkerResponse {
    pos: Vec2<i32>,
    z_bounds: (f32, f32),
    mesh: Mesh<TerrainPipeline>,
    started_tick: u64,
}

/// Function executed by worker threads dedicated to chunk meshing.
fn mesh_worker(
    pos: Vec2<i32>,
    z_bounds: (f32, f32),
    started_tick: u64,
    volume: <TerrainMap as SampleVol<Aabr<i32>>>::Sample,
    range: Aabb<i32>,
) -> MeshWorkerResponse {
    MeshWorkerResponse {
        pos,
        z_bounds,
        mesh: volume.generate_mesh(range),
        started_tick,
    }
}

pub struct Terrain {
    chunks: HashMap<Vec2<i32>, TerrainChunk>,

    // The mpsc sender and receiver used for talking to meshing worker threads.
    // We keep the sender component for no reason other than to clone it and send it to new workers.
    mesh_send_tmp: mpsc::Sender<MeshWorkerResponse>,
    mesh_recv: mpsc::Receiver<MeshWorkerResponse>,
    mesh_todo: HashMap<Vec2<i32>, ChunkMeshState>,
}

impl Terrain {
    pub fn new() -> Self {
        // Create a new mpsc (Multiple Produced, Single Consumer) pair for communicating with
        // worker threads that are meshing chunks.
        let (send, recv) = mpsc::channel();

        Self {
            chunks: HashMap::new(),

            mesh_send_tmp: send,
            mesh_recv: recv,
            mesh_todo: HashMap::new(),
        }
    }

    /// Maintain terrain data. To be called once per tick.
    pub fn maintain(
        &mut self,
        renderer: &mut Renderer,
        client: &Client,
        focus_pos: Vec3<f32>,
        loaded_distance: f32,
        view_mat: Mat4<f32>,
        proj_mat: Mat4<f32>,
    ) {
        let current_tick = client.get_tick();

        // Add any recently created or changed chunks to the list of chunks to be meshed.
        for pos in client
            .state()
            .changes()
            .new_chunks
            .iter()
            .chain(client.state().changes().changed_chunks.iter())
        {
            // TODO: ANOTHER PROBLEM HERE!
            // What happens if the block on the edge of a chunk gets modified? We need to spawn
            // a mesh worker to remesh its neighbour(s) too since their ambient occlusion and face
            // elision information changes too!
            for i in -1..2 {
                for j in -1..2 {
                    let pos = pos + Vec2::new(i, j);

                    if !self.chunks.contains_key(&pos) {
                        let mut neighbours = true;
                        for i in -1..2 {
                            for j in -1..2 {
                                neighbours &= client
                                    .state()
                                    .terrain()
                                    .get_key(pos + Vec2::new(i, j))
                                    .is_some();
                            }
                        }

                        if neighbours {
                            self.mesh_todo.entry(pos).or_insert(ChunkMeshState {
                                pos,
                                started_tick: current_tick,
                                active_worker: false,
                            });
                        }
                    }
                }
            }
        }
        // Remove any models for chunks that have been recently removed.
        for pos in &client.state().changes().removed_chunks {
            self.chunks.remove(pos);
            self.mesh_todo.remove(pos);
        }

        for todo in self
            .mesh_todo
            .values_mut()
            // Only spawn workers for meshing jobs without an active worker already.
            .filter(|todo| !todo.active_worker)
        {
            // Find the area of the terrain we want. Because meshing needs to compute things like
            // ambient occlusion and edge elision, we also need the borders of the chunk's
            // neighbours too (hence the `- 1` and `+ 1`).
            let aabr = Aabr {
                min: todo
                    .pos
                    .map2(TerrainMap::chunk_size(), |e, sz| e * sz as i32 - 1),
                max: todo
                    .pos
                    .map2(TerrainMap::chunk_size(), |e, sz| (e + 1) * sz as i32 + 1),
            };

            // Copy out the chunk data we need to perform the meshing. We do this by taking a
            // sample of the terrain that includes both the chunk we want and its neighbours.
            let volume = match client.state().terrain().sample(aabr) {
                Ok(sample) => sample,
                // Either this chunk or its neighbours doesn't yet exist, so we keep it in the
                // queue to be processed at a later date when we have its neighbours.
                Err(VolMap2dErr::NoSuchChunk) => return,
                _ => panic!("Unhandled edge case"),
            };

            // The region to actually mesh
            let min_z = volume
                .iter()
                .fold(i32::MAX, |min, (_, chunk)| chunk.get_min_z().min(min));
            let max_z = volume
                .iter()
                .fold(i32::MIN, |max, (_, chunk)| chunk.get_max_z().max(max));

            let aabb = Aabb {
                min: Vec3::from(aabr.min) + Vec3::unit_z() * (min_z - 1),
                max: Vec3::from(aabr.max) + Vec3::unit_z() * (max_z + 1),
            };

            // Clone various things so that they can be moved into the thread.
            let send = self.mesh_send_tmp.clone();
            let pos = todo.pos;

            // Queue the worker thread.
            client.thread_pool().execute(move || {
                let _ = send.send(mesh_worker(
                    pos,
                    (min_z as f32, max_z as f32),
                    current_tick,
                    volume,
                    aabb,
                ));
            });
            todo.active_worker = true;
        }

        // Receive a chunk mesh from a worker thread and upload it to the GPU, then store it.
        // Only pull out one chunk per frame to avoid an unacceptable amount of blocking lag due
        // to the GPU upload. That still gives us a 60 chunks / second budget to play with.
        if let Ok(response) = self.mesh_recv.recv_timeout(Duration::new(0, 0)) {
            match self.mesh_todo.get(&response.pos) {
                // It's the mesh we want, insert the newly finished model into the terrain model
                // data structure (convert the mesh to a model first of course).
                Some(todo) if response.started_tick == todo.started_tick => {
                    self.chunks.insert(
                        response.pos,
                        TerrainChunk {
                            model: renderer
                                .create_model(&response.mesh)
                                .expect("Failed to upload chunk mesh to the GPU!"),
                            locals: renderer
                                .create_consts(&[TerrainLocals {
                                    model_offs: Vec3::from(
                                        response.pos.map2(TerrainMap::chunk_size(), |e, sz| {
                                            e as f32 * sz as f32
                                        }),
                                    )
                                    .into_array(),
                                }])
                                .expect("Failed to upload chunk locals to the GPU!"),
                            visible: false,
                            z_bounds: response.z_bounds,
                        },
                    );
                }
                // Chunk must have been removed, or it was spawned on an old tick. Drop the mesh
                // since it's either out of date or no longer needed.
                _ => {}
            }
        }

        // Construct view frustum
        let frustum = Frustum::from_modelview_and_projection(
            &view_mat.into_col_array(),
            &proj_mat.into_col_array(),
        );

        // Update chunk visibility
        let chunk_sz = TerrainChunkSize::SIZE.x as f32;
        for (pos, chunk) in &mut self.chunks {
            let chunk_pos = pos.map(|e| e as f32 * chunk_sz);

            // Limit focus_pos to chunk bounds and ensure the chunk is within the fog boundary
            let nearest_in_chunk = Vec2::from(focus_pos).clamped(chunk_pos, chunk_pos + chunk_sz);
            let in_range = Vec2::<f32>::from(focus_pos).distance_squared(nearest_in_chunk)
                < loaded_distance.powf(2.0);

            // Ensure the chunk is within the view frustrum
            let chunk_mid = Vec3::new(
                chunk_pos.x + chunk_sz / 2.0,
                chunk_pos.y + chunk_sz / 2.0,
                (chunk.z_bounds.0 + chunk.z_bounds.1) * 0.5,
            );
            let chunk_radius = (chunk.z_bounds.1 - chunk.z_bounds.0)
                .max(chunk_sz / 2.0)
                .powf(2.0)
                .mul(2.0)
                .sqrt();
            let in_frustum = frustum.sphere_intersecting(
                &chunk_mid.x,
                &chunk_mid.y,
                &chunk_mid.z,
                &chunk_radius,
            );

            chunk.visible = in_range && in_frustum;
        }
    }

    pub fn render(&self, renderer: &mut Renderer, globals: &Consts<Globals>) {
        for (_pos, chunk) in &self.chunks {
            if chunk.visible {
                renderer.render_terrain_chunk(&chunk.model, globals, &chunk.locals);
            }
        }
    }
}
