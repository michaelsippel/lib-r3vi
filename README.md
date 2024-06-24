# r3vi - Rust Runtime for Reactive View-Projections

**r3vi** is a Rust library designed to handle reactive transformations on data views, known as *Projections*. These projections are updated incrementally and efficiently whenever the source data changes, utilizing fine-grained diffs.

## Key Concepts
- **Views**: Abstract accessor interfaces that define the protocol for updates, including the structure of the diffs.
- **Observers**: Entities that can register to monitor a *View* and receive notifications with the appropriate diff messages whenever the view undergoes changes.
- **Projections**: Transformations from one view to another. A projection comprises a target view and an observer that listens to changes in the source view and updates the target view accordingly.
r3vi provides essential data structures and projections to build projectional pipelines with an interface similar to native Rust iterators, enabling seamless and reactive data transformations.

### Examples
Below is an example demonstrating how to use **r3vi** to create and manipulate reactive views:

```rust
use r3vi::buffer::vec::*;

fn main() {
    // Create a new VecBuffer of i32
    let mut buffer = VecBuffer::<i32>::new();
    buffer.push(3);

    // Create a projected port from the buffer
    let projected_port = buffer.get_port()
                          .to_sequence()  // Convert VecBuffer to SequenceView
                          .map(|x| x + 10)
                          .filter(|x| *x > 10);

    // Obtain the projected view
    let projected_view = projected_port.get_view();

    // Assert initial values in the projected view
    assert_eq!(projected_view.get(&0), Some(13));

    // Add more elements to the buffer and observe changes in the projected view
    buffer.push(5);   // 5 maps to 15 in the projection
    buffer.push(-9);  // -9 maps to 1, which is filtered out
    buffer.push(1);   // 1 maps to 11 in the projection

    // Assert updated values in the projected view
    assert_eq!(projected_view.get(&1), Some(15));
    assert_eq!(projected_view.get(&2), Some(11));
}
```

### How It Works
1. **Creating a Buffer**: We start by creating a `VecBuffer` to store integers.
2. **Defining a Projection**: Using `get_port()`, we convert the buffer to a sequence view, apply a map transformation to add 10 to each element, and then filter out values less than or equal to 10.
3. **Obtaining a View**: The transformed data is accessed through `get_view()`, allowing us to interact with the projected data.
4. **Reactive Updates**: As new elements are added to the buffer, the projections update reactively, maintaining the transformations and filters defined.

This example demonstrates the reactive nature of **r3vi**'s projections, where changes in the source data are automatically and incrementally propagated to the target view.

### Getting Started
To start using **r3vi**, add it to your `Cargo.toml`:

```toml
[dependencies]
r3vi = "0.1"
```

For detailed documentation and more examples, please refer to the official documentation.

## License
This project is licensed under the [GPLv3](COPYING) License.

