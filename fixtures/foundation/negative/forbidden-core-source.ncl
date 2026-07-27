let model = import "../minimal-core-model.ncl" in
model [] "#![no_std]\nextern crate alloc;\nuse std::fs;\n"
