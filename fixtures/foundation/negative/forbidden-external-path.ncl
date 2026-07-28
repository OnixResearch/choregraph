let model = import "../minimal-core-model.ncl" in
model [{ name = "execution-graph-core", requirement = "=0.1.0", source = "external-path", path = "../execution-graph" }] "#![no_std]\nextern crate alloc;\n"
