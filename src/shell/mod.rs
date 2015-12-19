use std::{io, result, thread, process};
use std::io::{Read, Write};
use pty;
use mio;

pub use self::error::*;
use self::raw_handler::*;

pub mod winsize;
pub mod error;

mod raw_handler;
mod command;
mod terminal;

pub type Result<T> = result::Result<T, Error>;

pub trait PtyHandler {
    fn input(&mut self, _data: Vec<u8>) {}
    fn output(&mut self, _data: Vec<u8>) {}
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
            try!(terminal::setup_terminal(self.pty().unwrap()));
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
