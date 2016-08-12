use libc;
use std::io;

#[cfg(target_os="macos")]
const TIOCGWINSZ: libc::c_ulonglong = 0x5413;
#[cfg(target_os="macos")]
const TIOCSWINSZ: libc::c_ulonglong = 0x5414;

#[cfg(target_os="linux")]
const TIOCGWINSZ: libc::c_ulong = 0x5413;
#[cfg(target_os="linux")]
const TIOCSWINSZ: libc::c_ulong = 0x5414;

#[repr(C)]
#[derive(PartialEq, Debug, Default)]
pub struct Winsize {
    pub ws_row: libc::c_ushort, // rows, in characters
    pub ws_col: libc::c_ushort, // columns, in characters
    pub ws_xpixel: libc::c_ushort, // horizontal size, pixels
    pub ws_ypixel: libc::c_ushort, // vertical size, pixels
}

pub fn from_fd(fd: libc::c_int) -> io::Result<Winsize> {
    let winsize = Winsize::default();

    unsafe {
        libc::ioctl(fd, TIOCGWINSZ, &winsize);
    }

    Ok(winsize)
}

pub fn set(fd: libc::c_int, winsize: &Winsize) {
    unsafe {
        libc::ioctl(fd, TIOCSWINSZ, winsize);
    }
}
