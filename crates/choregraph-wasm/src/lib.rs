//! Canonical, stateless byte adapter for the Choregraph production kernel.
//!
//! r[impl choregraph.wasm.module]
//! r[impl choregraph.wasm.boundary]
//!
//! The pure dispatcher re-admits all artifacts on every call. The component shell only
//! transports bytes. It grants no host, effect, persistence, scheduling, or authorization
//! authority.

mod codec;
mod dispatch;
mod schema;

pub use dispatch::{
    CLAIM_BOUNDARY, INPUT_IDENTITY_CONTEXT, MAX_OUTER_REQUEST_BYTES, MAX_OUTER_RESULT_BYTES,
    MAX_TRACE_EVENTS, decode_result, dispatch, encode_call,
};
pub use schema::*;

#[cfg(target_arch = "wasm32")]
mod component {
    wit_bindgen::generate!({
        path: "wit",
        world: "kernel",
    });

    struct KernelComponent;

    impl Guest for KernelComponent {
        fn call(request: Vec<u8>) -> Vec<u8> {
            crate::dispatch(&request)
        }
    }

    export!(KernelComponent);
}
