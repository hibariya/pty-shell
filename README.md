# pty-shell

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]

pty = '0.1.6'
pty-shell = '0.1.0'
```

For example, add src/main.rs as following:

```rust
extern crate pty;
extern crate pty_shell;

use pty_shell::{PtyProxy, PtyHandler};
use std::process::{Command, Stdio};

struct Shell;
impl PtyHandler for Shell {
    fn input(&mut self, _data: Vec<u8>) {
        aplay("sound-effect-for-input.wav");
    }

    fn output(&mut self, _data: Vec<u8>) {
        aplay("sound-effect-for-output.wav");
    }
}

fn aplay<F: AsRef<str>>(file: F) {
    Command::new("aplay").arg(file.as_ref()).stderr(Stdio::null()).spawn();
}

fn main() {
    let child = pty::fork().unwrap();

    child.exec("bash");
    child.proxy(Shell);
    child.wait();
}
```

## Contributing

1. Fork it ( https://github.com/hibariya/pty-shell/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## License

Copyright (c) 2015 Hika Hibariya

Distributed under the [MIT License](LICENSE.txt).
