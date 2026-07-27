let model = import "../minimal-core-model.ncl" in
model [{ name = "tokio", source = "registry" }] "#![no_std]\nextern crate alloc;\n"
