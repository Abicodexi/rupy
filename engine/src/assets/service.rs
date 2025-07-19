use crate::log_info;
use crate::AssetRequest;
use crate::BindGroupManager;
use crate::CacheKey;
use crate::CacheStorage;
use crate::EngineError;
use crate::Material;
use crate::MaterialAsset;
use crate::Model;
use crate::ModelAsset;
use crate::OwnedVertexBufferLayout;
use crate::RenderBindGroupLayouts;
use crate::Texture;
use crate::Vertex;
use crate::VertexInstance;
use crate::{
    log_error, MaterialManager, ModelManager, PipelineManager, ShaderManager, TextureManager,
};
use crossbeam::channel::{Receiver, Sender};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::ComputePipeline;
use wgpu::DepthStencilState;
use wgpu::RenderPipeline;
use wgpu::ShaderModule;
use wgpu::VertexBufferLayout;

static GLOBAL_ASSET_SERVICE: OnceCell<Arc<AssetService>> = OnceCell::new();
static GLOBAL_ASSET_TX: OnceCell<Sender<AssetRequest>> = OnceCell::new();

pub struct AssetService {
    queue: Arc<wgpu::Queue>,
    device: Arc<wgpu::Device>,
    bind_groups: Arc<RwLock<BindGroupManager>>,
    bind_group_layouts: Arc<RenderBindGroupLayouts>,
    materials: Arc<RwLock<MaterialManager>>,
    models: Arc<RwLock<ModelManager>>,
    textures: Arc<RwLock<TextureManager>>,
    shaders: Arc<RwLock<ShaderManager>>,
    pipelines: Arc<RwLock<PipelineManager>>,
}

impl AssetService {
    pub fn new(
        queue: Arc<wgpu::Queue>,
        device: Arc<wgpu::Device>,
        bind_groups: BindGroupManager,
        materials: MaterialManager,
        models: ModelManager,
        textures: TextureManager,
        shaders: ShaderManager,
        pipelines: PipelineManager,
    ) -> Self {
        let bind_group_layouts = RenderBindGroupLayouts::new(&device);
        Self {
            queue,
            device,
            bind_groups: Arc::new(RwLock::new(bind_groups)),
            bind_group_layouts: bind_group_layouts.into(),
            materials: Arc::new(RwLock::new(materials)),
            models: Arc::new(RwLock::new(models)),
            textures: Arc::new(RwLock::new(textures)),
            shaders: Arc::new(RwLock::new(shaders)),
            pipelines: Arc::new(RwLock::new(pipelines)),
        }
    }
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn bind_group_layouts(&self) -> &RenderBindGroupLayouts {
        &self.bind_group_layouts
    }
    pub fn get_material(&self, key: &CacheKey) -> Option<Arc<Material>> {
        self.materials
            .read()
            .ok()
            .and_then(|mgr| mgr.get_resource(key).cloned())
    }
    pub fn materials(
        &self,
    ) -> Result<
        RwLockReadGuard<'_, MaterialManager>,
        PoisonError<RwLockReadGuard<'_, MaterialManager>>,
    > {
        self.materials.read()
    }
    pub fn models(
        &self,
    ) -> Result<RwLockReadGuard<'_, ModelManager>, PoisonError<RwLockReadGuard<'_, ModelManager>>>
    {
        self.models.read()
    }
    pub fn get_model(&self, key: &CacheKey) -> Option<Arc<Model>> {
        match self.models.read() {
            Ok(mgr) => mgr.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Model manager poisoned: {}", e);
                None
            }
        }
    }

    pub fn get_shader(&self, key: &CacheKey) -> Option<Arc<ShaderModule>> {
        match self.shaders.read() {
            Ok(mgr) => mgr.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Shader manager poisoned: {}", e);
                None
            }
        }
    }

    pub fn get_texture(&self, key: &CacheKey) -> Option<Arc<Texture>> {
        match self.textures.read() {
            Ok(mgr) => mgr.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Texture manager poisoned: {}", e);
                None
            }
        }
    }

    pub fn get_render_pipeline(&self, key: &CacheKey) -> Option<Arc<wgpu::RenderPipeline>> {
        match self.pipelines.read() {
            Ok(mgr) => mgr.render.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Pipeline manager poisoned: {}", e);
                None
            }
        }
    }
    pub fn get_compute_pipeline(&self, key: &CacheKey) -> Option<Arc<wgpu::ComputePipeline>> {
        match self.pipelines.read() {
            Ok(mgr) => mgr.compute.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Pipeline manager poisoned: {}", e);
                None
            }
        }
    }
    pub fn get_bind_group(&self, key: &CacheKey) -> Option<Arc<wgpu::BindGroup>> {
        match self.bind_groups.read() {
            Ok(mgr) => mgr.get_resource(key).cloned(),
            Err(e) => {
                log_error!("Bind group manager poisoned: {}", e);
                None
            }
        }
    }
    pub fn pipelines(
        &self,
    ) -> Result<
        RwLockReadGuard<'_, PipelineManager>,
        PoisonError<RwLockReadGuard<'_, PipelineManager>>,
    > {
        self.pipelines.read()
    }
    /// Get all loaded materials.
    pub fn all_materials(&self) -> Vec<Arc<Material>> {
        self.materials
            .read()
            .map(|mgr| mgr.all().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all loaded models.
    pub fn all_models(&self) -> Vec<Arc<Model>> {
        self.models
            .read()
            .map(|mgr| mgr.all().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all loaded textures.
    pub fn all_textures(&self) -> Vec<Arc<Texture>> {
        self.textures
            .read()
            .map(|mgr| mgr.all().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all loaded pipelines.
    pub fn all_pipelines(&self) -> Vec<Arc<wgpu::RenderPipeline>> {
        self.pipelines
            .read()
            .map(|mgr| mgr.render.all().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all loaded shaders.
    pub fn all_shaders(&self) -> Vec<Arc<wgpu::ShaderModule>> {
        self.shaders
            .read()
            .map(|mgr| mgr.all().cloned().collect())
            .unwrap_or_default()
    }
    /// Get a model or load it synchronously if missing.
    pub fn get_or_load_model(
        &self,
        key: &CacheKey,
        file: String,
        v_shader: String,
        f_shader: String,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Option<Arc<Model>> {
        if let Some(model) = self.get_model(key) {
            Some(model)
        } else {
            self.load_model(
                file,
                v_shader,
                f_shader,
                bind_group_layouts,
                primitive,
                format,
                depth_stencil,
            );
            self.get_model(key)
        }
    }

    /// Get a material or load synchronously if missing.
    pub fn get_or_load_material(
        &self,
        key: &CacheKey,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
        format:wgpu::TextureFormat, 
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Option<Arc<Material>> {
        if let Some(mat_arc) = self.get_material(key) {
            Some(mat_arc)
        } else {
            self.load_material(
                bind_group_layouts,
                mat,
                v_shader,
                f_shader,
                primitive,
                format,
                depth_stencil,
            );
            self.get_material(key)
        }
    }
    pub fn get_or_load_shader(&self, key: &CacheKey, shader: &str) -> Option<Arc<ShaderModule>> {
        if let Some(shader_arc) = self.get_shader(key) {
            Some(shader_arc)
        } else {
            self.load_shader(shader);
            self.get_shader(key)
        }
    }
    pub fn get_or_load_texture(&self, key: &CacheKey, texture: &str) -> Option<Arc<Texture>> {
        if let Some(texture_arc) = self.get_texture(key) {
            Some(texture_arc)
        } else {
            self.load_texture(texture);
            self.get_texture(key)
        }
    }
    pub fn get_or_create_texture<F>(
        &self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> std::result::Result<Arc<Texture>, EngineError>
    where
        F: FnOnce() -> Result<Arc<Texture>, EngineError>,
    {
        if let Ok(mut textures) = self.textures.write() {
            let tex = textures.get_or_create(key, create_fn)?;
            return Ok(tex);
        } else {
            Err(EngineError::AssetLoadError(format!(
                "Failed to create texture: {}",
                key.id()
            )))
        }
    }
    pub fn get_or_load_render_pipeline<'a>(
        &self,
        f_shader: &str,
        v_shader: &str,
        layout: wgpu::PipelineLayout,
        buffers: &'a [VertexBufferLayout<'a>],
        format: wgpu::TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        key: CacheKey,
        label: String,
    ) -> Option<Arc<RenderPipeline>> {
        if let Some(pipeline_arc) = self.get_render_pipeline(&key) {
            Some(pipeline_arc)
        } else {
            self.load_render_pipeline(
                f_shader,
                v_shader,
                layout,
                buffers,
                format,
                depth_stencil,
                key,
                label,
            );
            self.get_render_pipeline(&key)
        }
    }
    pub fn get_or_load_compute_pipeline<'a>(
        &self,
        c_shader: &str,
        layout: wgpu::PipelineLayout,
        entry_point: Option<&'a str>,
        key: CacheKey,
        label: &str,
    ) -> Option<Arc<ComputePipeline>> {
        if let Some(pipeline_arc) = self.get_compute_pipeline(&key) {
            Some(pipeline_arc)
        } else {
            self.load_compute_pipeline(c_shader, layout, entry_point, key, label);
            self.get_compute_pipeline(&key)
        }
    }
    pub fn get_or_create_bind_group<F>(
        &self,
        key: crate::CacheKey,
        create_fn: F,
    ) -> std::result::Result<Arc<wgpu::BindGroup>, EngineError>
    where
        F: FnOnce() -> Result<Arc<wgpu::BindGroup>, EngineError>,
    {
        if let Ok(mut bind_groups) = self.bind_groups.write() {
            let bg = bind_groups.get_or_create(key, create_fn)?;
            return Ok(bg);
        } else {
            Err(EngineError::AssetLoadError(format!(
                "Failed to create bind group: {}",
                key.id()
            )))
        }
    }

    pub fn insert_bind_group(&self, key: CacheKey, bind_group: Arc<wgpu::BindGroup>) {
        if let Ok(mut bind_groups) = self.bind_groups.write() {
            if !bind_groups.contains_resource(&key) {
                bind_groups.insert_resource(key, bind_group);
            }
        }
    }
    pub fn get_bind_group_for_texture<'a>(
        &self,
        texture: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<Arc<wgpu::BindGroup>> {
        if let (Ok(mut textures), Ok(mut bind_groups)) =
            (self.textures.write(), self.bind_groups.write())
        {
            bind_groups.bind_group_for(&self.queue, &self.device, &mut textures, texture, layout)
        } else {
            None
        }
    }

    pub fn load_material(
        &self,

        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        let mat_name = mat.name.clone();

        if let (
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = materials.load_tobj_sync(
                &self.queue,
                &self.device,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut bind_groups,
                self.bind_group_layouts(),
                bind_group_layouts,
                mat,
                &v_shader,
                &f_shader,
                primitive,
                format,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                depth_stencil,
            ) {
                log_error!("{}", e.to_string());
            } else {
                log_info!("Loaded material: {}", mat_name);
            }
        }
    }
    pub async fn load_material_async(
        &self,

        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        mat: tobj::Material,
        v_shader: String,
        f_shader: String,
        primitive: wgpu::PrimitiveState,
       format: wgpu::TextureFormat, 
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        let mat_name = mat.name.clone();

        if let (
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = materials
                .load_tobj_async(
                    &self.queue,
                    &self.device,
                    &mut textures,
                    &mut shaders,
                    &mut pipelines,
                    &mut bind_groups,
                    self.bind_group_layouts(),
                    bind_group_layouts,
                    mat,
                    &v_shader,
                    &f_shader,
                    primitive,
                   format, 
                    &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                    depth_stencil,
                )
                .await
            {
                log_error!("{}", e.to_string());
            } else {
                log_info!("Loaded material: {}", mat_name);
            }
        }
    }
    pub fn load_material_asset(
        &self,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        layouts: &RenderBindGroupLayouts,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        asset: MaterialAsset,
    ) {
        let asset_name = asset.name.clone();

        if let (
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = materials.load_asset(
                self.device.as_ref(),
                self.queue.as_ref(),
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut bind_groups,
                layouts,
                bind_group_layouts,
                asset,
                buffers,
            ) {
                log_error!("{}", e.to_string());
            } else {
                log_info!("Loaded material asset: {}", asset_name);
            }
        }
    }
    pub async fn load_material_asset_async(
        &self,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        asset: MaterialAsset,
        buffers: &[VertexBufferLayout<'_>],
    ) {
        let asset_name = asset.name.clone();

        if let (
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = materials
                .load_asset_async(
                    self.device(),
                    self.queue(),
                    &mut textures,
                    &mut shaders,
                    &mut pipelines,
                    &mut bind_groups,
                    self.bind_group_layouts(),
                    bind_group_layouts,
                    asset,
                    buffers,
                )
                .await
            {
                log_error!("{}", e.to_string());
            } else {
                log_info!("Loaded material asset: {}", asset_name);
            }
        }
    }
    pub fn load_model(
        &self,
        file: String,
        v_shader: String,
        f_shader: String,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
        color_target: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if let (
            Ok(mut models),
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.models.write(),
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = pollster::FutureExt::block_on(models.load_obj(
                &self.queue,
                &self.device,
                &mut materials,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut bind_groups,
                self.bind_group_layouts(),
                &file,
                &v_shader,
                &f_shader,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                bind_group_layouts,
                primitive,
                color_target,
                depth_stencil,
            )) {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub fn load_gltf(
        &self,
        file: String,
        v_shader: String,
        f_shader: String,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
        format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if let (
            Ok(mut models),
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.models.write(),
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = pollster::FutureExt::block_on(models.load_gltf(
                &self.queue,
                &self.device,
                &mut materials,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut bind_groups,
                self.bind_group_layouts(),
                &file,
                &v_shader,
                &f_shader,
                &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                bind_group_layouts,
                primitive,
                format,
                depth_stencil,
            )) {
                log_error!("Failed to load glTF: {}", e.to_string());
            }
        }
    }
    pub async fn load_model_async(
        &self,
        file: String,
        v_shader: String,
        f_shader: String,
        bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
        primitive: wgpu::PrimitiveState,
       format: wgpu::TextureFormat,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if let (
            Ok(mut models),
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.models.write(),
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = models
                .load_obj(
                    &self.queue,
                    &self.device,
                    &mut materials,
                    &mut textures,
                    &mut shaders,
                    &mut pipelines,
                    &mut bind_groups,
                    self.bind_group_layouts(),
                    &file,
                    &v_shader,
                    &f_shader,
                    &[Vertex::LAYOUT, VertexInstance::LAYOUT],
                    bind_group_layouts,
                    primitive,
                    format,
                    depth_stencil,
                )
                .await
            {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub fn load_model_asset(
        &self,
        bind_group_layouts: Vec<Arc<BindGroupLayout>>,
        asset: ModelAsset,
        buffers: &[VertexBufferLayout<'_>],
    ) {
        if let (
            Ok(mut models),
            Ok(mut materials),
            Ok(mut textures),
            Ok(mut pipelines),
            Ok(mut shaders),
            Ok(mut bind_groups),
        ) = (
            self.models.write(),
            self.materials.write(),
            self.textures.write(),
            self.pipelines.write(),
            self.shaders.write(),
            self.bind_groups.write(),
        ) {
            if let Err(e) = models.load_asset(
                &self.queue,
                &self.device,
                &mut materials,
                &mut textures,
                &mut shaders,
                &mut pipelines,
                &mut bind_groups,
                self.bind_group_layouts(),
                bind_group_layouts,
                buffers,
                asset,
            ) {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub fn load_render_pipeline<'a>(
        &self,
        f_shader: &str,
        v_shader: &str,
        layout: wgpu::PipelineLayout,
        buffers: &'a [VertexBufferLayout<'a>],
        format: wgpu::TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        key: CacheKey,
        label: String,
    ) {
        if let Ok(mut pipelines) = self.pipelines.write() {
            if !pipelines.render.contains_resource(&key) {
                if let Err(e) = pipelines.render.create_pipeline(
                    &self,
                    f_shader,
                    v_shader,
                    layout,
                    buffers,
                    format,
                    depth_stencil,
                    key,
                    label,
                ) {
                    log_error!("{}", e.to_string());
                };
            }
        }
    }
    pub fn load_compute_pipeline<'a>(
        &self,
        c_shader: &str,
        layout: wgpu::PipelineLayout,
        entry_point: Option<&'a str>,
        key: CacheKey,
        label: &str,
    ) {
        if let Ok(mut pipelines) = self.pipelines.write() {
            if !pipelines.compute.contains_resource(&key) {
                if let Err(e) = pipelines.compute.create_pipeline(
                    &self,
                    c_shader,
                    layout,
                    entry_point,
                    key,
                    label,
                ) {
                    log_error!("{}", e.to_string());
                };
            }
        }
    }
    pub fn load_bind_group<'a>(
        &self,
        key: CacheKey,
        layout: &'a BindGroupLayout,
        entries: &'a [BindGroupEntry<'a>],
        label: &str,
    ) {
        if let Ok(mut bind_groups) = self.bind_groups.write() {
            if !bind_groups.contains_resource(&key) {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(label),
                    layout,
                    entries,
                });
                bind_groups.insert_resource(key, bind_group.into());
            }
        }
    }
    pub fn load_texture(&self, texture: &str) {
        if let Ok(mut textures) = self.textures.write() {
            if let Err(e) = textures.load(&self.queue, &self.device, &texture) {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub async fn load_texture_async(&self, texture: &str) {
        if let Ok(mut textures) = self.textures.write() {
            if let Err(e) = textures
                .load_async(&self.queue, &self.device, &texture)
                .await
            {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub fn load_shader(&self, shader: &str) {
        if let Ok(mut shaders) = self.shaders.write() {
            if let Err(e) = shaders.load(&self.device, &shader) {
                log_error!("{}", e.to_string());
            }
        }
    }
    pub fn spawn_thread(
        queue: Arc<wgpu::Queue>,
        device: Arc<wgpu::Device>,
        rx: Arc<Receiver<AssetRequest>>,
    ) -> std::thread::JoinHandle<()> {
        let bind_groups = BindGroupManager::new();
        let materials = MaterialManager::new();
        let models = ModelManager::new();
        let textures = TextureManager::new();
        let shaders = ShaderManager::new();
        let pipelines = PipelineManager::new();
        spawn_asset_service_thread(
            queue,
            device,
            rx,
            bind_groups,
            materials,
            models,
            textures,
            shaders,
            pipelines,
        )
    }
}

fn spawn_asset_service_thread(
    queue: Arc<wgpu::Queue>,
    device: Arc<wgpu::Device>,
    rx: Arc<Receiver<AssetRequest>>,
    bind_groups: BindGroupManager,
    materials: MaterialManager,
    models: ModelManager,
    textures: TextureManager,
    shaders: ShaderManager,
    pipelines: PipelineManager,
) -> std::thread::JoinHandle<()> {
    let service = Arc::new(AssetService::new(
        queue,
        device,
        bind_groups,
        materials,
        models,
        textures,
        shaders,
        pipelines,
    ));
    GLOBAL_ASSET_SERVICE.set(service.clone()).ok().unwrap();
    let rx_clone = rx.clone();
    std::thread::spawn(move || {
        asset_service_thread(service, rx_clone);
    })
}

fn asset_service_thread(service: Arc<AssetService>, rx: Arc<Receiver<AssetRequest>>) {
    while let Ok(request) = rx.recv() {
        match request {
            AssetRequest::Shutdown => {
                log_info!("Shutdown");
                break;
            }
            AssetRequest::LoadShader { shader } => {
                if let Ok(mut shaders) = service.shaders.write() {
                    if let Err(e) = shaders.load(&service.device, &shader) {
                        log_error!("{}", e.to_string());
                    }
                }
            }
            AssetRequest::LoadRenderPipeline {
                layout,
                f_shader,
                v_shader,
                format,
                key,
                label,
                buffers,
                depth_stencil,
            } => {
                let buffer_reconstruct = OwnedVertexBufferLayout::reconstruct_layouts(&buffers);
                service.load_render_pipeline(
                    &f_shader,
                    &v_shader,
                    layout,
                    &buffer_reconstruct,
                    format,
                    depth_stencil,
                    key,
                    label,
                );
            }
            AssetRequest::LoadTexture { texture } => {
                service.load_texture(&texture);
            }
            AssetRequest::LoadMaterial {
                bind_group_layouts,
                mat,
                v_shader,
                f_shader,
                primitive,
                format,
                depth_stencil,
            } => {
                service.load_material(
                    bind_group_layouts,
                    mat,
                    v_shader,
                    f_shader,
                    primitive,
                    format,
                    depth_stencil,
                );
            }
            AssetRequest::LoadMaterialAsset {
                bind_group_layouts,
                asset,
            } => {
                let buffers = &[Vertex::LAYOUT, VertexInstance::LAYOUT];
                service.load_material_asset(
                    buffers,
                    service.bind_group_layouts(),
                    bind_group_layouts,
                    asset,
                );
            }
            AssetRequest::LoadModel {
                file,
                v_shader,
                f_shader,
                bind_group_layouts,
                primitive,
                format,
                depth_stencil,
            } => {
                service.load_model(
                    file,
                    v_shader,
                    f_shader,
                    bind_group_layouts,
                    primitive,
                    format,
                    depth_stencil,
                );
            }
            AssetRequest::LoadModelAsset {
                bind_group_layouts,
                asset,
            } => {
                let buffers = &[Vertex::LAYOUT, VertexInstance::LAYOUT];
                service.load_model_asset(bind_group_layouts, asset, buffers);
            }
        }
    }
    log_info!("Asset service thread exiting");
}

pub fn asset_service() -> &'static Arc<AssetService> {
    GLOBAL_ASSET_SERVICE
        .get()
        .expect("AssetService not initialized!")
}
pub fn asset_request_tx() -> &'static Sender<AssetRequest> {
    GLOBAL_ASSET_TX
        .get()
        .expect("AssetService not initialized!")
}
