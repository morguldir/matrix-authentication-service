// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// TODO: Unlink the UNIX socket on drop?

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use hyper::server::accept::Accept;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream, UnixListener, UnixStream},
};

pub enum SocketAddr {
    Unix(tokio::net::unix::SocketAddr),
    Net(std::net::SocketAddr),
}

impl From<tokio::net::unix::SocketAddr> for SocketAddr {
    fn from(value: tokio::net::unix::SocketAddr) -> Self {
        Self::Unix(value)
    }
}

impl From<std::net::SocketAddr> for SocketAddr {
    fn from(value: std::net::SocketAddr) -> Self {
        Self::Net(value)
    }
}

impl std::fmt::Debug for SocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unix(l) => std::fmt::Debug::fmt(l, f),
            Self::Net(l) => std::fmt::Debug::fmt(l, f),
        }
    }
}

pub enum UnixOrTcpListener {
    Unix(UnixListener),
    Tcp(TcpListener),
}

impl From<UnixListener> for UnixOrTcpListener {
    fn from(listener: UnixListener) -> Self {
        Self::Unix(listener)
    }
}

impl From<TcpListener> for UnixOrTcpListener {
    fn from(listener: TcpListener) -> Self {
        Self::Tcp(listener)
    }
}

impl TryFrom<std::os::unix::net::UnixListener> for UnixOrTcpListener {
    type Error = std::io::Error;

    fn try_from(listener: std::os::unix::net::UnixListener) -> Result<Self, Self::Error> {
        Ok(Self::Unix(UnixListener::from_std(listener)?))
    }
}

impl TryFrom<std::net::TcpListener> for UnixOrTcpListener {
    type Error = std::io::Error;

    fn try_from(listener: std::net::TcpListener) -> Result<Self, Self::Error> {
        Ok(Self::Tcp(TcpListener::from_std(listener)?))
    }
}

impl UnixOrTcpListener {
    /// Get the local address of the listener
    ///
    /// # Errors
    ///
    /// Returns an error on rare cases where the underlying [`TcpListener`] or
    /// [`UnixListener`] couldn't provide the local address
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        match self {
            Self::Unix(listener) => listener.local_addr().map(SocketAddr::from),
            Self::Tcp(listener) => listener.local_addr().map(SocketAddr::from),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = UnixOrTcpConnectionProj]
    pub enum UnixOrTcpConnection {
        Unix {
            #[pin]
            stream: UnixStream,
        },

        Tcp {
            #[pin]
            stream: TcpStream,
        },
    }
}

impl UnixOrTcpConnection {
    /// Get the local address of the stream
    ///
    /// # Errors
    ///
    /// Returns an error on rare cases where the underlying [`TcpStream`] or
    /// [`UnixStream`] couldn't provide the local address
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        match self {
            Self::Unix { stream, .. } => stream.local_addr().map(SocketAddr::from),
            Self::Tcp { stream, .. } => stream.local_addr().map(SocketAddr::from),
        }
    }

    /// Get the remote address of the stream
    ///
    /// # Errors
    ///
    /// Returns an error on rare cases where the underlying [`TcpStream`] or
    /// [`UnixStream`] couldn't provide the remote address
    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        match self {
            Self::Unix { stream, .. } => stream.peer_addr().map(SocketAddr::from),
            Self::Tcp { stream, .. } => stream.peer_addr().map(SocketAddr::from),
        }
    }
}

impl Accept for UnixOrTcpListener {
    type Error = std::io::Error;
    type Conn = UnixOrTcpConnection;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        let conn = match &*self {
            Self::Unix(listener) => {
                let (stream, _remote_addr) = ready!(listener.poll_accept(cx))?;
                UnixOrTcpConnection::Unix { stream }
            }

            Self::Tcp(listener) => {
                let (stream, _remote_addr) = ready!(listener.poll_accept(cx))?;
                UnixOrTcpConnection::Tcp { stream }
            }
        };

        Poll::Ready(Some(Ok(conn)))
    }
}

impl AsyncRead for UnixOrTcpConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.project() {
            UnixOrTcpConnectionProj::Unix { stream } => stream.poll_read(cx, buf),
            UnixOrTcpConnectionProj::Tcp { stream } => stream.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for UnixOrTcpConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.project() {
            UnixOrTcpConnectionProj::Unix { stream } => stream.poll_write(cx, buf),
            UnixOrTcpConnectionProj::Tcp { stream } => stream.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.project() {
            UnixOrTcpConnectionProj::Unix { stream } => stream.poll_flush(cx),
            UnixOrTcpConnectionProj::Tcp { stream } => stream.poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.project() {
            UnixOrTcpConnectionProj::Unix { stream } => stream.poll_shutdown(cx),
            UnixOrTcpConnectionProj::Tcp { stream } => stream.poll_shutdown(cx),
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.project() {
            UnixOrTcpConnectionProj::Unix { stream } => stream.poll_write_vectored(cx, bufs),
            UnixOrTcpConnectionProj::Tcp { stream } => stream.poll_write_vectored(cx, bufs),
        }
    }

    fn is_write_vectored(&self) -> bool {
        match self {
            UnixOrTcpConnection::Unix { stream } => stream.is_write_vectored(),
            UnixOrTcpConnection::Tcp { stream } => stream.is_write_vectored(),
        }
    }
}