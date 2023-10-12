use std::collections::HashMap;
use std::hash::Hash;

struct Manager {
    lists: HashMap<&str, Box<dyn ListLike>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {

        }
    }
}

trait ListLike {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_a_manager() {
        let _ = ma

    }
}