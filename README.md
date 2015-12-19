# pty-shell

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]

pty = '0.1.6'
pty-shell = '0.1.0'
```

and this to your crate root:

```rust
extern crate pty;
extern crate pty_shell;

use pty_shell::{PtyProxy, PtyHandler};
use std::process::{Command, Stdio};

struct Shell;
impl PtyHandler for Shell {
    fn input(&mut self, _data: Vec<u8>) {
        Command::new("aplay").arg("sound1.wav").stderr(Stdio::null()).spawn().unwrap();
    }

    fn output(&mut self, _data: Vec<u8>) {
        Command::new("aplay").arg("sound2.wav").stderr(Stdio::null()).spawn().unwrap();
    }
}

fn main() {
    let child = pty::fork().unwrap();

    child.exec("bash").unwrap();
    child.proxy(Shell).unwrap();
    child.wait().unwrap();
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
