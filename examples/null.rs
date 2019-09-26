#![warn(clippy::unimplemented)]

use async_trait::async_trait;
use std::{borrow::Cow, env, io, path::PathBuf};
use tokio_fuse::{
    fs::Filesystem,
    op::{OperationResult, Operations},
    reply::{AttrOut, OpenOut, WriteOut},
    request::{Header, OpGetattr, OpOpen, OpRead, OpRelease, OpSetattr, OpWrite},
};

#[tokio::main(single_thread)]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "tokio_fuse=debug");
    pretty_env_logger::init();

    let mountpoint = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, ""))?;
    if !mountpoint.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "the mountpoint must be a regular file",
        ));
    }

    let mut fs = Filesystem::new(mountpoint)?;
    let mut op = Null;

    loop {
        if fs.receive().await? {
            break;
        }
        fs.process(&mut op).await?;
    }

    Ok(())
}

struct Null;

#[async_trait(?Send)]
impl Operations for Null {
    async fn getattr<'a>(
        &'a mut self,
        header: &'a Header,
        _: &'a OpGetattr,
    ) -> OperationResult<AttrOut> {
        match header.nodeid() {
            1 => Ok(root_attr().into()),
            _ => Err(libc::ENOENT),
        }
    }

    async fn setattr<'a>(
        &'a mut self,
        header: &'a Header,
        _: &'a OpSetattr,
    ) -> OperationResult<AttrOut> {
        match header.nodeid() {
            1 => Ok(root_attr().into()),
            _ => Err(libc::ENOENT),
        }
    }

    async fn open<'a>(&'a mut self, header: &'a Header, _: &'a OpOpen) -> OperationResult<OpenOut> {
        match header.nodeid() {
            1 => Ok(OpenOut::default()),
            _ => Err(libc::ENOENT),
        }
    }

    async fn read<'a>(
        &'a mut self,
        header: &'a Header,
        _: &'a OpRead,
    ) -> OperationResult<Cow<'a, [u8]>> {
        match header.nodeid() {
            1 => Ok(Cow::Borrowed(&[] as &[u8])),
            _ => Err(libc::ENOENT),
        }
    }

    async fn write<'a>(
        &'a mut self,
        header: &'a Header,
        _: &'a OpWrite,
        buf: &'a [u8],
    ) -> OperationResult<WriteOut> {
        match header.nodeid() {
            1 => {
                let mut out = WriteOut::default();
                out.set_size(buf.len() as u32);
                Ok(out)
            }
            _ => Err(libc::ENOENT),
        }
    }

    async fn release<'a>(
        &'a mut self,
        header: &'a Header,
        _: &'a OpRelease,
    ) -> OperationResult<()> {
        match header.nodeid() {
            1 => Ok(()),
            _ => Err(libc::ENOENT),
        }
    }
}

fn root_attr() -> libc::stat {
    let mut attr: libc::stat = unsafe { std::mem::zeroed() };
    attr.st_mode = libc::S_IFREG | 0o644;
    attr.st_nlink = 1;
    attr.st_uid = unsafe { libc::getuid() };
    attr.st_gid = unsafe { libc::getgid() };
    attr
}
