let model = import "../minimal-core-model.ncl" in
model [{ name = "nickel-lang", requirement = "*", source = "registry", path = "" }] "#![no_std]\nextern crate alloc;\n"
