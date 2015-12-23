use libc;
use libc::funcs::bsd44::ioctl;
use std::io;

#[cfg(target_os="macos")]
const TIOCGWINSZ: libc::c_ulonglong = 0x5413;
#[cfg(target_os="macos")]
const TIOCSWINSZ: libc::c_ulonglong = 0x5414;

#[cfg(target_os="linux")]
const TIOCGWINSZ: libc::c_int = 0x5413;
#[cfg(target_os="linux")]
const TIOCSWINSZ: libc::c_int = 0x5414;

#[repr(C)]
#[derive(PartialEq, Debug)]
pub struct Winsize {
    pub ws_row: libc::c_ushort, // rows, in characters
    pub ws_col: libc::c_ushort, // columns, in characters
    pub ws_xpixel: libc::c_ushort, // horizontal size, pixels
    pub ws_ypixel: libc::c_ushort, // vertical size, pixels
}

pub fn from_fd(fd: libc::c_int) -> io::Result<Winsize> {
    let winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        ioctl(fd, TIOCGWINSZ, &winsize);
    }

    Ok(winsize)
}

pub fn set(fd: libc::c_int, winsize: &Winsize) {
    unsafe {
        ioctl(fd, TIOCSWINSZ, winsize);
    }
}
