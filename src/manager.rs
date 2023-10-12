use crate::list::{BufferList, MemoryList};
use std::io::{BufReader, Cursor};
struct Manager<'a> {
    clients: BufferList<Cursor<&'a str>>,
    subjects: MemoryList<&'a str>,
    landing_pages: MemoryList<&'a str>,
}

impl Manager<'_> {
    pub fn new() -> Self {
        Self {
            clients: BufferList::new(BufReader::new(Cursor::new(
                "test@aol.com\ntest@web.com\ntest@mail.com",
            ))),
            subjects: MemoryList::new(vec!["Hi again", "Since we last spoke"]),
            landing_pages: MemoryList::new(vec![
                "https://business.com/lp/new",
                "https://business.com/lp/current",
                "https://business.com/lp/best",
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_a_manager() {
        let _ = Manager::new();
    }

    #[test]
    fn it_should_add_list_to_manager() {
        let mut manager = Manager::new();
        assert_eq!(manager.clients.next().unwrap(), "test@aol.com");

        assert_eq!(manager.subjects.next().unwrap(), "Hi again");

        let collection: Vec<&str> = manager.landing_pages.into_iter().collect();
        assert_eq!(collection.len(), 3);
        assert_eq!(collection[0], "https://business.com/lp/new");
        assert_eq!(collection[1], "https://business.com/lp/current");
        assert_eq!(collection[2], "https://business.com/lp/best");
    }
}
