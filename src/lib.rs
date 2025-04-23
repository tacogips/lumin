//! # Lumin
//! 
//! Lumin is a library for searching and displaying local files.
//! 
//! ## Features
//! 
//! * File searching - Search file contents using regex patterns
//! * File traversal - Explore directory structures with customizable filters
//! * File viewing - Display file contents with type detection and metadata
//! * Directory tree - Display directory structures in a hierarchical tree format

/// File content searching functionality using regex patterns
pub mod search;
/// Directory traversal and file listing functionality
pub mod traverse;
/// File content viewing with type detection and formatting
pub mod view;
/// Directory tree structure visualization
pub mod tree;
