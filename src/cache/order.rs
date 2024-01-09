#[derive(Debug)]
pub struct Key {
    id: i64,
    type_url: String,
}

pub type Keys = Vec<Key>;
