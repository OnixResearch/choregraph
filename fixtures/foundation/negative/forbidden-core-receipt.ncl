let model = import "../minimal-core-model.ncl" in
model [{ name = "kamacite-core", source = "registry" }] "#![no_std]\nextern crate alloc;\n"
