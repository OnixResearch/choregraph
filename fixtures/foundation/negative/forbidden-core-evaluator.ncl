let model = import "../minimal-core-model.ncl" in
model [{ name = "nickel-lang", source = "registry" }] "#![no_std]\nextern crate alloc;\n"
