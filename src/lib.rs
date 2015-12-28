#![deny(missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate libc;
extern crate nix;
extern crate pty;
extern crate termios;
extern crate mio;

use std::{fmt, io, result, thread, process};
use std::io::{Read, Write};

pub use self::error::*;
pub use self::terminal::*;
use self::raw_handler::*;
use self::winsize::Winsize;

pub mod error;
pub mod terminal;
pub mod winsize;

mod raw_handler;
mod command;

pub type Result<T> = result::Result<T, Error>;

pub trait PtyHandler {
    fn input(&mut self, _data: &[u8]) {}
    fn output(&mut self, _data: &[u8]) {}
    fn resize(&mut self, _winsize: &Winsize) {}
}

pub trait PtyProxy {
    fn exec<S: AsRef<str>>(&self, shell: S) -> Result<()>;
    fn proxy<H: PtyHandler>(&self, handler: H) -> Result<()>;
    fn do_proxy<H: PtyHandler>(&self, handler: H) -> Result<()>;
}

impl PtyProxy for pty::Child {
    fn exec<S: AsRef<str>>(&self, shell: S) -> Result<()> {
        if self.pid() == 0 {
            command::exec(shell);
        }

        Ok(())
    }

    fn proxy<H: PtyHandler + 'static>(&self, handler: H) -> Result<()> {
        if self.pid() != 0 {
            try!(setup_terminal(self.pty().unwrap()));
            try!(self.do_proxy(handler));
        }

        Ok(())
    }

    fn do_proxy<H: PtyHandler + 'static>(&self, handler: H) -> Result<()> {
        let mut event_loop = try!(mio::EventLoop::new());

        let mut writer = self.pty().unwrap();
        let (input_reader, mut input_writer) = try!(mio::unix::pipe());

        thread::spawn(move || {
            handle_input(&mut writer, &mut input_writer).unwrap_or_else(|e| {
                println!("{:?}", e);
                process::exit(1);
            });
        });

        let mut reader = self.pty().unwrap();
        let (output_reader, mut output_writer) = try!(mio::unix::pipe());
        let message_sender = event_loop.channel();

        thread::spawn(move || {
            handle_output(&mut reader, &mut output_writer).unwrap_or_else(|e| {
                println!("{:?}", e);
                process::exit(1);
            });

            message_sender.send(Instruction::Shutdown).unwrap();
        });

        try!(event_loop.register(&input_reader, INPUT, mio::EventSet::readable(), mio::PollOpt::level()));
        try!(event_loop.register(&output_reader, OUTPUT, mio::EventSet::readable(), mio::PollOpt::level()));

        RawHandler::register_sigwinch_handler();

        let mut raw_handler = RawHandler::new(input_reader, output_reader, self.pty().unwrap(), Box::new(handler));

        try!(event_loop.run(&mut raw_handler));

        Ok(())
    }
}

pub struct PtyCallback {
    input_handler: Box<FnMut(&[u8])>,
    output_handler: Box<FnMut(&[u8])>,
}

impl PtyCallback {
    pub fn new<I, O>(input_handler: I, output_handler: O) -> Self
        where I: FnMut(&[u8]) + 'static, O: FnMut(&[u8]) + 'static
    {
        PtyCallback {
            input_handler: Box::new(input_handler),
            output_handler: Box::new(output_handler),
        }
    }
}

impl fmt::Debug for PtyCallback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(PtyCallback)")
    }
}

impl PtyHandler for PtyCallback {
    fn input(&mut self, data: &[u8]) {
        (&mut *self.input_handler)(data);
    }

    fn output(&mut self, data: &[u8]) {
        (&mut *self.output_handler)(data);
    }
}

fn handle_input(writer: &mut pty::ChildPTY, handler_writer: &mut mio::unix::PipeWriter) -> Result<()> {
    let mut input = io::stdin();
    let mut buf   = [0; 128];

    loop {
        let nread = try!(input.read(&mut buf));

        try!(writer.write(&buf[..nread]));
        try!(handler_writer.write(&buf[..nread]));
    }
}

fn handle_output(reader: &mut pty::ChildPTY, handler_writer: &mut mio::unix::PipeWriter) -> Result<()> {
    let mut output = io::stdout();
    let mut buf    = [0; 1024 * 10];

    loop {
        let nread = try!(reader.read(&mut buf));

        if nread <= 0 {
            break;
        } else {
            try!(output.write(&buf[..nread]));
            let _ = output.flush();

            try!(handler_writer.write(&buf[..nread]));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate pty;

    use PtyProxy;
    use PtyHandler;
    use terminal::restore_termios;

    struct TestHandler;
    impl PtyHandler for TestHandler {
        fn output(&mut self, data: &[u8]) {
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
