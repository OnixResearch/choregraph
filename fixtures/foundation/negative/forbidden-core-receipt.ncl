let model = import "../minimal-core-model.ncl" in
model [{ name = "kamacite-core", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
