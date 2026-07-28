let model = import "../minimal-core-model.ncl" in
model [{ name = "cap-std", requirement = "*", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
