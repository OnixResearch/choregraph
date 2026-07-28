let model = import "../minimal-core-model.ncl" in
model [{ name = "tokio", requirement = "*", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
