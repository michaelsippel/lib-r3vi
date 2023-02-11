#![feature(trait_alias)]

//! Rust Runtime for Reactive View-Projections
//!
//! Using **r3vi** you can define *Projections*, i.e. transformations on *Views* that are
//! updated *reactively* on changes in the source-view.
//! These updates are performed incrementally using fine-granuar diffs.
//!
//! *Views* are abstract accessor-interfaces that also define the update protocol (the diff).
//! *Observers* can register to observe a *View* and are notified with the according diff-message
//! whenever the view changes.
//! *Projections* are transformations from one view into antoher.
//! They are made of the target view and an observer that observes the source view.
//!
//! R3vi provides basic data-structures and projections to build projectional pipelines
//! with an interface similar to native rust iterators.
//!
//!
//!# Examples
//!
//! ```
//! use r3vi::buffer::vec::*;
//!
//! let mut buffer = VecBuffer::<i32>::new();
//! buffer.push(3);
//!
//! let projected_port = buffer.get_port()
//!                       .to_sequence()     // make SequenceView from Vec
//!                       .map(|x| x + 10)
//!                       .filter(|x| x > 10);
//!
//! let projected_view = projected_port.get_view();
//!
//! assert_eq!(projected_view.get(&0), Some(13));
//!
//! buffer.push(5);   // maps to 15
//! buffer.push(-9);  // maps to 1, is eliminated by filter
//! buffer.push(1);   // maps to 11
//!
//! assert_eq!(projected_view.get(&1), Some(15));
//! assert_eq!(projected_view.get(&2), Some(11));
//!
//! ```

pub mod view;
pub mod buffer;
pub mod projection;

