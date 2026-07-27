let model = import "../minimal-core-model.ncl" in
model [{ name = "basalt", source = "registry" }] "#![no_std]\nextern crate alloc;\n"
