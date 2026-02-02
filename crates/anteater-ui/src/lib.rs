//! Anteater UI Layer
//!
//! This crate provides the egui-based user interface for the Anteater debugger.
//! It implements a flexible docking panel system inspired by RAD Debugger.

#![allow(dead_code, unused_variables)]

pub mod app;
pub mod panels;
pub mod widgets;
pub mod mock;
pub mod theme;

pub use app::AnteaterApp;
pub use theme::ITerm2Theme;
