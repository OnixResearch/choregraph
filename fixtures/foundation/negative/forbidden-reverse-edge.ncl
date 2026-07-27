let model = import "../minimal-core-model.ncl" in
model [{ name = "choregraph-nickel", source = "workspace" }] "#![no_std]\nextern crate alloc;\n"
