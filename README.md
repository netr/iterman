# iterman

### Introduction
This crate offers a seamless way to manage multiple collections in Rust. From vectors to buffer readers, initialize, name, iterate, and write results all in one place.

### Features
- Simplified list initialization
- Name-based access for efficient management
- State persistence through customizable write functions
- Support for different types of lists including EOF-sensitive and round-robin lists

### Usage
```rust
// Initialize manager
let mut manager = ListManager::new();

// Add named list
manager.add_list("test", vec![1, 2, 3]);

// Iterate over list
while let Some(item) = manager.get_list_by_name("test").next() {
  println!("{}", item);
}
```

## Installation

To use [crate-name], add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
crate-name = "0.1.0"
```

### Contribution Guidelines Section

This section is crucial for open-source projects as it sets the tone for how contributions are managed. Since you're targeting developers, this section can be fairly technical, explaining how to set up the development environment, run tests, and contribute changes.

### Contributing

We welcome contributions! Here's how you can contribute:

1. Fork the repository.
2. Clone your fork: `git clone https://github.com/yourusername/crate-name.git`
3. Create a new branch: `git checkout -b my-feature-branch`
4. Make your changes.
5. Run tests: `cargo test`
6. Commit your changes: `git commit -am 'Add my feature'`
7. Push to your branch: `git push origin my-feature-branch`
8. Create a Pull Request on GitHub.

Please make sure your contributions adhere to our coding guidelines.
