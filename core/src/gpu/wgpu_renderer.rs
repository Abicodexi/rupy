use crate::CacheStorage;

#[warn(dead_code)]
pub struct WgpuRenderer {}

impl WgpuRenderer {
    pub fn new() -> Self {
        WgpuRenderer {}
    }
    pub fn compute_pass(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &crate::World,
        managers: &mut crate::Managers,
    ) {
        if let Some(projection) = world.projection() {
            let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("compute encoder"),
            });
            projection.compute_projection(
                queue,
                encoder,
                managers,
                Some("equirect projection compute pass"),
            );
        }
    }
}

impl crate::Renderer for WgpuRenderer {
    fn render(
        &self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        managers: &mut crate::Managers,
        rpass: &mut wgpu::RenderPass,
        world: &crate::World,
        camera: &crate::camera::Camera,
    ) {
        let frustum = &camera.frustum;
        if let Some(projection) = world.projection() {
            projection.render(rpass, managers, camera);
        }
        for (entity, rend_opt) in world.get_renderables().iter().enumerate() {
            let rend = match rend_opt {
                Some(r) if r.visible => r,
                _ => continue,
            };
            let model = crate::CacheStorage::get(&managers.model_manager, &rend.model_key).unwrap();
            for mesh_instance in &model.meshes {
                if let Some(mesh) = <crate::MeshManager as crate::CacheStorage<crate::Mesh>>::get(
                    &managers.mesh_manager,
                    &mesh_instance.material_key,
                ) {
                    let material = crate::CacheStorage::get(
                        &managers.material_manager,
                        &mesh_instance.material_key,
                    )
                    .unwrap();
                    let pipeline = managers
                        .render_pipeline_manager
                        .get(&material.shader_key)
                        .unwrap();

                    rpass.set_pipeline(&pipeline);
                    for (i, bg) in material.bind_groups.iter().enumerate() {
                        rpass.set_bind_group(i as u32, bg.as_ref(), &[]);
                    }

                    if world
                        .instance
                        .batches()
                        .contains_key(&crate::Entity(entity))
                    {
                        let instance_data: Vec<_> = world
                            .instance
                            .raw_data_for(crate::Entity(entity), Some(frustum));
                        if !instance_data.is_empty() {
                            let vertex_buffer = crate::CacheStorage::get(
                                &managers.buffer_manager.w_buffer,
                                &mesh_instance.vertex_buffer_key,
                            )
                            .unwrap();
                            let index_buffer = crate::CacheStorage::get(
                                &managers.buffer_manager.w_buffer,
                                &mesh_instance.index_buffer_key,
                            )
                            .unwrap();
                            let instance_buffer = crate::WgpuBuffer::from_data(
                                queue,
                                device,
                                &instance_data,
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance batch buffer", model.name)),
                            );
                            rpass.set_index_buffer(
                                index_buffer.buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            rpass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
                            rpass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
                            rpass.draw_indexed(
                                0..mesh.index_count,
                                0,
                                0..instance_data.len() as u32,
                            );
                        }
                    } else {
                        let instance_buffer = if let Some(tr) = &world.transforms[entity] {
                            let ib = crate::WgpuBuffer::from_data(
                                queue,
                                device,
                                &tr.data(),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance buffer", model.name)),
                            );
                            queue.write_buffer(&ib.buffer, 0, bytemuck::cast_slice(&tr.data()));
                            ib
                        } else {
                            let tr = crate::Transform::default();
                            let ib = crate::WgpuBuffer::from_data(
                                queue,
                                device,
                                &crate::Transform::default().data(),
                                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                Some(&format!("{} instance buffer", model.name)),
                            );
                            queue.write_buffer(&ib.buffer, 0, bytemuck::cast_slice(&tr.data()));
                            ib
                        };

                        let vertex_buffer = crate::CacheStorage::get(
                            &managers.buffer_manager.w_buffer,
                            &mesh.vertex_buffer_key,
                        )
                        .unwrap();
                        let index_buffer = crate::CacheStorage::get(
                            &managers.buffer_manager.w_buffer,
                            &mesh.index_buffer_key,
                        )
                        .unwrap();

                        rpass.set_index_buffer(
                            index_buffer.buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        rpass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
                        rpass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

                        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                }
            }
        }
    }
}
