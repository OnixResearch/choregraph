let model = import "../minimal-core-model.ncl" in
model [{ name = "tokio", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
