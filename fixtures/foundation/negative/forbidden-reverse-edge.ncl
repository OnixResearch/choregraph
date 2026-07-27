let model = import "../minimal-core-model.ncl" in
model [{ name = "choregraph-nickel", source = "workspace", path = "" }] "#![no_std]\nextern crate alloc;\n"
