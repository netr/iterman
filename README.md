# iterman

### Introduction
This crate offers a seamless way to create and manage lists of items. It offers a simple interface for creating lists from different sources and iterating over them. It also offers a way to persist the state of the list through a customizable write function.

### Features
- Simplified list initialization
- Thread-safe list iteration
- Support for different types of lists including memory lists and stream lists
- Support for different style of iteration including EOF-sensitive and round-robin lists

### Usage
```rust
use std::io::{BufReader, Cursor};
use std::collections::HashMap;
use iterman::{ListLike, MemoryList, BufferList, mem_list_from_dir};

fn main() {
    // in-memory iterator
    let list = MemoryList::new_round_robin(vec![2, 3, 4]); // will continue infinitely
    assert_eq!(list.take(6).collect::<Vec<i32>>(), [2, 3, 4, 2, 3, 4]);

    // buffer iterator
    let reader = BufReader::new(Cursor::new("1\n2\n3\n"));
    let list = BufferList::new(reader); // will reach EOF and stop
    assert_eq!(list.collect::<Vec<String>>(), ["1", "2", "3"]);
    
    // in-memory iterator from a directory of files
    // each file's contents will be a list item
    let dir = mem_list_from_dir("path/to/dir");
    assert_eq!(dir.collect::<Vec<String>>().len(), 10);
}
```

### Example of a use case
```rust
use crate::list::{BufferList, MemoryList};
use std::io::{BufReader, Cursor};
struct Manager<'a> {
    clients: BufferList<File>,
    subjects: MemoryList<&'a str>,
    landing_pages: MemoryList<&'a str>,
    bodies: MemoryList<&'a str>,
}

impl Manager<'_> {
    pub fn new() -> Self {
        Self {
            clients: BufferList::new(BufReader::new(File::open("clients.txt").unwrap())),
            subjects: BufferList::new_round_robin(BufReader::new(File::open("subjects.txt").unwrap())),
            landing_pages: MemoryList::new(vec![
                "https://business.com/lp/new",
                "https://business.com/lp/current",
                "https://business.com/lp/best",
            ]),
            bodies: mem_list_from_dir("path/to/dir"),
        }
    }
}

fn main() {
    let mut manager = Manager::new();
    assert_eq!(manager.clients.next().unwrap(), "test@aol.com");
    assert_eq!(manager.subjects.next().unwrap(), "Hi again");
    
    let collection: Vec<&str> = manager.landing_pages.into_iter().collect();
    assert_eq!(collection.len(), 3);
    assert_eq!(collection[0], "https://business.com/lp/new");
    assert_eq!(collection[1], "https://business.com/lp/current");
    assert_eq!(collection[2], "https://business.com/lp/best");
    
    assert_eq!(manager.bodies.next().unwrap(), "This is the body of the email");
}
```

## Installation

To use [iterman], add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
iterman = "0.1.0"
```

### Contribution Guidelines Section

This section is crucial for open-source projects as it sets the tone for how contributions are managed. Since you're targeting developers, this section can be fairly technical, explaining how to set up the development environment, run tests, and contribute changes.

### Contributing

We welcome contributions! Here's how you can contribute:

1. Fork the repository.
2. Clone your fork: `git clone https://github.com/netr/iterman.git`
3. Create a new branch: `git checkout -b my-feature-branch`
4. Make your changes.
5. Run tests: `cargo test`
6. Commit your changes: `git commit -am 'Add my feature'`
7. Push to your branch: `git push origin my-feature-branch`
8. Create a Pull Request on GitHub.

Please make sure your contributions adhere to our coding guidelines.


### TODO
```markdown
- [ ] Add support for more types of lists
    - [ ] Create `DirList` that can be initialized with a directory and each file is a list item
    - [ ] Create a `ChunkList` that can be initialized with a list and a chunk size
    - [ ] Create a list that can be initialized with a closure
- [ ] Add support for more types of write functions
```