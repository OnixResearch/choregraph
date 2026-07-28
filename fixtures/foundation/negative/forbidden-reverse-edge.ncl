let model = import "../minimal-core-model.ncl" in
model [{ name = "choregraph-nickel", requirement = "=0.1.0", source = "workspace", path = "" }] "#![no_std]\nextern crate alloc;\n"
