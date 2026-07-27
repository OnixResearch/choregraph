let model = import "../minimal-core-model.ncl" in
model [{ name = "basalt", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
