let model = import "../minimal-core-model.ncl" in
model [{ name = "cap-std", source = "registry" }] "#![no_std]\nextern crate alloc;\n"
