#![deny(missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate libc;
extern crate nix;
extern crate pty;
extern crate termios;
extern crate mio;

pub use self::shell::*;

mod shell;

mod pty_shell {
}

#[cfg(test)]
mod tests {
    extern crate pty;

    use PtyProxy;
    use PtyHandler;
    use shell::restore_termios;

    struct TestHandler;
    impl PtyHandler for TestHandler {
        fn input(&mut self, _data: Vec<u8>) {}
        fn output(&mut self, data: Vec<u8>) {
            assert!(data.len() != 0);
        }
    }

    #[test]
    fn it_can_spawn() {
        let child = pty::fork().unwrap();
        restore_termios();

        child.exec("pwd").unwrap();

        assert!(child.wait().is_ok());
    }

    #[test]
    fn it_can_hook_stdout_with_handler() {
        let child = pty::fork().unwrap();
        restore_termios();

        child.proxy(TestHandler).unwrap();
        child.exec("pwd").unwrap();

        assert!(child.wait().is_ok());
    }

    #[test]
    fn it_can_hook_stdout_with_callback() {
        use PtyCallback;

        let child = pty::fork().unwrap();
        restore_termios();

        child.proxy(PtyCallback::new(
            |_input| {},
            |output| assert!(output.len() != 0),
        )).unwrap();
        child.exec("pwd").unwrap();

        assert!(child.wait().is_ok());
    }
}
