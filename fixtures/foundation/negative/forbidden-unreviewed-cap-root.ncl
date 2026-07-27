let model = import "../minimal-core-model.ncl" in
model [{ name = "cap-root-core", source = "local-path", path = "../cap-root/crates/cap-root-core" }] "#![no_std]\nextern crate alloc;\n"
