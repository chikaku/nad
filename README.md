# Nad

Incomplete Lua VM only bytecode is supported for now

## Usage

- Dump function prototype

```bash
cargo run -- -dump -debug /path/to/bytecode
# or
cargo install nad
nad -debug -dump /path/to/bytecode
```

- Execute bytecode file

```bash
cargo run -- -debug /path/to/bytecode
# or
cargo install nad
nad -debug /path/to/bytecode
```

- Use `nad` library

```rust
use nad::State;
use nad::Reader;

fn main() {
    let path = "/path/to/bytecode";
    
    // read prototype
    let prototype = Reader::from_file(path).prototype();
    
    // execute main function
    State::from_file(path).call(0, 0);
}
```

## TODO

- Metatable 
- Iterator
- Error handler
- ...
