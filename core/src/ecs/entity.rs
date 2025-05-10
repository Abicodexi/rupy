#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);


#[derive(Debug)]
pub struct InstanceBatch {
    batch: std::vec::Vec<Batch>,
    batches: std::collections::HashMap<super::Entity, Vec<(super::Entity, super::Transform)>>,
}
type Batch = (crate::Entity, std::vec::Vec<(super::Entity, super::Transform)>);

impl InstanceBatch {
    pub fn new() -> Self {
        Self {
            batch: std::vec::Vec::new(),
            batches: std::collections::HashMap::new(),
        }
    }
    pub fn batches(
        &self,
    ) -> &std::vec::Vec<Batch> {
        &self.batch
    }
    fn insert(&mut self, batch: Batch){
        let id = batch.0.0.clone();
        if batch.0.0 > self.batch.len() {
            self.batch.resize(id, batch);
            return;
        }
        self.batch[id] = batch;

    }
    fn get(&self, entity:& crate::Entity) -> &Batch{
        return &self.batch[entity.0]
    }
    pub fn raw_data_for(
        &self,
        target: super::Entity,
        frustum: Option<&crate::camera::Frustum>,
    ) -> Vec<crate::TransformRaw> {
        if target.0 > self.batch.len() {
            return Vec::new();
        }
        let batch = self.get(&target);
        let raw_data: Vec<crate::TransformRaw> =  batch.1
        .iter()
        .filter_map(|&(_source, transform)| {
            let pos = cgmath::Point3::new(
                transform.model_matrix.w.x,
                transform.model_matrix.w.y,
                transform.model_matrix.w.z,
            );

            if frustum.map_or(true, |f| f.contains_sphere(pos, 0.1)) {
                Some(transform.data())
            } else {
                None
            }
        })
        .collect();
    raw_data
        
    }
    pub fn raw_data(
        &self,
        frustum: Option<&crate::camera::Frustum>,
    ) -> Vec<Vec<crate::TransformRaw>> {
        self.batches
            .values()
            .map(|batch| {
                batch
                    .iter()
                    .filter_map(|&(_source_entity, transform)| {
                        let pos = cgmath::Point3::new(
                            transform.model_matrix.w.x,
                            transform.model_matrix.w.y,
                            transform.model_matrix.w.z,
                        );
                        if frustum.map_or(true, |f| f.contains_sphere(pos, 0.1)) {
                            Some(transform.data())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect()
    }
}
