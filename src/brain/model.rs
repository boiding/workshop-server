use std::collections::HashMap;

use crate::simulation::FlockId;

#[derive(Deserialize)]
pub struct BrainResponse(HashMap<FlockId, Intent>);

#[derive(Deserialize)]
pub struct Intent {
    pub heading: f64,
    pub speed: f64,
}
