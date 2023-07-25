use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncRead {
    type Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}
