
#[derive(Clone, Debug, Default)]
pub struct Transform {

}

impl Transform {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn rotate(&self, angle: u32) -> &Self {
        self
    }

    pub fn mirror(&self, angle: u32) -> &Self {
        self
    }
    pub fn translate(&self) -> &Self {
        self
    }
}


