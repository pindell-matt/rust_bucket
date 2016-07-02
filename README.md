## Rust Bucket (work in progress)
Check out this [example repo](https://github.com/selfup/r_b_i_e) showing Iron using Rust Bucket to get started on stable.

### Goals
* Provide a simple JSON key-value store API
* Write to the filesystem for persistence
* Flexible structured or unstructured tables / records
* Can infer any data type with Serialize / Deserialize implementations via [Serde](https://github.com/serde-rs/serde)
* Works on stable, beta, nightly
* Synchronous performance
* Suitable for microservices
* Quick to setup

### Drawbacks
* No mmap
* Files don't lock, all usage should be synchronous
* Currently a library and not a server
* Project dependent
* Not yet on crates.io
* Performance is heavily influenced by SSD / HDD io speeds (but storage is cheap)
