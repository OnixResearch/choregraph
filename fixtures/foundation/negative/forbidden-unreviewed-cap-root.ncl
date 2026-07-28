let model = import "../minimal-core-model.ncl" in
model [{ name = "cap-root-core", requirement = "=0.1.0", source = "local-path", path = "../cap-root/crates/cap-root-core" }] "#![no_std]\nextern crate alloc;\n"
