let model = import "../minimal-core-model.ncl" in
model [{ name = "execution-graph-core", source = "external-path", path = "../execution-graph" }] "#![no_std]\nextern crate alloc;\n"
