#![cfg_attr(not(feature = "turso"), allow(dead_code))]

//! Turso-backed substrate implementation for Iklo.
//!
//! This crate provides a `Substrate` trait implementation using Turso as the backing database.
//! The Turso implementation is gated behind the `turso` feature and is not enabled by default.
