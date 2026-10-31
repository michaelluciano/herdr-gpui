//! Git graph view model: the drawing geometry of one commit row and the
//! selection and request bookkeeping behind the view. No GPUI types, so every
//! rule here is tested without a window.

pub(crate) mod geometry;
mod load;
mod render;
pub(crate) mod state;
mod tab;

#[cfg(test)]
mod tests;
