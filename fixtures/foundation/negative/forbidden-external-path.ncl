let model = import "../minimal-core-model.ncl" in
model [{ name = "execution-graph-core", source = "external-path" }] "#![no_std]\nextern crate alloc;\n"
